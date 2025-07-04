use crate::traits::SelectOnlyQuery;
use sqlx::{
    Column, ColumnIndex, Decode, Executor, IntoArguments, Pool, Row, Type, database::Database,
};
use std::marker::PhantomData;
use std::ops::Deref;

/// Type-safe wrapper for a database connection pool.
///
/// `SqlPool` provides compile-time safety and database disambiguation through marker types.
/// This is essential for applications that work with multiple databases, as it prevents
/// accidentally mixing connections or operations between different database instances.
///
/// ## Key Features
/// - **Compile-time safety**: Prevents mixing different database instances
/// - **Zero-cost abstraction**: No runtime overhead over raw sqlx pools
/// - **Clone-friendly**: Cheap cloning suitable for sharing across async tasks
/// - **Framework integration**: Perfect for dependency injection in web frameworks
///
/// ## Type Parameters
/// * `P` - The sqlx database driver type (`sqlx::Postgres`, `sqlx::MySql`, `sqlx::Sqlite`)
/// * `DB` - A marker type to distinguish different database instances at compile time
///
/// ## Marker Types for Database Distinction
/// Use unit structs as marker types to distinguish different databases:
/// ```rust
/// struct MainDatabase;
/// struct AnalyticsDatabase;  
/// struct CacheDatabase;
///
/// let main_pool = SqlPool::from_pool::<MainDatabase>(main_pg_pool);
/// let analytics_pool = SqlPool::from_pool::<AnalyticsDatabase>(analytics_pg_pool);
/// let cache_pool = SqlPool::from_pool::<CacheDatabase>(cache_sqlite_pool);
/// ```
///
/// ## Integration with Web Frameworks
/// ### actix-web
/// ```rust
/// use actix_web::{web, App, HttpServer};
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     let main_pool = SqlPool::from_pool::<MainDatabase>(create_pg_pool().await);
///     let analytics_pool = SqlPool::from_pool::<AnalyticsDatabase>(create_mysql_pool().await);
///     
///     HttpServer::new(move || {
///         App::new()
///             .app_data(web::Data::new(main_pool.clone()))
///             .app_data(web::Data::new(analytics_pool.clone()))
///             .route("/users", web::get().to(get_users))
///     })
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
///
/// async fn get_users(main_db: web::Data<SqlPool<sqlx::Postgres, MainDatabase>>) -> impl Responder {
///     let user_table = main_db.get_table::<User>();
///     // ... use user_table
/// }
/// ```
///
/// ## Thread Safety
/// `SqlPool` is `Send + Sync` and can be safely shared across async tasks and threads.
/// The underlying sqlx pool uses `Arc` internally, making cloning very efficient.
pub struct SqlPool<P: Database, DB>(Pool<P>, PhantomData<DB>);

impl<P: Database, DB> SqlPool<P, DB> {
    /// Get direct access to the underlying sqlx::Pool.
    ///
    /// This method provides access to the raw sqlx pool for advanced operations
    /// that aren't covered by the typed wrappers, such as:
    /// - Custom transaction management
    /// - Raw SQL execution with specific sqlx features
    /// - Pool statistics and health checks
    /// - Direct integration with sqlx query builders
    ///
    /// ## Example
    /// ```rust
    /// # use typed_sqlx_client::SqlPool;
    /// # use sqlx::PgPool;
    /// # struct MainDB;
    /// # let pool = SqlPool::from_pool::<MainDB>(PgPool::connect("postgres://...").await.unwrap());
    /// // Execute raw SQL
    /// let result = sqlx::query("SELECT version()")
    ///     .fetch_one(pool.pool())
    ///     .await?;
    ///
    /// // Get pool statistics
    /// let size = pool.pool().size();
    /// let idle = pool.pool().num_idle();
    /// println!("Pool size: {}, idle: {}", size, idle);
    /// # Ok::<(), sqlx::Error>(())
    /// ```
    pub fn pool(&self) -> &Pool<P> {
        &self.0
    }
}

