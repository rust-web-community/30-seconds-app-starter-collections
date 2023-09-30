use coi::Inject;
use crate::schemas::User;
use async_trait::async_trait;
use uuid::Uuid;

// The repository abstraction allow to swap for a different backend than postgres while keeping all the rest code, if you wish to
// Caching users is a good idea, but for security reason, the cache should be private, short-lived, and invalidated as needed: role change, password change etc.
#[async_trait]
pub trait UserRepository: Inject {
    async fn get_user(&self, id:Uuid) -> Option<User>;
    async fn create_user(&self, id: Uuid) -> Result<(), ()>;
}