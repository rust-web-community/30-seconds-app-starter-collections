use crate::schemas::{Todo, TodoUpdateRequest};
use coi::Inject;
use async_trait::async_trait;

#[async_trait]
pub trait TodoRepository: Inject {
    async fn read_all(&self) -> Vec<Todo>;
    async fn read_one(&self, id: i64) -> Result<Todo, ()>;
    async fn create_one(&self, t: &Todo) -> Result<(), Todo>;
    async fn update_one(&self, id: i64, t: TodoUpdateRequest) -> Result<Todo, ()>;
    async fn delete_one(&self, id: i64) -> Result<(), ()>;
    async fn read_filter(&self, search_text: &str) -> Vec<Todo>;
}
