pub use crate::store_interface::Proxy;
use async_trait::async_trait;
use bytes::Bytes;
use coi::{Inject, Provide};
pub use reqwest;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Default, Inject)]
pub struct RqClient {
    pub inner: reqwest::Client,
}

// Our cache implementation is a little naive and grow indefinitely in memory.
// This is not a huge deal if your user struct is small, which it should
// But at least, security wise, the risk of poisoning it are very slim; we don't rely on external services
// and we keep a max duration relatively short

#[derive(Provide)]
#[coi(provides pub dyn Proxy with RqClient::new())]
pub struct RqClientProvider;

impl RqClient {
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Proxy for RqClient {
    async fn make_request(&self, method: &str, url: &str, user_id: &Uuid) -> (u16, Bytes) {
        let mut headers = reqwest::header::HeaderMap::new();
        // Transmit all necessary user info through HTTP headers; its agnostic of query methods and simplifies handling for services
        let header_name = reqwest::header::HeaderName::from_str("X-User").unwrap();
        let header_value =
            reqwest::header::HeaderValue::from_str(user_id.to_string().as_str()).unwrap();
        headers.insert(header_name, header_value);

        let request = match method {
            "POST" => self.inner.post(url).headers(headers),
            "PUT" => self.inner.put(url).headers(headers),
            "DELETE" => self.inner.delete(url).headers(headers),
            _str => self.inner.get(url).headers(headers),
        };
        let result = request.send().await;
        if result.is_err() {
            return (500_u16, Bytes::default());
        }
        let response = result.unwrap();
        (response.status().as_u16(), response.bytes().await.unwrap())
    }
}
