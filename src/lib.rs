//! # typed_sqlx_client
//!
//! A type-safe, extensible Rust library for managing multiple SQL databases and tables with sqlx.
//!
//! - Provides generic, type-safe wrappers (`SqlPool`, `SqlTable`) for sqlx connection pools and table handles.
//! - Supports storing multiple pools (for different databases) in frameworks like actix-web.
//! - Enables per-table trait implementations (e.g., `CrudOps`, `CrudOpsRef`) for flexible, type-driven database access.
//! - Designed for projects needing clear separation and type safety across many databases and tables.
//!
//! ## Example usage
//!
//! ```rust
//! struct MainDb;
//! struct MyTable;
//! let pool = MySqlPoolOptions::new().connect(&db_url).await?;
//! let typed_pool = SqlPool::from_pool::<MainDb>(pool);
//! let table = typed_pool.get_table::<MyTable>();
//! sqlx::query("SELECT * FROM my_table").fetch_all(table.as_ref()).await?;
//! ```
//!
//! ## Example for trait implementation
//!
//! ```rust
//! use typed_sqlx_client::{CrudOpsRef, SqlTable};
//! use sqlx::MySql;
//! struct MainDb;
//! struct UserEntity { id: i32, name: String }
//! // Implementing CRUD trait for a specific table-entity binding
//! #[async_trait::async_trait]
//! impl CrudOpsRef<i32, UserEntity> for SqlTable<MySql, MainDb, UserEntity> {
//!     type Error = String;
//!     async fn insert(&self, entity: &UserEntity) -> Result<(), Self::Error> { Ok(()) }
//!     // ... other methods ...
//! }
//! ```

pub mod tables;
pub mod traits;

pub use tables::*;
pub use traits::*;
