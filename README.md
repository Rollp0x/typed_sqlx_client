# typed_sqlx_client

A type-safe, extensible Rust library for managing multiple SQL databases and tables with [sqlx](https://github.com/launchbadge/sqlx).

- Provides generic, type-safe wrappers (`SqlPool`, `SqlTable`) for sqlx connection pools and table handles.
- Supports storing multiple pools (for different databases) in frameworks like actix-web.
- Enables per-table trait implementations (e.g., `CrudOps`, `CrudOpsRef`) for flexible, type-driven database access.
- Designed for projects needing clear separation and type safety across many databases and tables.

## Features
- Type-safe pool and table wrappers for sqlx
- Easy integration with actix-web and other frameworks
- Per-table trait implementations for CRUD and custom operations
- Supports multiple databases and tables in a single project

## Example Usage
```rust
use typed_sqlx_client::{SqlPool, SqlTable};
use sqlx::mysql::MySqlPoolOptions;

struct MainDb;
struct UserEntity { id: i32, name: String }

#[tokio::main]
async fn main() {
    let pool = MySqlPoolOptions::new().connect("mysql://user:pass@localhost/db").await.unwrap();
    let typed_pool = SqlPool::from_pool::<MainDb>(pool);
    let table = typed_pool.get_table::<UserEntity>();
    let _ = sqlx::query("SELECT * FROM user_infos").fetch_all(table.as_ref()).await;
}
```

## Example: Implementing CRUD Trait for a Table-Entity Binding
```rust
use typed_sqlx_client::{CrudOpsRef, SqlTable};
use sqlx::MySql;
struct MainDb;
struct UserEntity { id: i32, name: String }

#[async_trait::async_trait]
impl CrudOpsRef<i32, UserEntity> for SqlTable<MySql, MainDb, UserEntity> {
    type Error = String;
    async fn insert(&self, entity: &UserEntity) -> Result<(), Self::Error> { Ok(()) }
    // ... other methods ...
}
```

## License
MIT OR Apache-2.0