/// Implement Clone for SqlPool.
///
/// sqlx pools are designed to be cloned cheaply (they use Arc internally),
/// so cloning a SqlPool is efficient and recommended for sharing across async tasks.
impl<P: Database, DB> Clone for SqlPool<P, DB> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<P: Database> SqlPool<P, ()> {
    /// Create a typed SqlPool from a raw sqlx pool.
    ///
    /// This is the primary factory method for creating `SqlPool` instances. The marker
    /// type `DB` provides compile-time distinction between different database instances,
    /// preventing accidental mixing of database operations.
    ///
    /// ## Type Parameters
    /// * `DB` - A marker type (usually a unit struct) to identify this database instance
    ///
    /// ## Design Pattern
    /// The marker type pattern allows multiple database pools to coexist safely:
    /// ```rust
    /// // Define marker types for different databases
    /// struct UserDatabase;
    /// struct LoggingDatabase;
    /// struct CacheDatabase;
    ///
    /// // Create pools with different markers
    /// let user_pool = SqlPool::from_pool::<UserDatabase>(user_pg_pool);
    /// let log_pool = SqlPool::from_pool::<LoggingDatabase>(log_mysql_pool);
    /// let cache_pool = SqlPool::from_pool::<CacheDatabase>(cache_sqlite_pool);
    ///
    /// // Type system prevents mixing:
    /// let user_table = user_pool.get_table::<User>();     // ✓ Correct
    /// let log_table = log_pool.get_table::<LogEntry>();   // ✓ Correct
    /// // let wrong = user_pool.get_table::<LogEntry>();   // ✗ Compile error
    /// ```
    ///
    /// ## Example
    /// ```rust
    /// use sqlx::PgPool;
    /// use typed_sqlx_client::SqlPool;
    ///
    /// struct MainDB;
    ///
    /// let raw_pool = PgPool::connect("postgres://user:pass@localhost/db").await?;
    /// let typed_pool = SqlPool::from_pool::<MainDB>(raw_pool);
    /// # Ok::<(), sqlx::Error>(())
    /// ```
    pub fn from_pool<DB>(pool: Pool<P>) -> SqlPool<P, DB> {
        SqlPool(pool, PhantomData)
    }
}

