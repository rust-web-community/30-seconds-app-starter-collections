use crate::schemas::User;
use async_trait::async_trait;
use bytes::Bytes;
use uuid::Uuid;

// The repository abstraction allow to swap for a different backend than postgres while keeping all the rest code, if you wish to
// Caching users is a good idea, but for security reason, the cache should be private, short-lived, and invalidated as needed: role change, password change etc.
#[async_trait]
pub trait UserRepository {
    async fn get_user(&self, id: Uuid) -> Option<User>;
    async fn create_user(&self, u: &User) -> Result<(), ()>;
}

#[async_trait]
pub trait CacheRepository {
    async fn get_user(&self, id: Uuid) -> Option<User>;
    async fn create_user(&self, u: &User) -> Result<(), ()>;
}

#[async_trait]
pub trait Proxy {
    async fn make_request(&self, method: &str, url: &str, user_id: &Uuid) -> (u16, Bytes);
}
