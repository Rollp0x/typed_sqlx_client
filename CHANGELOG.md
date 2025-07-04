# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2025-07-03
### Added
- Feature: `CrudOpsRef` derive macro for automatic CRUD implementation on entity structs.
- Feature: Automatic detection of primary key via `#[crud(primary_key)]` or fallback to first field.
- Feature: All field types are automatically checked for required `sqlx` traits.
- Feature: Batch insert (`insert_batch`) support.

### Added
- `execute_select_as_only<T>` to `SelectOnlyQuery` trait for type-safe dynamic SELECT queries.

### Removed
- Breaking: The old `CrudOps` trait is removed; only `CrudOpsRef` is needed for reference-based CRUD.

## [0.1.1] - 2025-07-02
### Added
- Feature: Macro `select_only_query!` for all `SqlTable` types, supporting MySQL/Postgres/Sqlite with type-preserving JSON output.
- Type preservation: int/float/bool columns are native JSON types; unsupported types are null.
- Feature-gated `serde_json` dependency for minimal builds.
- Improved documentation and usage examples.

### Changed
- Internal: code and doc formatting, macro path compatibility, and trait bound refinements.

## [0.1.0] - 2025-07-01
### Added
- Initial release.
- Type-safe wrappers for sqlx pools and tables (`SqlPool`, `SqlTable`).
- Per-table trait support for CRUD operations (`CrudOps`, `CrudOpsRef`).
- Example integration with actix-web and multiple databases.
- Example for implementing CRUD trait for a table-entity binding.