/// Type-safe handle for database table operations.
///
/// `SqlTable` is the core abstraction that brings together a database connection pool
/// with table-specific type information. This design enables **per-table trait implementations**
/// rather than a monolithic SQL client approach.
///
/// ## 🎯 Design Philosophy: Per-Table Traits vs Monolithic Client
///
/// ### ❌ Traditional Monolithic Approach
/// ```rust,ignore
/// // BAD: One client handles all tables with generic methods
/// struct SqlClient {
///     pool: Pool<Postgres>
/// }
/// impl SqlClient {
///     fn insert_user(&self, user: User) { /* */ }
///     fn insert_post(&self, post: Post) { /* */ }
///     fn insert_comment(&self, comment: Comment) { /* */ }
///     // ... hundreds of methods for different tables
/// }
/// ```
///
/// ### ✅ This Library's Per-Table Trait Approach
/// ```rust
/// // GOOD: Each table type implements only what it needs
/// #[derive(FromRow, CrudOpsRef)]
/// #[crud(table = "users", db = "postgres")]
/// struct User { /* fields */ }
///
/// #[derive(FromRow, CrudOpsRef)]
/// #[crud(table = "posts", db = "postgres")]  
/// struct Post { /* fields */ }
///
/// // Each table can have different trait implementations:
/// impl CustomAnalytics for SqlTable<Postgres, MainDB, User> {
///     fn get_user_metrics(&self) -> impl Future<Output = UserMetrics> { /* */ }
/// }
///
/// impl ContentModeration for SqlTable<Postgres, MainDB, Post> {
///     fn moderate_content(&self) -> impl Future<Output = ModerationResult> { /* */ }
/// }
/// ```
///
/// ## 🚀 Benefits of This Design
///
/// 1. **🎛️ Granular Control**: Each table implements only the operations it needs
/// 2. **🔒 Type Safety**: Impossible to call user operations on post tables
/// 3. **📦 Modularity**: Add/remove table-specific behaviors independently
/// 4. **⚡ Performance**: No runtime overhead for unused operations
/// 5. **🧪 Testability**: Mock and test each table's behavior in isolation
/// 6. **👥 Team Scalability**: Different teams can work on different table logic
///
/// ## 🏗️ Multi-Database, Multi-Table Architecture
///
/// ```rust
/// // Define database markers
/// struct UserDB;      // User service database
/// struct ContentDB;   // Content management database  
/// struct AnalyticsDB; // Analytics database
///
/// // Each database can have multiple tables
/// let user_pool = SqlPool::from_pool::<UserDB>(user_pg_pool);
/// let content_pool = SqlPool::from_pool::<ContentDB>(content_mysql_pool);
/// let analytics_pool = SqlPool::from_pool::<AnalyticsDB>(analytics_clickhouse_pool);
///
/// // Each table gets its own specialized handle
/// let users = user_pool.get_table::<User>();           // User CRUD + custom user logic
/// let profiles = user_pool.get_table::<UserProfile>(); // Profile-specific operations
/// let posts = content_pool.get_table::<Post>();        // Content operations
/// let comments = content_pool.get_table::<Comment>();  // Comment moderation
/// let events = analytics_pool.get_table::<Event>();    // Analytics operations
/// ```
///
/// ## Key Features
/// - **Automatic trait implementations**: CRUD operations are generated via derive macros
/// - **Type safety**: Compile-time prevention of table/database mismatches
/// - **Flexible querying**: Both type-safe and JSON-based query methods
/// - **Framework integration**: Designed for dependency injection patterns
///
/// ## Usage Patterns
///
/// ### 1. Basic CRUD Operations
/// ```rust
/// use typed_sqlx_client::{CrudOpsRef, SqlPool};
/// use sqlx::FromRow;
///
/// #[derive(FromRow, CrudOpsRef)]
/// #[crud(table = "users", db = "postgres")]
/// struct User {
///     #[crud(primary_key)]
///     id: Option<i64>,
///     name: String,
///     email: String,
/// }
///
/// struct MainDB;
///
/// # async fn example(pool: SqlPool<sqlx::Postgres, MainDB>) -> Result<(), sqlx::Error> {
/// let user_table = pool.get_table::<User>();
///
/// // CRUD operations
/// let user = User { id: None, name: "Alice".to_string(), email: "alice@example.com".to_string() };
/// user_table.insert(&user).await?;
///
/// let found_user = user_table.get_by_id(&1).await?;
/// # Ok(())
/// # }
/// ```
///
/// ### 2. Advanced Queries
/// ```rust
/// # use typed_sqlx_client::{CrudOpsRef, SqlPool, SelectOnlyQuery};
/// # use sqlx::FromRow;
/// # #[derive(FromRow, CrudOpsRef)]
/// # #[crud(table = "users", db = "postgres")]
/// # struct User { id: Option<i64>, name: String, email: String }
/// # struct MainDB;
/// # async fn example(user_table: typed_sqlx_client::SqlTable<sqlx::Postgres, MainDB, User>) -> Result<(), sqlx::Error> {
/// // Type-safe custom queries
/// let active_users: Vec<User> = user_table
///     .execute_select_as_only::<User>("SELECT * FROM users WHERE active = true")
///     .await?;
///
/// // Aggregation queries
/// let counts: Vec<(i64,)> = user_table
///     .execute_select_as_only::<(i64,)>("SELECT COUNT(*) FROM users")
///     .await?;
///
/// // Dynamic JSON queries
/// let json_data = user_table
///     .execute_select_only("SELECT name, email FROM users WHERE created_at > NOW() - INTERVAL '7 days'")
///     .await?;
/// # Ok(())
/// # }
/// ```
///
/// ### 3. Integration with Web Frameworks
/// ```rust
/// use actix_web::{web, HttpResponse, Result};
///
/// async fn get_user(
///     user_table: web::Data<SqlTable<sqlx::Postgres, MainDB, User>>,
///     path: web::Path<i64>
/// ) -> Result<HttpResponse> {
///     let user_id = path.into_inner();
///     match user_table.get_by_id(&user_id).await {
///         Ok(Some(user)) => Ok(HttpResponse::Ok().json(user)),
///         Ok(None) => Ok(HttpResponse::NotFound().finish()),
///         Err(_) => Ok(HttpResponse::InternalServerError().finish()),
///     }
/// }
/// ```
///
/// ## Thread Safety
/// `SqlTable` is `Send + Sync` and can be safely shared across async tasks and threads.
/// It's designed to be cloned efficiently for use in web handlers and async contexts.
#[derive(Clone)]
pub struct SqlTable<P: Database, DB, Table>(SqlPool<P, DB>, PhantomData<Table>);

