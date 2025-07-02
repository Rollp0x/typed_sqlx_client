# typed_sqlx_client

A type-safe, extensible Rust library for managing multiple SQL databases and tables with [sqlx](https://github.com/launchbadge/sqlx).

- Provides generic, type-safe wrappers (`SqlPool`, `SqlTable`) for sqlx connection pools and table handles.
- Supports storing multiple pools (for different databases) in frameworks like actix-web.
- Enables per-table trait implementations (e.g., `CrudOps`, `CrudOpsRef`, `SelectOnlyQuery`) for flexible, type-driven database access.
- Designed for projects needing clear separation and type safety across many databases and tables.

## Version
Current: **0.1.1**

## Features
- Type-safe pool and table wrappers for sqlx
- Easy integration with actix-web and other frameworks
- Per-table trait implementations for CRUD and custom operations
- Supports multiple databases and tables in a single project
- Macro for read-only SELECT queries with type-preserving JSON output

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

## Example: Read-only SELECT Query Macro
```rust
// Enable the feature for your database, e.g. mysql
// In Cargo.toml: features = ["mysql"]
use typed_sqlx_client::select_only_query;

// The macro is automatically invoked for SqlTable<MySql, .., ..> if the feature is enabled.
// Usage for custom database types:
// select_only_query!(sqlx::mysql::MySql);

// Query results preserve int/float/bool types in JSON; unsupported types are null.
```

## Type Preservation
- Integer, float, and boolean columns are preserved as native JSON types in the result.
- Text/JSON columns are parsed as JSON if possible, otherwise as strings.
- Unsupported or unrecognized column types will be `null` in the JSON result.

## License
MIT OR Apache-2.0
