use serde::{Deserialize, Serialize};

/// Task to do.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub admin: bool,
}
