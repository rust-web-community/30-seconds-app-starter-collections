use uuid::Uuid;

/// Task to do.
#[derive(Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub admin: bool,
}
