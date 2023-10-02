use std::time::Instant;
use uuid::Uuid;

// Our simplified user model
// Only add access-related data in this User model; other business user data will be accessed by other services using the provided id
#[derive(Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub admin: bool,
}

pub struct CacheEntry {
    pub admin: bool,
    pub timestamp: Instant,
}
