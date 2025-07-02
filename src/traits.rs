use async_trait::async_trait;
/// Trait for async CRUD operations using references to entities.
/// Suitable for large entities or when avoiding unnecessary copies.
#[async_trait]
pub trait CrudOpsRef<ID, Entity> {
    type Error;

    /// Insert a single entity by reference.
    async fn insert(&self, entity: &Entity) -> Result<(), Self::Error>;
    /// Insert a batch of entities by reference.
    async fn insert_batch(&self, entities: &[Entity]) -> Result<(), Self::Error>;
    /// Get an entity by id, returns owned entity if found.
    async fn get_by_id(&self, id: &ID) -> Result<Option<Entity>, Self::Error>;
    /// Update an entity by id, using a reference to the new entity data.
    async fn update_by_id(&self, id: &ID, entity: &Entity) -> Result<(), Self::Error>;
    /// Delete an entity by id.
    async fn delete_by_id(&self, id: &ID) -> Result<(), Self::Error>;
}

/// Trait for async CRUD operations using owned entities.
/// Suitable for small entities or when ownership transfer is desired.
#[async_trait]
pub trait CrudOps<ID, Entity> {
    type Error;

    /// Insert a single entity by value (ownership is moved).
    async fn insert(&self, entity: Entity) -> Result<(), Self::Error>;
    /// Insert a batch of entities by value (ownership is moved).
    async fn insert_batch(&self, entities: Vec<Entity>) -> Result<(), Self::Error>;
    /// Get an entity by id, returns owned entity if found.
    async fn get_by_id(&self, id: &ID) -> Result<Option<Entity>, Self::Error>;
    /// Update an entity by id, using an owned entity (ownership is moved).
    async fn update_by_id(&self, id: &ID, entity: Entity) -> Result<(), Self::Error>;
    /// Delete an entity by id.
    async fn delete_by_id(&self, id: &ID) -> Result<(), Self::Error>;
}

/// Trait for executing only safe, read-only SQL queries (SELECT) and returning a custom result type.
///
/// This trait is designed for advanced scenarios where you need to run dynamic or arbitrary SELECT queries
/// and get results in a flexible format (such as JSON or a custom struct). It is strongly recommended to
/// only allow SELECT statements to ensure database safety and prevent accidental data modification.
#[async_trait]
pub trait SelectOnlyQuery {
    /// The error type for query execution.
    type MError;
    /// The output/result type for the query (e.g. a JSON wrapper or custom struct).
    type Output;

    /// Execute a raw SELECT SQL query and return the result.
    ///
    /// # Arguments
    /// * `query` - The SQL string to execute. Only SELECT statements are allowed.
    async fn execute_select_only(&self, query: &str) -> Result<Self::Output, Self::MError>;
}
