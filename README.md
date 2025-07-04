# typed_sqlx_client

A type-safe, extensible Rust library for managing multiple SQL databases and tables with [sqlx](https://github.com/launchbadge/sqlx).

- Provides generic, type-safe wrappers (`SqlPool`, `SqlTable`) for sqlx connection pools and table handles.
- Supports storing multiple pools (for different databases) in frameworks like actix-web.
- **Automatic CRUD implementation:** Use the `#[derive(CrudOpsRef)]` macro to generate full CRUD for your entity structs.
- Enables per-table trait implementations (e.g., `CrudOpsRef`, `SelectOnlyQuery`) for flexible, type-driven database access.
- Designed for projects needing clear separation and type safety across many databases and tables.

## Version
Current: **0.2.0**

## What's New in 0.2.0

- **`CrudOpsRef` derive macro**: Now supports field renaming, custom table name and database type, and primary key selection. Works with MySQL, PostgreSQL, and SQLite.
- **Type-safe batch insert**: `insert_batch` is available for all supported databases.
- **Flexible SELECT queries**: The `SelectOnlyQuery` trait now provides `execute_select_as_only<T>`, allowing you to fetch results as any struct or tuple implementing `sqlx::FromRow` for type-safe querying.
- **Improved documentation and examples**.

## Features
- Type-safe pool and table wrappers for sqlx
- Easy integration with actix-web and other frameworks
- Per-table trait implementations for CRUD and custom operations
- Supports multiple databases and tables in a single project
- Flexible read-only SELECT queries:
    - `execute_select_only`: Returns results as type-preserving JSON (`serde_json::Value`), suitable for dynamic or unknown schemas.
    - `execute_select_as_only<T>`: Returns results as any struct or tuple implementing `sqlx::FromRow`, for type-safe querying.
- Automatic CRUD derive macro for entity structs

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

## Example: Automatic CRUD Derive Macro
```rust
use typed_sqlx_client::{CrudOpsRef, SqlTable};
use sqlx::FromRow;

#[derive(CrudOpsRef, FromRow)]
#[crud(table = "users")]
struct UserEntity {
    #[crud(primary_key)]
    id: i32,
    name: String,
    email: String,
}
```

## Example: Read-only SELECT Query Macro
```rust
// Enable the feature for your database, e.g. mysql
// In Cargo.toml: features = ["mysql"]
use typed_sqlx_client::select_only_query;
```







MIT OR Apache-2.0## License- Unsupported or unrecognized column types will be `null` in the JSON result.- Text/JSON columns are parsed as JSON if possible, otherwise as strings.- Integer, float, and boolean columns are preserved as native JSON types in the result.## Type Preservation``````

## Type Preservation
- Integer, float, and boolean columns are preserved as native JSON types in the result.
- Text/JSON columns are parsed as JSON if possible, otherwise as strings.
- Unsupported or unrecognized column types will be `null` in the JSON result.

## License
MIT OR Apache-2.0
