use async_trait::async_trait;
use sqlx::{MySql, mysql::MySqlPoolOptions};
use typed_sqlx_client::{CrudOpsRef, SqlPool, SqlTable};

// Marker types for DB and tables
struct MainDb;

// Example entity types
#[derive(Debug, Clone)]
pub struct UserEntity {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct LogEntity {
    pub id: i32,
    pub message: String,
}

// Implementing the CrudOpsRef trait for UserEntity and LogEntity
#[async_trait]
impl CrudOpsRef<i32, UserEntity> for SqlTable<MySql, MainDb, UserEntity> {
    type Error = String;

    async fn insert(&self, entity: &UserEntity) -> Result<(), Self::Error> {
        println!("insert user: {:?}", entity);
        Ok(())
    }
    async fn insert_batch(&self, entities: &[UserEntity]) -> Result<(), Self::Error> {
        println!("insert_batch users: {:?}", entities);
        Ok(())
    }
    async fn get_by_id(&self, id: &i32) -> Result<Option<UserEntity>, Self::Error> {
        println!("get_by_id user: {}", id);
        Ok(Some(UserEntity {
            id: *id,
            name: "demo".to_string(),
        }))
    }
    async fn update_by_id(&self, id: &i32, entity: &UserEntity) -> Result<(), Self::Error> {
        println!("update_by_id user: {} -> {:?}", id, entity);
        Ok(())
    }
    async fn delete_by_id(&self, id: &i32) -> Result<(), Self::Error> {
        println!("delete_by_id user: {}", id);
        Ok(())
    }
}

#[async_trait]
impl CrudOpsRef<i32, LogEntity> for SqlTable<MySql, MainDb, LogEntity> {
    type Error = String;

    async fn insert(&self, entity: &LogEntity) -> Result<(), Self::Error> {
        println!("insert log: {:?}", entity);
        Ok(())
    }
    async fn insert_batch(&self, entities: &[LogEntity]) -> Result<(), Self::Error> {
        println!("insert_batch logs: {:?}", entities);
        Ok(())
    }
    async fn get_by_id(&self, id: &i32) -> Result<Option<LogEntity>, Self::Error> {
        println!("get_by_id log: {}", id);
        Ok(Some(LogEntity {
            id: *id,
            message: "log demo".to_string(),
        }))
    }
    async fn update_by_id(&self, id: &i32, entity: &LogEntity) -> Result<(), Self::Error> {
        println!("update_by_id log: {} -> {:?}", id, entity);
        Ok(())
    }
    async fn delete_by_id(&self, id: &i32) -> Result<(), Self::Error> {
        println!("delete_by_id log: {}", id);
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let main_pool = MySqlPoolOptions::new()
        .connect("mysql://user:pass@localhost/main_db")
        .await
        .unwrap();
    let main_db = SqlPool::from_pool::<MainDb>(main_pool);
    let user_table = main_db.get_table::<UserEntity>();
    let log_table = main_db.get_table::<LogEntity>();

    // calling trait methods
    let _ = user_table
        .insert(&UserEntity {
            id: 1,
            name: "Alice".to_string(),
        })
        .await;
    let _ = log_table
        .insert(&LogEntity {
            id: 1,
            message: "Hello".to_string(),
        })
        .await;
}
