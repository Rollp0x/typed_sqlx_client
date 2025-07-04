# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-07-04

This is a major release with significant API improvements and breaking changes.

### 🚀 Added
- **New `#[derive(CrudOpsRef)]` macro**: Automatic CRUD implementation for structs implementing `sqlx::FromRow`
  - Support for all three databases: MySQL, PostgreSQL, and SQLite
  - Field renaming with `#[crud(rename = "column_name")]`
  - Custom table names with `#[crud(table = "table_name")]`
  - Database-specific implementations with `#[crud(db = "postgres|mysql|sqlite")]`
  - Primary key auto-detection with `#[crud(primary_key)]` attribute
  - Intelligent `Option<T>` unwrapping for primary key types
- **Enhanced `SelectOnlyQuery` trait**:
  - New `execute_select_as_only<T>()` method for type-safe result deserialization
  - Support for custom projections, aggregations, and complex queries
  - Direct implementation on `SqlTable` (no macro generation required)
  - Better error handling and type safety
- **Comprehensive documentation**:
  - Detailed crate-level documentation with migration guide
  - Extensive examples for all supported use cases
  - API reference with usage patterns and best practices
  - Troubleshooting guide and common error solutions

### 🔧 Changed
- **Direct trait implementation**: `SelectOnlyQuery` is now implemented directly on `SqlTable` instead of being macro-generated
- **Improved error messages**: Better compile-time error reporting and IDE support
- **Enhanced type safety**: Stronger type checking for database operations
- **Better performance**: More efficient query generation and execution

### ❌ Breaking Changes
- **REMOVED**: `CrudOps` trait - Replace with `#[derive(CrudOpsRef)]` macro
- **CHANGED**: `SelectOnlyQuery` implementation approach (users generally won't notice this change)
- **MIGRATION GUIDE**: See updated README.md and documentation for migration instructions

### 📚 Documentation
- Complete rewrite of crate-level documentation
- Added comprehensive examples for all databases
- Enhanced API documentation with usage patterns
- Added troubleshooting section and common patterns
- Updated README with new features and migration guide

### 🧪 Examples
- Updated all examples to use new `CrudOpsRef` derive macro
- Added examples showing field renaming and custom table mapping
- Enhanced multi-database example with better type safety
- Added examples for both JSON and type-safe query patterns

## [0.1.1] - 2025-07-02

### Added
- Feature: Macro `select_only_query!` for all `SqlTable` types, supporting MySQL/Postgres/Sqlite with type-preserving JSON output
- Type preservation: int/float/bool columns are native JSON types; unsupported types are null
- Feature-gated `serde_json` dependency for minimal builds
- Improved documentation and usage examples

### Changed
- Internal: code and doc formatting, macro path compatibility, and trait bound refinements

## [0.1.0] - 2025-07-01

### Added
- Initial release
- Type-safe wrappers for sqlx pools and tables (`SqlPool`, `SqlTable`)
- Per-table trait support for CRUD operations (`CrudOps`, `CrudOpsRef`)
- Example integration with actix-web and multiple databases
- Example for implementing CRUD trait for a table-entity binding
