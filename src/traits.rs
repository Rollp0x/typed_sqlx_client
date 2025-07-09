use std::future::Future;

/// Trait for reference-based async CRUD operations on database entities.
///
/// This trait provides standard Create, Read, Update, Delete operations for database entities.
/// All operations use references to avoid unnecessary copying of large entities, making it
/// memory-efficient and performant.
///
/// ## Key Features
/// - **Reference-based**: All operations take `&Entity` to avoid unnecessary clones
/// - **Async by design**: All methods return `Future`s compatible with async/await
/// - **Error handling**: Generic error type for database-specific error handling
/// - **Batch support**: Optimized batch insert operations
///
/// ## Type Parameters
/// * `ID` - The type of the primary key (e.g., `i64`, `Uuid`, `String`)
/// * `Entity` - The entity/struct type representing a database row
///
/// ## Implementation
/// This trait is automatically implemented when you use the `#[derive(CrudOpsRef)]` macro
/// on structs that implement `sqlx::FromRow`. The macro generates database-specific
/// implementations for MySQL, PostgreSQL, and SQLite.
///
/// ## Example Usage
/// ```rust
/// use typed_sqlx_client::{CrudOpsRef, SqlDB};
/// use sqlx::FromRow;
/// use uuid::Uuid;
///
/// #[derive(FromRow, CrudOpsRef, Debug)]
/// #[crud(table = "users", db = "postgres")]
/// struct User {
///     #[crud(primary_key)]
///     id: Option<Uuid>,
///     name: String,
///     email: String,
/// }
///
/// struct MainDB;
///
/// // Usage in async context
/// # async fn example(user_table: typed_sqlx_client::SqlTable<sqlx::Postgres, MainDB, User>) -> Result<(), sqlx::Error> {
/// let user = User {
///     id: Some(Uuid::new_v4()),
///     name: "Alice".to_string(),
///     email: "alice@example.com".to_string()
/// };
///
/// // Create
/// user_table.insert(&user).await?;
///
/// // Read
/// let found_user = user_table.get_by_id(&user.id.unwrap()).await?;
///
/// // Update
/// if let Some(mut existing_user) = found_user {
///     existing_user.email = "new_email@example.com".to_string();
///     user_table.update_by_id(&user.id.unwrap(), &existing_user).await?;
/// }
///
/// // Delete
/// user_table.delete_by_id(&user.id.unwrap()).await?;
///
/// // Batch operations
/// let users = vec![user; 5];
/// user_table.insert_batch(&users).await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Error Handling
/// All operations return a `Result` with the associated `Error` type, typically `sqlx::Error`.
/// Handle database errors appropriately in your application:
///
/// ```rust
/// # use typed_sqlx_client::CrudOpsRef;
/// # async fn example(user_table: impl CrudOpsRef<uuid::Uuid, User, Error = sqlx::Error>, user: &User, id: &uuid::Uuid) {
/// match user_table.insert(user).await {
///     Ok(()) => println!("User inserted successfully"),
///     Err(sqlx::Error::Database(db_err)) if db_err.constraint() == Some("users_email_key") => {
///         eprintln!("Email already exists");
///     }
///     Err(e) => eprintln!("Database error: {}", e),
/// }
/// # }
/// # struct User;
/// ```
pub trait CrudOpsRef<ID, Entity> {
    /// The error type for operations
    type Error;

