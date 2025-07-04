use std::future::Future;

/// Trait for async CRUD operations using references to entities.
/// Suitable for large entities or when avoiding unnecessary copies.
pub trait CrudOpsRef<ID, Entity> {
    /// The error type for operations
    type Error;

    /// Insert a single entity by reference.
    fn insert(&self, entity: &Entity) -> impl Future<Output = Result<(), Self::Error>> + Send;
    /// Insert a batch of entities by reference.
    fn insert_batch(&self, entities: &[Entity]) -> impl Future<Output = Result<(), Self::Error>> + Send;
    /// Get an entity by id, returns owned entity if found.
    fn get_by_id(&self, id: &ID) -> impl Future<Output = Result<Option<Entity>, Self::Error>> + Send;
    /// Update an entity by id, using a reference to the new entity data.
    fn update_by_id(&self, id: &ID, entity: &Entity) -> impl Future<Output = Result<(), Self::Error>> + Send;
    /// Delete an entity by id.
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

/// Trait for executing only safe, read-only SQL queries (SELECT) and returning a custom result type.
///
/// This trait is designed for advanced scenarios where you need to run dynamic or arbitrary SELECT queries
/// and get results in a flexible format (such as JSON or a custom struct). It is strongly recommended to
/// only allow SELECT statements to ensure database safety and prevent accidental data modification.
pub trait SelectOnlyQuery<P: sqlx::Database> {
    /// The error type for query execution.
    type MError;
    /// The output/result type for the query (e.g. a JSON wrapper or custom struct).
    type Output;

    /// Execute a raw SELECT SQL query and return the result.
    ///
    /// # Arguments
    /// * `query` - The SQL string to execute. Only SELECT statements are allowed.
    fn execute_select_only(&self, query: &str) -> impl Future<Output = Result<Self::Output, Self::MError>> + Send;

    /// Execute a raw SELECT SQL query and return the result as a vector of strongly-typed entities.
    ///
    /// This method allows you to execute arbitrary SELECT statements and directly deserialize each row
    /// into a Rust struct or tuple that implements [`sqlx::FromRow`]. This is useful for type-safe queries,
    /// custom projections, or when you want to work with concrete types instead of generic JSON values.
    ///
    /// # Type Parameters
    /// * `T` - The target type for each row, must implement [`sqlx::FromRow`].
    ///
    /// # Arguments
    /// * `query` - The SQL SELECT statement to execute.
    ///
    /// # Returns
    /// * `Result<Vec<T>, Self::MError>` - A vector of deserialized rows, or an error.
    ///
    /// # Example
    /// ```
    /// let users: Vec<User> = table.execute_select_as_only::<User>("SELECT * FROM users").await?;
    /// ```
    fn execute_select_as_only<T>(&self, query: &str) -> impl Future<Output = Result<Vec<T>, Self::MError>> + Send
    where
        T: for<'r> sqlx::FromRow<'r, <P as sqlx::Database>::Row> + Send + Unpin + 'static;
}