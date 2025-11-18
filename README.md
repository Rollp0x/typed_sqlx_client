# typed_sqlx_client

[![Crates.io](https://img.shields.io/crates/v/typed_sqlx_client.svg)](https://crates.io/crates/typed_sqlx_client)
[![Documentation](https://docs.rs/typed_sqlx_client/badge.svg)](https://docs.rs/typed_sqlx_client)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/Rollp0x/typed_sqlx_client#license)

A type-safe, extensible Rust library for managing multiple SQL databases and tables with [sqlx](https://github.com/launchbadge/sqlx). Build robust database applications with automatic CRUD operations, flexible queries, and compile-time safety.

## ✨ Key Features

- 🚀 **Automatic CRUD**: `#[derive(CrudOpsRef)]` generates complete CRUD operations
- 🛡️ **Type Safety**: Compile-time prevention of database/table mix-ups
- 🗄️ **Multi-Database**: MySQL, PostgreSQL, and SQLite support with unified API
- 🎛️ **Per-Table Traits**: Each table can implement custom traits independently (not monolithic SQL client)
- 🔍 **Flexible Queries**: Both type-safe and JSON-based SELECT operations
- 🌐 **Framework Ready**: Perfect for actix-web, warp, and other async frameworks
- ⚡ **Zero Runtime Cost**: All type safety is compile-time only

## 🚀 Quick Start

Add to your `Cargo.toml`:
```toml
[dependencies]
typed_sqlx_client = "0.2.2"
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio", "uuid"] }
```

Define your entity and get instant CRUD:
```rust
use typed_sqlx_client::{CrudOpsRef, SqlDB, SelectOnlyQuery};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

#[derive(FromRow, CrudOpsRef, Debug)]
#[crud(table = "users", db = "postgres")]
struct User {
    #[crud(primary_key)]
    id: Option<Uuid>,
    #[crud(rename = "user_name")]  // Maps to 'user_name' column
    name: String,
    email: String,
}

struct MainDB;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // Setup typed pool
    let pool = PgPool::connect("postgres://...").await?;
    let db = SqlDB::from_pool::<MainDB>(pool);
    let user_table = db.get_table::<User>();

    // You can get the table name for custom queries:
    let table_name = user_table.table_name();

    println!("Using table name: {}", table_name);
    // CRUD operations work immediately!
    let user = User { 
        id: Some(Uuid::new_v4()), 
        name: "Alice".to_string(), 
        email: "alice@example.com".to_string() 
    };
    
    user_table.insert(&user).await?;
    let found = user_table.get_by_id(&user.id.unwrap()).await?;
    
    // Type-safe queries
    let users: Vec<User> = user_table
        .execute_select_as_only::<User>(&format!("SELECT * FROM {} WHERE active = true", table_name))
        .await?;
    
    // Dynamic JSON queries  
    let json_data = user_table
        .execute_select_only(&format!("SELECT name, email FROM {}", table_name))
        .await?;
        
    Ok(())
}
```

## 📋 What's New in v0.2.2

### New: Table Name Accessor
- `CrudOpsRef` trait now provides a `table_name(&self) -> &'static str` method, allowing you to retrieve the table name at runtime for custom queries and dynamic SQL generation.
- The derive macro automatically implements this method for all tables, using the value from `#[crud(table = "...")]`.

## 📋 What's New in v0.2.0

### 🎉 Major Improvements
- **Replaced `CrudOps` trait** with powerful `#[derive(CrudOpsRef)]` macro
- **Added `execute_select_as_only<T>()`** for type-safe SELECT queries
- **Enhanced field mapping** with `#[crud(rename = "...")]`
- **Direct trait implementation** on `SqlTable` (no more macros!)
- **Better error messages** and IDE support

### 🔄 Migration from v0.1.x
```rust
// OLD (v0.1.x) - Remove this
impl CrudOps<i64, User> for SqlTable<Postgres, MainDB, User> { /* manual impl */ }

// NEW (v0.2.0) - Just add this!  
#[derive(FromRow, CrudOpsRef)]
#[crud(table = "users", db = "postgres")]
struct User { /* fields */ }
```

## 🗄️ Database Support

| Database   | Status | CRUD Support | Query Support | Example |
|------------|--------|--------------|---------------|---------|
| PostgreSQL | ✅ Stable | ✅ Full | ✅ Both modes | `db = "postgres"` |
| MySQL      | ✅ Stable | ✅ Full | ✅ Both modes | `db = "mysql"` |
| SQLite     | ✅ Stable | ✅ Full | ✅ Both modes | `db = "sqlite"` |

## 📚 Advanced Examples

### Multi-Database Setup
```rust
struct MainDatabase;
struct AnalyticsDatabase;
struct CacheDatabase;

// Type safety prevents mixing databases!
let main_db = SqlDB::from_pool::<MainDatabase>(pg_pool);
let analytics_db = SqlDB::from_pool::<AnalyticsDatabase>(mysql_pool);
let cache_db = SqlDB::from_pool::<CacheDatabase>(sqlite_pool);

let users = main_db.get_table::<User>();          // ✅ 
let events = analytics_db.get_table::<Event>();   // ✅
// let wrong = main_db.get_table::<Event>();      // ❌ Compile error!
```

### Custom Field Mapping
```rust
#[derive(FromRow, CrudOpsRef)]
#[crud(table = "user_profiles", db = "postgres")]
struct UserProfile {
    #[crud(primary_key)]
    id: Option<Uuid>,
    
    #[crud(rename = "full_name")]
    name: String,                    // Rust: name ↔ DB: full_name
    
    #[crud(rename = "email_address")]  
    email: String,                   // Rust: email ↔ DB: email_address
    
    #[crud(rename = "birth_date")]
    birthday: Option<NaiveDate>,     // Rust: birthday ↔ DB: birth_date
    
    is_active: bool,                 // Same name in both
}
```

### Advanced Queries
```rust
// Aggregations with type safety
let stats: Vec<(String, i64, f64)> = user_table
    .execute_select_as_only::<(String, i64, f64)>(
        "SELECT department, COUNT(*), AVG(salary) FROM users GROUP BY department"
    ).await?;

// Custom projections  
let user_emails: Vec<(String,)> = user_table
    .execute_select_as_only::<(String,)>("SELECT email FROM users WHERE active = true")
    .await?;

// Dynamic queries for unknown schemas
let dynamic_data = user_table
    .execute_select_only("SELECT * FROM user_settings WHERE user_id = 123")
    .await?;

for row in dynamic_data {
    if let Some(setting_name) = row.get("setting_name").and_then(|v| v.as_str()) {
        println!("Setting: {}", setting_name);
    }
}
```

### Framework Integration (actix-web)
```rust
use actix_web::{web, App, HttpServer, HttpResponse, Result};

async fn get_user(
    user_table: web::Data<SqlTable<sqlx::Postgres, MainDB, User>>,
    path: web::Path<Uuid>
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    match user_table.get_by_id(&user_id).await {
        Ok(Some(user)) => Ok(HttpResponse::Ok().json(user)),
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = SqlDB::from_pool::<MainDB>(create_pool().await);
    let user_table = db.get_table::<User>();
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(user_table.clone()))
            .route("/users/{id}", web::get().to(get_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## 🏗️ Multi-Table Custom Traits Example

Perfect for scenarios where different tables need different business logic:

```rust
use typed_sqlx_client::{CrudOpsRef, SqlDB, SelectOnlyQuery};
use sqlx::FromRow;

// User table with analytics capabilities
#[derive(FromRow, CrudOpsRef)]
#[crud(table = "users", db = "postgres")]
struct User {
    #[crud(primary_key)]
    id: Option<i64>,
    name: String,
    email: String,
}

// Custom trait for user analytics
trait UserAnalytics {
    async fn get_active_users(&self) -> Result<Vec<User>, sqlx::Error>;
    async fn get_user_engagement_stats(&self) -> Result<EngagementStats, sqlx::Error>;
}

impl UserAnalytics for SqlTable<sqlx::Postgres, MainDB, User> {
    async fn get_active_users(&self) -> Result<Vec<User>, sqlx::Error> {
        self.execute_select_as_only::<User>(
            "SELECT * FROM users WHERE last_login > NOW() - INTERVAL '30 days'"
        ).await
    }
    
    async fn get_user_engagement_stats(&self) -> Result<EngagementStats, sqlx::Error> {
        let stats: Vec<(i64, f64)> = self.execute_select_as_only::<(i64, f64)>(
            "SELECT COUNT(*), AVG(session_duration) FROM user_sessions WHERE created_at > NOW() - INTERVAL '7 days'"
        ).await?;
        // ... process stats
        todo!()
    }
}

// Post table with content moderation
#[derive(FromRow, CrudOpsRef)]
#[crud(table = "posts", db = "postgres")]
struct Post {
    #[crud(primary_key)]
    id: Option<i64>,
    title: String,
    content: String,
    status: String,
}

// Custom trait for content moderation
trait ContentModeration {
    async fn moderate_post(&self, post_id: i64) -> Result<ModerationResult, sqlx::Error>;
    async fn get_flagged_posts(&self) -> Result<Vec<Post>, sqlx::Error>;
}

impl ContentModeration for SqlTable<sqlx::Postgres, MainDB, Post> {
    async fn moderate_post(&self, post_id: i64) -> Result<ModerationResult, sqlx::Error> {
        // Custom moderation logic specific to posts
        todo!()
    }
    
    async fn get_flagged_posts(&self) -> Result<Vec<Post>, sqlx::Error> {
        self.execute_select_as_only::<Post>(
            "SELECT * FROM posts WHERE status = 'flagged' ORDER BY created_at DESC"
        ).await
    }
}

// Usage: Each table has both CRUD + custom capabilities
struct MainDB;

# async fn example() -> Result<(), sqlx::Error> {
let db = SqlDB::from_pool::<MainDB>(pg_pool);

let user_table = db.get_table::<User>();
let post_table = db.get_table::<Post>();

// Standard CRUD (from derive macro)
user_table.insert(&user).await?;
post_table.insert(&post).await?;

// Table-specific operations (custom traits)
let active_users = user_table.get_active_users().await?;
let flagged_posts = post_table.get_flagged_posts().await?;
let user_stats = user_table.get_user_engagement_stats().await?;
let moderation_result = post_table.moderate_post(123).await?;
# Ok(())
# }
```

## 🔧 Attribute Reference

### Struct-level Attributes
```rust
#[crud(table = "table_name")]          // Custom table name
#[crud(db = "postgres|mysql|sqlite")]  // Database type  
#[crud(table = "users", db = "postgres")]  // Combined
```

### Field-level Attributes  
```rust
#[crud(primary_key)]                   // Mark as primary key
#[crud(rename = "column_name")]         // Map to different column name
```

## 📖 Documentation

- [📚 API Documentation](https://docs.rs/typed_sqlx_client)
- [🔧 Migration Guide](CHANGELOG.md#020---2025-07-04)
- [💡 Examples](./examples/)

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## 📄 License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## ⭐ Acknowledgments

- Built on top of the excellent [sqlx](https://github.com/launchbadge/sqlx) crate
- Inspired by the need for type-safe multi-database applications
- Thanks to all contributors and users!
