use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Task to do.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Todo {
    /// Unique id for the todo item.
    pub id: i64,
    /// Description of the tasks to do.
    pub value: String,
    /// Mark is the task done or not
    pub checked: bool,
}

/// Request to update existing `Todo` item.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct TodoUpdateRequest {
    /// Optional new value for the `Todo` task.
    pub value: Option<String>,
    /// Optional check status to mark is the task done or not.
    pub checked: Option<bool>,
}

/// Todo endpoint error responses
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub enum ErrorResponse {
    /// When Todo is not found by search term.
    NotFound(String),
    /// When there is a conflict storing a new todo.
    Conflict(String),
    /// When todo endpoint was called without correct credentials
    Unauthorized(String),
}