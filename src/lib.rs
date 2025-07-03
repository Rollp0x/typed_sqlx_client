//! # typed_sqlx_client
//!
//! A type-safe, extensible Rust library for managing multiple SQL databases and tables with sqlx.
//!
//! - Provides generic, type-safe wrappers (`SqlPool`, `SqlTable`) for sqlx connection pools and table handles.
//! - Supports storing multiple pools (for different databases) in frameworks like actix-web.
//! - Enables per-table trait implementations (e.g., `CrudOps`, `CrudOpsRef`) for flexible, type-driven database access.
//! - Designed for projects needing clear separation and type safety across many databases and tables.
//!
//! ## Derive Macro Support
//!
//! The `CrudOpsRef` derive macro allows you to quickly implement CRUD traits for your table structs.
//!
//! **Limitations:**  
//! `CrudOpsRef` currently only supports MySQL and SQLite.  
//! **Postgres is not supported** due to differences in SQL parameter placeholder syntax.
//!
//! ## Example
//!
//! ```rust
//! #[derive(sqlx::FromRow, CrudOpsRef)]
//! #[crud(table = "users")]
//! struct User {
//!     #[crud(primary_key)]
//!     id: i64,
//!     name: String,
//!     email: String,
//! }
//! ```

pub mod tables;
pub mod traits;

pub use tables::*;
pub use traits::*;

// Re-export the CrudOpsRef derive macro
pub use typed_sqlx_client_macros::CrudOpsRef;