impl<P: Database, DB> SqlPool<P, DB> {
    /// Create a typed table handle for a specific entity.
    ///
    /// This method creates a `SqlTable` instance that provides type-safe access to
    /// database operations for a specific table/entity. The resulting handle can be
    /// used for CRUD operations, queries, and other table-specific operations.
    ///
    /// ## Type Parameters
    /// * `Table` - The entity struct representing the table (must implement required traits)
    ///
    /// ## Usage Patterns
    ///
    /// ### Single Table Access
    /// ```rust
    /// # use typed_sqlx_client::SqlPool;
    /// # use sqlx::FromRow;
    /// # #[derive(FromRow)] struct User { id: i64, name: String }
    /// # struct MainDB;
    /// # let pool: SqlPool<sqlx::Postgres, MainDB> = todo!();
    /// let user_table = pool.get_table::<User>();
    /// ```
    ///
    /// ### Multiple Tables from Same Database
    /// ```rust
    /// # use typed_sqlx_client::SqlPool;
    /// # use sqlx::FromRow;
    /// # #[derive(FromRow)] struct User { id: i64, name: String }
    /// # #[derive(FromRow)] struct Post { id: i64, title: String }
    /// # #[derive(FromRow)] struct Comment { id: i64, content: String }
    /// # struct MainDB;
    /// # let pool: SqlPool<sqlx::Postgres, MainDB> = todo!();
    /// let user_table = pool.get_table::<User>();
    /// let post_table = pool.get_table::<Post>();
    /// let comment_table = pool.get_table::<Comment>();
    /// ```
    ///
    /// ### Dependency Injection Pattern
    /// ```rust
    /// use std::sync::Arc;
    ///
    /// struct AppState {
    ///     user_table: Arc<SqlTable<sqlx::Postgres, MainDB, User>>,
    ///     post_table: Arc<SqlTable<sqlx::Postgres, MainDB, Post>>,
    /// }
    ///
    /// impl AppState {
    ///     fn new(pool: SqlPool<sqlx::Postgres, MainDB>) -> Self {
    ///         Self {
    ///             user_table: Arc::new(pool.get_table::<User>()),
    ///             post_table: Arc::new(pool.get_table::<Post>()),
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// ## Performance Notes
    /// - This method is very lightweight (just creates a new struct with PhantomData)
    /// - The resulting `SqlTable` can be cloned efficiently
    /// - Multiple table handles can share the same underlying pool safely
    pub fn get_table<Table>(&self) -> SqlTable<P, DB, Table> {
        SqlTable(self.clone(), PhantomData)
    }
}

impl<P: Database, DB, Table> SqlTable<P, DB, Table> {
    /// Get direct access to the underlying sqlx::Pool.
    ///
    /// This method provides access to the raw sqlx pool for advanced database operations
    /// that may not be covered by the typed interface, such as:
    /// - Complex transaction management
    /// - Database administration commands
    /// - Custom query builders or raw SQL
    /// - Pool monitoring and statistics
    ///
    /// ## Example
    /// ```rust
    /// # use typed_sqlx_client::SqlTable;
    /// # let user_table: SqlTable<sqlx::Postgres, (), ()> = todo!();
    /// // Start a transaction
    /// let mut tx = user_table.get_pool().begin().await?;
    ///
    /// // Execute multiple operations in transaction
    /// sqlx::query("INSERT INTO users (name) VALUES ($1)")
    ///     .bind("Alice")
    ///     .execute(&mut *tx)
    ///     .await?;
    ///
    /// sqlx::query("INSERT INTO profiles (user_id, bio) VALUES ($1, $2)")
    ///     .bind(1)
    ///     .bind("Software Developer")
    ///     .execute(&mut *tx)
    ///     .await?;
    ///
    /// tx.commit().await?;
    /// # Ok::<(), sqlx::Error>(())
    /// ```
    pub fn get_pool(&self) -> &Pool<P> {
        self.0.pool()
    }
}

