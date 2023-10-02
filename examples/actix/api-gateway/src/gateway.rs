use crate::store_interface::Proxy;
use crate::store_interface::UserRepository;
use crate::stores::cache::User;
use crate::stores::config::{self, get_key};
use bytes::Bytes;
use jwt_simple::prelude::{Claims, Duration, JWTClaims, MACLike, NoCustomClaims};
use std::{str::FromStr, sync::Arc};
use uuid::Uuid;

// Implements proxying any method towards authenticated microservices.
pub async fn proxy(
    repository: Arc<dyn UserRepository>,
    cache: Arc<dyn UserRepository>,
    client: Arc<dyn Proxy>,
    method: &str,
    path: &str,
    cookie: Option<String>,
) -> Result<(Bytes, u16, String), u16> {
    // For the purpose of browser testing, we use a session cookie.
    // Not authenticated - Could redirect to a front signup page
    if cookie.is_none() {
        return Err(401);
    }
    // Check token signature
    // In real life, keys should be taken from env, and injected in your production via CI/CD tools
    let key = get_key();

    let try_claims: Result<JWTClaims<NoCustomClaims>, jwt_simple::Error> =
        key.verify_token(cookie.unwrap().as_str(), None);
    if try_claims.is_err() {
        return Err(401);
    }
    let claims = try_claims.unwrap();
    let try_user_id = claims.subject;
    if try_user_id.is_none() {
        return Err(401);
    }
    let user_id_str = try_user_id.unwrap();
    let user_id = Uuid::from_str(&user_id_str).unwrap();
    // We also verify the user exists. This enable for permission check

    let mut try_user: Option<User> = cache.get_user(user_id.clone()).await;
    if try_user.is_none() {
        let db_user = repository.get_user(user_id.clone()).await;
        if db_user.is_none() {
            return Err(401);
        }
        try_user = db_user;
    }
    let user: User = try_user.unwrap();
    let _res = cache.create_user(&user).await;
    // User is authenticated. Cookie jwt could be refreshed starting from here

    // Lazy load of the dynamic routing config file
    let my_config = config::get_config();

    for route in &my_config.routes {
        // See config for more details
        if route.methods.contains(&method.to_owned()) && path.starts_with(&route.prefix) {
            if !user.admin && route.restrict_admin {
                return Err(403);
            }
            // Prepare proxy request

            let stripped_path = path.strip_prefix(&route.prefix).unwrap();

            // We support proxying variable path this way
            let (status, resp_bytes) = client
                .make_request(
                    method,
                    format!("http://{}{}", route.service, stripped_path).as_str(),
                    &user.id,
                )
                .await;
            // Refresh token, to avoid cutting session during browsing
            let refresh_token = gen_session_token(user_id).await;
            return Ok((resp_bytes, status, refresh_token));
        }
    }

    Err(404)
}

pub async fn gen_user(repository: Arc<dyn UserRepository>) -> Result<Uuid, ()> {
    let user = User {
        id: Uuid::new_v4(),
        admin: false,
    };
    repository.create_user(&user).await?;
    Ok(user.id)
}

pub async fn gen_session_token(user_id: Uuid) -> String {
    // Exact duration might vary based on business security requirements.
    let mut claims = Claims::create(Duration::from_days(1));
    claims.subject = Some(user_id.to_string());
    let key = get_key();
    key.authenticate(claims).unwrap()
}
