//! # typed_sqlx_client
//!
//! A type-safe, extensible Rust library for managing multiple SQL databases and tables with sqlx.
//!
//! - Provides generic, type-safe wrappers (`SqlPool`, `SqlTable`) for sqlx connection pools and table handles.
//! - Supports storing multiple pools (for different databases) in frameworks like actix-web.
//! - Enables per-table trait implementations (e.g., `CrudOps`, `CrudOpsRef`) for flexible, type-driven database access.
//! - Designed for projects needing clear separation and type safety across many databases and tables.
//!

pub mod tables;
pub mod traits;

pub use tables::*;
pub use traits::*;


