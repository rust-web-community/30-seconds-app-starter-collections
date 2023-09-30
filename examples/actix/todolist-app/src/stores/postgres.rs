
use deadpool_postgres::*;
use tokio_postgres::NoTls;
use coi::{Provide, Inject};
use crate::store_interface::TodoRepository;
use crate::schemas::{Todo, TodoUpdateRequest};
use async_trait::async_trait;

#[derive(Inject)]
pub struct PostgresTodo {
    pub pool: deadpool_postgres::Pool
}

impl PostgresTodo {
    pub fn new(pool: deadpool_postgres::Pool) -> Self {
        Self { pool: pool }
    }
}


#[derive(Provide)]
#[coi(provides pub dyn TodoRepository with PostgresTodo::new(self.pool.clone()))]
pub struct TodoPostgresProvider {
    pub pool: deadpool_postgres::Pool
}

impl TodoPostgresProvider
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
        client.execute("CREATE TABLE IF NOT EXISTS todo (id INT PRIMARY KEY, value TEXT, checked BOOLEAN);", &[]).await
    }
}


#[async_trait]
impl TodoRepository for PostgresTodo {
    async fn read_all(&self) -> Vec<Todo> {
        let client = self.pool.get().await.unwrap();
        let rows = client.query("SELECT * FROM todo;", &[]).await.unwrap();
        let mut todos: Vec<Todo> = Vec::new();
        for row in rows{
            todos.push(Todo{id: i64::from(row.get::<_, i32>(0)), value: row.get(1), checked: row.get(2)});
        }
        todos
    }

    async fn read_one(&self, id: i64) -> Result<Todo, ()> {
        let client = self.pool.get().await.unwrap();
        let result = client.query_one("SELECT * FROM todo WHERE id = $1;", &[&(id as i32)]).await;
        match result {
            Ok(row) => Ok(Todo{id: i64::from(row.get::<_, i32>(0)), value: row.get(1), checked: row.get(2)}),
            Err(_err) => Err(())
        }
    }

    async fn create_one(&self, t: &Todo) -> Result<(), Todo> {
        let client = self.pool.get().await.unwrap();
        let result = client.query_one("INSERT INTO todo VALUES ($1, $2, $3) ON CONFLICT DO NOTHING RETURNING id;", &[&(t.id as i32), &t.value, &t.checked]).await;
        let row = result.unwrap();
        let ret_id = row.get::<_, Option<i32>>(&0);
        if ret_id.is_none(){
            let todo = self.read_one(t.id.clone()).await.unwrap();
            return Err(todo)
        }
        Ok(())
    }
    async fn update_one(&self, id: i64, todo_update: TodoUpdateRequest) -> Result<Todo, ()> {
        let client = self.pool.get().await.unwrap();
        let result = client.query_one("UPDATE todo SET value=COALESCE($1, value), checked=COALESCE($2, checked) FROM todo WHERE id=$3 RETURNING *;",
         &[&todo_update.value, &todo_update.checked, &id]).await;
        match result {
            Ok(row) => Ok(Todo{id: i64::from(row.get::<_, i32>(0)), value: row.get(1), checked: row.get(2)}),
            Err(_err) => Err(())
        }
    }

    async fn delete_one(&self, id: i64) -> Result<(), ()> {
        let client = self.pool.get().await.unwrap();
        let result = client.execute("DELETE FROM todo WHERE id=$1;",&[&(id as i32)]).await;
        match result {
            Ok(_status) => Ok(()),
            Err(_err) => Err(())
        }
    }

    async fn read_filter(&self, search_text: &str) -> Vec<Todo>  {
        let like_search_text = format!("%{}%", search_text);
        let client = self.pool.get().await.unwrap();
        let rows = client.query("SELECT * FROM todo WHERE value LIKE $1;", &[&like_search_text]).await.unwrap();
        let mut todos: Vec<Todo> = Vec::new();
        for row in rows{
            todos.push(Todo{id: i64::from(row.get::<_, i32>(0)), value: row.get(1), checked: row.get(2)});
        }
        todos    
    }
}
