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