/// Allow passing SqlTable as `&Pool<P>` to sqlx queries
impl<P: Database, DB, Table> AsRef<Pool<P>> for SqlTable<P, DB, Table> {
    fn as_ref(&self) -> &Pool<P> {
        self.get_pool()
    }
}

/// Allow using `&SqlTable` as `&Pool<P>` directly
impl<P: Database, DB, Table> Deref for SqlTable<P, DB, Table> {
    type Target = Pool<P>;

    fn deref(&self) -> &Self::Target {
        self.get_pool()
    }
}

impl<P: Database, DB, Table> SelectOnlyQuery<P> for SqlTable<P, DB, Table>
where
    DB: Sync + Send,
    Table: Sync + Send,
    P::Row: Row<Database = P>,
    P::Column: Column<Database = P>,
    for<'r> &'r Pool<P>: Executor<'r, Database = P>,
    for<'r> &'r str: ColumnIndex<P::Row>,
    for<'r> i64: Type<P> + Decode<'r, P>,
    for<'r> f64: Type<P> + Decode<'r, P>,
    for<'r> i32: Type<P> + Decode<'r, P>,
    for<'r> bool: Type<P> + Decode<'r, P>,
    for<'r> String: Type<P> + Decode<'r, P>,
    for<'q> P::Arguments<'q>: IntoArguments<'q, P>,
    for<'r> Vec<u8>: Type<P> + Decode<'r, P>,
{
    type MError = sqlx::Error;
    type Output = Vec<serde_json::Value>;

    async fn execute_select_only(&self, query: &str) -> Result<Self::Output, Self::MError> {
        let trimmed_query = query.trim().to_lowercase();
        if !trimmed_query.starts_with("select") {
            return Err(sqlx::Error::InvalidArgument(
                "Only SELECT queries are allowed".into(),
            ));
        }
        let pool = self.get_pool();
        let rows = sqlx::query(query).fetch_all(pool).await?;
        let columns = if let Some(row) = rows.first() {
            row.columns()
                .iter()
                .map(|c| c.name().to_string())
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        let mut result = Vec::new();
        for row in rows {
            let mut json_row = serde_json::Map::new();
            for column in &columns {
                let json_value = if let Ok(v) = row.try_get::<i64, _>(column.as_str()) {
                    serde_json::json!(v)
                } else if let Ok(v) = row.try_get::<f64, _>(column.as_str()) {
                    serde_json::json!(v)
                } else if let Ok(v) = row.try_get::<bool, _>(column.as_str()) {
                    serde_json::json!(v)
                } else if let Ok(s) = row.try_get::<String, _>(column.as_str()) {
                    serde_json::from_str(&s).unwrap_or(serde_json::json!(s))
                } else if let Ok(v) = row.try_get::<Vec<u8>, _>(column.as_str()) {
                    serde_json::json!(v)
                } else if let Ok(v) = row.try_get::<i32, _>(column.as_str()) {
                    serde_json::json!(v)
                } else {
                    serde_json::json!(null)
                };
                json_row.insert(column.clone(), json_value);
            }
            result.push(serde_json::Value::Object(json_row));
        }
        Ok(result)
    }

    async fn execute_select_as_only<T>(&self, query: &str) -> Result<Vec<T>, Self::MError>
    where
        T: for<'r> sqlx::FromRow<'r, <P as sqlx::Database>::Row> + Send + Unpin + 'static,
    {
        let trimmed_query = query.trim().to_lowercase();
        if !trimmed_query.starts_with("select") {
            return Err(sqlx::Error::InvalidArgument(
                "Only SELECT queries are allowed".into(),
            ));
        }
        let pool = self.get_pool();
        let values: Vec<T> = sqlx::query_as(query).fetch_all(pool).await?;
        Ok(values)
    }
}
