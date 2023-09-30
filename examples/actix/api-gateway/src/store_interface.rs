use coi::Inject;
use crate::schemas::User;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Inject {
    async fn get_user(&self, id: i64) -> Option<User>;
}