use coi::Inject;
use crate::schemas::User;
use async_trait::async_trait;
use uuid::Uuid;

// The repository abstraction allow to swap for a different backend than postgres while keeping all the rest code, if you wish to
#[async_trait]
pub trait UserRepository: Inject {
    async fn get_user(&self, id:Uuid) -> Option<User>;
    async fn create_user(&self, id: Uuid) -> Result<(), ()>;
}