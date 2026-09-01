# typed_sqlx_client

[![Crates.io](https://img.shields.io/crates/v/typed_sqlx_client.svg)](https://crates.io/crates/typed_sqlx_client)
[![Documentation](https://docs.rs/typed_sqlx_client/badge.svg)](https://docs.rs/typed_sqlx_client)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/Rollp0x/typed_sqlx_client#license)

A type-safe, extensible Rust library for managing multiple SQL databases and tables with [sqlx](https://github.com/launchbadge/sqlx). Build robust database applications with automatic CRUD operations, flexible queries, and compile-time safety.

## ✨ Key Features

- 🚀 **Automatic CRUD**: `#[derive(CrudOpsRef)]` generates full CRUD operations
- 🛡️ **Type Safety**: Compile-time prevention of database/table mix-ups
- 🗄️ **Multi-Database**: MySQL, PostgreSQL, and SQLite with a unified API
- 🔍 **Flexible Queries**: Type-safe & JSON SELECTs with safe parameter binding

## 🚀 Quick Start

```toml
[dependencies]
typed_sqlx_client = "0.2.4"
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio"] }
```

```rust
use typed_sqlx_client::{CrudOpsRef, SqlDB, SelectOnlyQuery};
use sqlx::{PgPool, FromRow};

#[derive(FromRow, CrudOpsRef, Debug)]
#[crud(table = "users", db = "postgres")]
struct User {
    #[crud(primary_key)]
    id: Option<i64>,
    name: String,
    email: String,
}

struct MainDB;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPool::connect("postgres://...").await?;
    let db = SqlDB::from_pool::<MainDB>(pool);
    let users = db.get_table::<User>();

    // CRUD (auto-generated)
    let user = User { id: None, name: "Alice".into(), email: "alice@example.com".into() };
    users.insert(&user).await?;
    let found = users.get_by_id(&1).await?;

    // Table name accessor
    let table_name = users.table_name();

    // Typed queries
    let all: Vec<User> = users.execute_select_as_only::<User>("SELECT * FROM users").await?;

    // Parameterized queries (safe binding)
    let by_name: Vec<User> = users
        .execute_select_as_query(
            sqlx::query_as::<sqlx::Postgres, User>("SELECT * FROM users WHERE name = $1")
                .bind("Alice"),
        )
        .await?;

    Ok(())
}
```

## 🏷️ Attributes

| Attribute | Level | Description |
|-----------|-------|-------------|
| `#[crud(table = "...")]` | struct | Table name (defaults to struct name) |
| `#[crud(db = "...")]` | struct | `mysql` / `postgres` / `sqlite` |
| `#[crud(primary_key)]` | field | Mark primary key (defaults to first field) |
| `#[crud(rename = "...")]` | field | Map field to a column name |
| `#[crud(encode_json)]` | field | Auto `Json()` wrap on insert/update |

## 🔍 Query API

`SelectOnlyQuery` is implemented directly on `SqlTable`. Only `SELECT` statements
are allowed; prefer `.bind()` over `format!` interpolation for safe parameter binding.

| Method | Input | Output |
|--------|-------|--------|
| `execute_select_only(&str)` | raw SQL | `Vec<serde_json::Value>` |
| `execute_select_as_only::<T>(&str)` | raw SQL | `Vec<T>` |
| `execute_select_query(Query)` | `sqlx::query(...).bind(...)` | `Vec<serde_json::Value>` |
| `execute_select_as_query(QueryAs)` | `sqlx::query_as::<DB, T>(...).bind(...)` | `Vec<T>` |

## 🗄️ Multi-Database

```rust
struct MainDB;
struct AnalyticsDB;

let main = SqlDB::from_pool::<MainDB>(pg_pool);
let analytics = SqlDB::from_pool::<AnalyticsDB>(mysql_pool);

let users = main.get_table::<User>();          // ✅
let events = analytics.get_table::<Event>();   // ✅
// let wrong = main.get_table::<Event>();      // ❌ compile error
```

## 📖 Documentation

- [📚 API Documentation](https://docs.rs/typed_sqlx_client)
- [🔧 Migration Guide](CHANGELOG.md#020---2025-07-04)
- [💡 Examples](./examples/)

## 🤝 Contributing

Contributions are welcome! Feel free to open an issue or submit a Pull Request.

## 📄 License

MIT OR Apache-2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
