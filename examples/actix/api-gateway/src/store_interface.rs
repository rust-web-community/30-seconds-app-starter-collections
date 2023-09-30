use coi::Inject;
use crate::schemas::User;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Inject {
    async fn get_user(&self, id:Uuid) -> Option<User>;
    async fn create_user(&self, id: Uuid) -> Result<(), ()>;
}