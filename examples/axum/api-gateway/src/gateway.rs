use std::str::FromStr;
use crate::stores::config::{self, get_key};
use jwt_simple::prelude::{Claims, Duration, MACLike};
use bytes::Bytes;
use uuid::Uuid;
use crate::schemas::User;

// Implements proxying any method towards authenticated microservices.
pub async fn proxy(method: &str, path: &str, user: User) -> Result<(Bytes, u16, String), u16> {
    // User is authenticated. Cookie jwt could be refreshed starting from here

    // Lazy load of the dynamic routing config file
    let my_config = config::get_config();
    for route in &my_config.routes {
        // See config for more details
        if route.methods.contains(&method.to_owned()) && path.starts_with(&route.prefix) {
            if !user.admin && route.restrict_admin{
                return Err(403);
            }
            // Prepare proxy request
            let client = reqwest::Client::new();
            let mut headers = reqwest::header::HeaderMap::new();
            // Transmit all necessary user info through HTTP headers; its agnostic of query methods and simplifies handling for services
            let header_name = reqwest::header::HeaderName::from_str("X-User").unwrap();
            let header_value = reqwest::header::HeaderValue::from_str(user.id.to_string().as_str()).unwrap();
            headers.insert(header_name, header_value);
            let stripped_path = path.strip_prefix(&route.prefix).unwrap();
            // We support proxying variable path this way
            let response = client.get(format!("http://{}{}", route.service, stripped_path)).headers(headers).send().await.unwrap();
            let status = response.status().as_u16();
            // Refresh token, to avoid cutting session during browsing
            let refresh_token = gen_session_token(user.id).await;
            return Ok((response.bytes().await.unwrap(), status, refresh_token))
        }
    }
    Err(404)
}


pub async fn gen_session_token (user_id: Uuid) -> String {
    let mut claims = Claims::create(Duration::from_days(1));
    claims.subject = Some(user_id.to_string());
    let key = get_key();
    key.authenticate(claims).unwrap()
}

