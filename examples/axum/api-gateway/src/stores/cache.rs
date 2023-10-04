use std::collections::HashMap;
use std::sync::RwLock;

use crate::schemas::CacheEntry;
pub use crate::schemas::User;
pub use crate::store_interface::CacheRepository;
use async_trait::async_trait;
use uuid::Uuid;

// Our cache implementation is a little naive and grow indefinitely in memory.
// This is not a huge deal if your user struct is small, which it should
// But at least, security wise, the risk of poisoning it are very slim; we don't rely on external services
// and we keep a max duration relatively short

pub struct InMemoryUser {
    users: RwLock<HashMap<Uuid, CacheEntry>>,
}

impl InMemoryUser {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl CacheRepository for InMemoryUser {
    async fn get_user(&self, id: Uuid) -> Option<User> {
        let cache = self.users.read().unwrap();
        let user = cache.get(&id);
        if user.is_some() {
            if user.unwrap().timestamp
                < std::time::Instant::now() - std::time::Duration::from_secs(60)
            {
                let mut w_cache = self.users.write().unwrap();
                w_cache.remove(&id);
                return None;
            }
            return Some(User {
                id: id,
                admin: user.unwrap().admin,
            });
        }
        None
    }

    async fn create_user(&self, u: &User) -> Result<(), ()> {
        let mut users = self.users.write().unwrap();
        users.insert(
            u.id,
            CacheEntry {
                admin: u.admin,
                timestamp: std::time::Instant::now(),
            },
        );
        Ok(())
    }
}
