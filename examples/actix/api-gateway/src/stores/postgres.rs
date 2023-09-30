
use deadpool_postgres::*;
use tokio_postgres::NoTls;
use coi::{Provide, Inject};
use uuid::Uuid;
use crate::store_interface::UserRepository;
use crate::schemas::User;
use async_trait::async_trait;

#[derive(Inject)]
pub struct PostgresUser {
    pub pool: deadpool_postgres::Pool
}

impl PostgresUser {
    pub fn new(pool: deadpool_postgres::Pool) -> Self {
        Self { pool: pool }
    }
}


#[derive(Provide)]
#[coi(provides pub dyn UserRepository with PostgresUser::new(self.pool.clone()))]
pub struct UserPostgresProvider {
    pub pool: deadpool_postgres::Pool
}

impl UserPostgresProvider
{
    pub async fn new(host: &str, user: &str, password: &str, dbname: &str) -> Self {
        // Connect to the database.
        let mut cfg = Config::new();
        if user.len() > 0{
            cfg.user = Some(user.to_string());
        }
        if password.len() > 0{
            cfg.password = Some(password.to_string());
        }
        cfg.host = Some(host.to_string());
        cfg.dbname = Some(dbname.to_string());
        cfg.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });
        let pool = cfg.create_pool(NoTls).unwrap();
        Self {pool: pool}
    }
    pub async fn migrate(&self) -> Result<u64, tokio_postgres::Error>{
        let client = self.pool.get().await.unwrap();
        client.execute("CREATE TABLE IF NOT EXISTS users (id UUID PRIMARY KEY, admin BOOLEAN);", &[]).await
    }
}


#[async_trait]
impl UserRepository for PostgresUser {
    async fn get_user(&self,id: Uuid) -> Option<User> {
        let client = self.pool.get().await.unwrap();
        let row = client.query_one("SELECT * FROM users WHERE id = $1;", &[&id]).await.unwrap();
        Some(User {id:row.get::<_, Uuid>(0), admin:row.get::<_, bool>(1)})
    }
    async fn create_user(&self, id: Uuid) -> Result<(), ()> {
        let client = self.pool.get().await.unwrap();
        client.execute("INSERT INTO users (id, admin) VALUES ($1, false);", &[&id]).await.unwrap();
        Ok(())
    }
}
