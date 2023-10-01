use std::{str::FromStr, sync::Arc};
use crate::stores::config::{self, get_key};
use jwt_simple::prelude::{JWTClaims, NoCustomClaims, MACLike, Claims, Duration};
use crate::store_interface::UserRepository;
use bytes::Bytes;
use uuid::Uuid;

// Implements proxying any method towards authenticated microservices.
pub async fn proxy(repository: Arc<dyn UserRepository>, method: &str, path: &str, cookie: Option<String>) -> Result<(Bytes, u16, String), u16> {
    // For the purpose of browser testing, we use a session cookie.
    // Not authenticated - Could redirect to a front signup page
    if cookie.is_none(){
        return Err(401);
    }
    // Check token signature
    // In real life, keys should be taken from env, and injected in your production via CI/CD tools
    let key = get_key();

    let try_claims: Result<JWTClaims<NoCustomClaims>, jwt_simple::Error> = key.verify_token(cookie.unwrap().as_str(), None);
    if try_claims.is_err(){
        return Err(401);
    }
    let claims = try_claims.unwrap();
    let try_user_id = claims.subject;
    if try_user_id.is_none(){
        return Err(401)
    }
    let user_id_str = try_user_id.unwrap();
    let user_id = Uuid::from_str(&user_id_str).unwrap();
    // We also verify the user exists. This enable for permission check
    let user = repository.get_user(user_id.clone()).await;
    if user.is_none(){
        return Err(401);
    }
    // User is authenticated. Cookie jwt could be refreshed starting from here

    // Lazy load of the dynamic routing config file
    let my_config = config::get_config();
    for route in &my_config.routes {
        // See config for more details
        if route.methods.contains(&method.to_owned()) && path.starts_with(&route.prefix) {
            if !user.unwrap().admin && route.restrict_admin{
                return Err(403);
            }
            // Prepare proxy request
            let client = reqwest::Client::new();
            let mut headers = reqwest::header::HeaderMap::new();
            // Transmit all necessary user info through HTTP headers; its agnostic of query methods and simplifies handling for services
            let header_name = reqwest::header::HeaderName::from_str("X-User").unwrap();
            let header_value = reqwest::header::HeaderValue::from_str(user_id_str.as_str()).unwrap();
            headers.insert(header_name, header_value);
            let stripped_path = path.strip_prefix(&route.prefix).unwrap();
            // We support proxying variable path this way
            let response = client.get(format!("http://{}{}", route.service, stripped_path)).headers(headers).send().await.unwrap();
            let status = response.status().as_u16();
            // Refresh token, to avoid cutting session during browsing
            let refresh_token = gen_session_token(user_id).await;
            return Ok((response.bytes().await.unwrap(), status, refresh_token))
        }
    }
    Err(404)
}

pub async fn gen_user(repository: Arc<dyn UserRepository>) -> Result<Uuid, ()> {
    let uuid = Uuid::new_v4();
    repository.create_user(uuid).await?;
    Ok(uuid)
}

pub async fn gen_session_token (user_id: Uuid) -> String {
    // Exact duration might vary based on business security requirements.
    let mut claims = Claims::create(Duration::from_days(1));
    claims.subject = Some(user_id.to_string());
    let key = get_key();
    key.authenticate(claims).unwrap()
}

