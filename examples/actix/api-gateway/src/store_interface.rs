use crate::schemas::User;
use async_trait::async_trait;
use bytes::Bytes;
use coi::Inject;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Inject {
    async fn get_user(&self, id: Uuid) -> Option<User>;
    async fn create_user(&self, u: &User) -> Result<(), ()>;
}

#[async_trait]
pub trait Proxy: Inject {
    async fn make_request(&self, method: &str, url: &str, user_id: &Uuid) -> (u16, Bytes);
}