    /// Insert a single entity into the database.
    ///
    /// This method adds a new record to the database table. If the entity has an
    /// auto-incrementing primary key, the database will assign the ID automatically.
    ///
    /// ## Arguments
    /// * `entity` - A reference to the entity to insert
    ///
    /// ## Returns
    /// * `Ok(())` if the insert was successful
    /// * `Err(Self::Error)` if the insert failed (e.g., constraint violations, connection issues)
    ///
    /// ## Example
    /// ```rust
    /// # use typed_sqlx_client::CrudOpsRef;
    /// # async fn example(table: impl CrudOpsRef<i64, User, Error = sqlx::Error>) -> Result<(), sqlx::Error> {
    /// let user = User { id: None, name: "Alice".to_string(), email: "alice@example.com".to_string() };
    /// table.insert(&user).await?;
    /// # Ok(())
    /// # }
    /// # struct User { id: Option<i64>, name: String, email: String }
    /// ```
    fn insert(&self, entity: &Entity) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Insert multiple entities into the database in a batch operation.
    ///
    /// This method performs batch insertion of multiple entities. The implementation
    /// may optimize the operation by using transactions or bulk insert statements.
    /// All inserts are performed atomically - if any insert fails, the entire
    /// batch operation fails.
    ///
    /// ## Arguments
    /// * `entities` - A slice of entities to insert
    ///
    /// ## Returns
    /// * `Ok(())` if all inserts were successful
    /// * `Err(Self::Error)` if any insert failed
    ///
    /// ## Example
    /// ```rust
    /// # use typed_sqlx_client::CrudOpsRef;
    /// # async fn example(table: impl CrudOpsRef<i64, User, Error = sqlx::Error>) -> Result<(), sqlx::Error> {
    /// let users = vec![
    ///     User { id: None, name: "Alice".to_string(), email: "alice@example.com".to_string() },
    ///     User { id: None, name: "Bob".to_string(), email: "bob@example.com".to_string() },
    /// ];
    /// table.insert_batch(&users).await?;
    /// # Ok(())
    /// # }
    /// # struct User { id: Option<i64>, name: String, email: String }
    /// ```
    fn insert_batch(
        &self,
        entities: &[Entity],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Retrieve an entity by its primary key.
    ///
    /// This method performs a SELECT query to find an entity with the specified primary key.
    /// The result is deserialized into the entity type using `sqlx::FromRow`.
    ///
    /// ## Arguments
    /// * `id` - A reference to the primary key value
    ///
    /// ## Returns
    /// * `Ok(Some(entity))` if the entity was found
    /// * `Ok(None)` if no entity with the given ID exists
    /// * `Err(Self::Error)` if the query failed
    ///
    /// ## Example
    /// ```rust
    /// # use typed_sqlx_client::CrudOpsRef;
    /// # async fn example(table: impl CrudOpsRef<i64, User, Error = sqlx::Error>) -> Result<(), sqlx::Error> {
    /// let user_id = 42;
    /// match table.get_by_id(&user_id).await? {
    ///     Some(user) => println!("Found user: {:?}", user),
    ///     None => println!("User with ID {} not found", user_id),
    /// }
    /// # Ok(())
    /// # }
    /// # #[derive(Debug)] struct User { id: Option<i64>, name: String, email: String }
    /// ```
    fn get_by_id(
        &self,
        id: &ID,
    ) -> impl Future<Output = Result<Option<Entity>, Self::Error>> + Send;

    /// Update an existing entity by its primary key.
    ///
    /// This method performs an UPDATE query to modify an existing record in the database.
    /// All fields except the primary key are updated with values from the provided entity.
    /// If no record with the given ID exists, the operation succeeds but affects 0 rows.
    ///
    /// ## Arguments
    /// * `id` - A reference to the primary key value of the record to update
    /// * `entity` - A reference to the entity containing the new data
    ///
    /// ## Returns
    /// * `Ok(())` if the update was successful (regardless of rows affected)
    /// * `Err(Self::Error)` if the update failed
    ///
    /// ## Example
    /// ```rust
    /// # use typed_sqlx_client::CrudOpsRef;
    /// # async fn example(table: impl CrudOpsRef<i64, User, Error = sqlx::Error>) -> Result<(), sqlx::Error> {
    /// let user_id = 42;
    /// let updated_user = User {
    ///     id: Some(user_id),
    ///     name: "Alice Updated".to_string(),
    ///     email: "alice.new@example.com".to_string()
    /// };
    /// table.update_by_id(&user_id, &updated_user).await?;
    /// # Ok(())
    /// # }
    /// # struct User { id: Option<i64>, name: String, email: String }
    /// ```
    fn update_by_id(
        &self,
        id: &ID,
        entity: &Entity,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Delete an entity by its primary key.
    ///
    /// This method performs a DELETE query to remove a record from the database.
    /// The operation succeeds even if no record with the given ID exists.
    ///
    /// ## Arguments
    /// * `id` - A reference to the primary key value of the record to delete
    ///
    /// ## Returns
    /// * `Ok(())` if the deletion was successful (even if no rows were affected)
    /// * `Err(Self::Error)` if the deletion failed
    ///
    /// ## Example
    /// ```rust
    /// # use typed_sqlx_client::CrudOpsRef;
    /// # async fn example(table: impl CrudOpsRef<i64, User, Error = sqlx::Error>) -> Result<(), sqlx::Error> {
    /// let user_id = 42;
    /// table.delete_by_id(&user_id).await?;
    /// println!("User with ID {} has been deleted (or didn't exist)", user_id);
    /// # Ok(())
    /// # }
    /// # struct User { id: Option<i64>, name: String, email: String }
    /// ```
    fn delete_by_id(&self, id: &ID) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

// /// Trait for async CRUD operations using owned entities.
// /// Suitable for small entities or when ownership transfer is desired.
// pub trait CrudOps<ID, Entity> {
//     /// The error type for operations
//     type Error;

//     /// Insert a single entity by value (ownership is moved).
//     fn insert(&self, entity: Entity) -> impl Future<Output = Result<(), Self::Error>> + Send;
//     /// Insert a batch of entities by value (ownership is moved).
//     fn insert_batch(&self, entities: Vec<Entity>) -> impl Future<Output = Result<(), Self::Error>> + Send;
//     /// Get an entity by id, returns owned entity if found.
//     fn get_by_id(&self, id: &ID) -> impl Future<Output = Result<Option<Entity>, Self::Error>> + Send;
//     /// Update an entity by id, using an owned entity (ownership is moved).
//     fn update_by_id(&self, id: &ID, entity: Entity) -> impl Future<Output = Result<(), Self::Error>> + Send;
//     /// Delete an entity by id.
//     fn delete_by_id(&self, id: &ID) -> impl Future<Output = Result<(), Self::Error>> + Send;
// }

/// Trait for executing safe, read-only SQL queries with flexible result types.
///
/// This trait provides two complementary query methods designed for different use cases:
/// - **JSON mode**: For dynamic schemas, debugging, or when working with unknown column structures
/// - **Type-safe mode**: For compile-time verified deserialization into specific Rust types
///
/// ## Safety & Security
/// This trait is designed exclusively for SELECT statements to ensure database safety.
/// All queries are validated to prevent accidental data modification operations.
///
/// ## Implementation Note
/// Starting from v0.2.0, this trait is implemented directly on `SqlTable<P, DB, Table>`
/// and no longer requires macro generation. This provides better IDE support and
/// compile-time error messages.
///
/// ## Type Parameters
/// * `P` - The sqlx database driver type (`sqlx::Postgres`, `sqlx::MySql`, `sqlx::Sqlite`)
///
/// ## Usage Patterns
///
/// ### 1. Type-safe queries (Recommended)
/// Use `execute_select_as_only<T>()` when you know the query structure:
/// ```rust
/// # use typed_sqlx_client::SelectOnlyQuery;
/// # async fn example(table: impl SelectOnlyQuery<sqlx::Postgres, MError = sqlx::Error>) -> Result<(), sqlx::Error> {
/// // Simple entity queries
/// let users: Vec<User> = table.execute_select_as_only::<User>("SELECT * FROM users").await?;
///
/// // Custom projections
/// let names: Vec<(String,)> = table.execute_select_as_only::<(String,)>("SELECT name FROM users").await?;
///
/// // Aggregations
/// let counts: Vec<(i64, String)> = table.execute_select_as_only::<(i64, String)>(
///     "SELECT COUNT(*), department FROM users GROUP BY department"
/// ).await?;
/// # Ok(())
/// # }
/// # struct User;
/// ```
///
/// ### 2. Dynamic JSON queries
/// Use `execute_select_only()` for flexible or unknown schemas:
/// ```rust
/// # use typed_sqlx_client::SelectOnlyQuery;
/// # async fn example(table: impl SelectOnlyQuery<sqlx::Postgres, MError = sqlx::Error, Output = Vec<serde_json::Value>>) -> Result<(), sqlx::Error> {
/// let json_rows = table.execute_select_only("SELECT * FROM users").await?;
/// for row in json_rows {
///     if let Some(name) = row.get("name").and_then(|v| v.as_str()) {
///         println!("User name: {}", name);
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Error Handling
/// Both methods return database-specific errors (typically `sqlx::Error`) for:
/// - Invalid SQL syntax
/// - Connection failures  
/// - Type conversion errors (type-safe mode only)
/// - Non-SELECT statements (security validation)
pub trait SelectOnlyQuery<P: sqlx::Database> {
    /// The error type for query execution.
    type MError;
    /// The output/result type for the query (e.g. a JSON wrapper or custom struct).
    type Output;

    /// Execute a SELECT query and return results as JSON values.
    ///
    /// This method provides maximum flexibility for dynamic queries where the column
    /// structure may not be known at compile time. Results are returned as a vector
    /// of `serde_json::Value` objects, allowing runtime inspection and manipulation.
    ///
    /// ## Use Cases
    /// - **Dynamic reporting**: Queries with variable column selection
    /// - **API endpoints**: Where client specifies custom projections
    /// - **Debugging**: Quick inspection of query results
    /// - **Schema exploration**: Understanding unknown database structures
    /// - **Aggregations**: When working with computed columns
    ///
    /// ## Arguments
    /// * `query` - The SQL SELECT statement to execute. Only SELECT statements are allowed.
    ///
    /// ## Returns
    /// * `Ok(Vec<serde_json::Value>)` - Each element represents one row as a JSON object
    /// * `Err(Self::MError)` - Database errors or non-SELECT statement rejection
    ///
    /// ## Data Type Mapping
    /// Database types are automatically converted to JSON:
    /// - Integers → `json!(number)`
    /// - Floats → `json!(number)`  
    /// - Booleans → `json!(boolean)`
    /// - Strings → `json!(string)` or parsed JSON if valid
    /// - Binary data → `json!(array)` of bytes
    /// - NULL values → `json!(null)`
    ///
    /// ## Example
    /// ```rust
    /// # use typed_sqlx_client::SelectOnlyQuery;
    /// # async fn example(table: impl SelectOnlyQuery<sqlx::Postgres, MError = sqlx::Error, Output = Vec<serde_json::Value>>) -> Result<(), sqlx::Error> {
    /// // Dynamic query with custom projection
    /// let results = table.execute_select_only(
    ///     "SELECT name, age, created_at FROM users WHERE department = 'engineering'"
    /// ).await?;
    ///
    /// for row in results {
    ///     let name = row["name"].as_str().unwrap_or("Unknown");
    ///     let age = row["age"].as_i64().unwrap_or(0);
    ///     println!("User: {} (age: {})", name, age);
    /// }
    ///
    /// // Aggregation query
    /// let stats = table.execute_select_only(
    ///     "SELECT department, COUNT(*) as count, AVG(age) as avg_age FROM users GROUP BY department"
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    fn execute_select_only(
        &self,
        query: &str,
    ) -> impl Future<Output = Result<Self::Output, Self::MError>> + Send;

    /// Execute a SELECT query and return strongly-typed results.
    ///
    /// This method provides compile-time type safety by deserializing query results
    /// directly into Rust types. It's the recommended approach when you know the
    /// expected structure of your query results.
    ///
    /// ## Key Benefits
    /// - **Type safety**: Compile-time verification of result structure
    /// - **Performance**: Direct deserialization without JSON intermediate step
    /// - **IDE support**: Full autocomplete and error checking
    /// - **Maintainability**: Refactoring-safe with strong typing
    ///
    /// ## Use Cases
    /// - **Entity queries**: `SELECT * FROM table` → `Vec<Entity>`
    /// - **Custom projections**: `SELECT name, email FROM users` → `Vec<(String, String)>`
    /// - **Aggregations**: `SELECT COUNT(*) FROM table` → `Vec<(i64,)>`
    /// - **Joins**: Complex queries with predictable result structure
    /// - **Computed fields**: Queries with calculated columns
    ///
    /// ## Type Parameters
    /// * `T` - The target type for deserialization. Must implement `sqlx::FromRow`.
    ///
    /// ## Arguments
    /// * `query` - The SQL SELECT statement to execute.
    ///
    /// ## Returns
    /// * `Ok(Vec<T>)` - Vector of deserialized results
    /// * `Err(Self::MError)` - Database errors, type conversion errors, or invalid SQL
    ///
    /// ## Supported Target Types
    ///
    /// ### 1. Full Entity Types
    /// ```rust
    /// # use typed_sqlx_client::SelectOnlyQuery;
    /// # use sqlx::FromRow;
    /// # #[derive(FromRow)]
    /// # struct User { id: i64, name: String, email: String }
    /// # async fn example(table: impl SelectOnlyQuery<sqlx::Postgres, MError = sqlx::Error>) -> Result<(), sqlx::Error> {
    /// let users: Vec<User> = table.execute_select_as_only::<User>(
    ///     "SELECT id, name, email FROM users WHERE active = true"
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### 2. Tuple Types (for projections)
    /// ```rust
    /// # use typed_sqlx_client::SelectOnlyQuery;
    /// # async fn example(table: impl SelectOnlyQuery<sqlx::Postgres, MError = sqlx::Error>) -> Result<(), sqlx::Error> {
    /// // Single column
    /// let names: Vec<(String,)> = table.execute_select_as_only::<(String,)>(
    ///     "SELECT name FROM users"
    /// ).await?;
    ///
    /// // Multiple columns
    /// let user_info: Vec<(String, i32, bool)> = table.execute_select_as_only::<(String, i32, bool)>(
    ///     "SELECT name, age, is_active FROM users"
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### 3. Aggregation Results
    /// ```rust
    /// # use typed_sqlx_client::SelectOnlyQuery;
    /// # async fn example(table: impl SelectOnlyQuery<sqlx::Postgres, MError = sqlx::Error>) -> Result<(), sqlx::Error> {
    /// let counts: Vec<(i64,)> = table.execute_select_as_only::<(i64,)>(
    ///     "SELECT COUNT(*) FROM users"
    /// ).await?;
    ///
    /// let department_stats: Vec<(String, i64, f64)> = table.execute_select_as_only::<(String, i64, f64)>(
    ///     "SELECT department, COUNT(*), AVG(salary) FROM employees GROUP BY department"
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Error Handling
    /// ```rust
    /// # use typed_sqlx_client::SelectOnlyQuery;
    /// # async fn example(table: impl SelectOnlyQuery<sqlx::Postgres, MError = sqlx::Error>) {
    /// match table.execute_select_as_only::<(String,)>("SELECT name FROM users").await {
    ///     Ok(names) => {
    ///         for (name,) in names {
    ///             println!("User: {}", name);
    ///         }
    ///     },
    ///     Err(sqlx::Error::ColumnNotFound(col)) => {
    ///         eprintln!("Column '{}' not found in result", col);
    ///     },
    ///     Err(e) => eprintln!("Query failed: {}", e),
    /// }
    /// # }
    /// ```
    fn execute_select_as_only<T>(
        &self,
        query: &str,
    ) -> impl Future<Output = Result<Vec<T>, Self::MError>> + Send
    where
        T: for<'r> sqlx::FromRow<'r, <P as sqlx::Database>::Row> + Send + Unpin + 'static;
}
