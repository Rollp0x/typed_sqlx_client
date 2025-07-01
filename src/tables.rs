use sqlx::{Pool, database::Database};
use std::marker::PhantomData;
use std::ops::Deref;

/// Type-safe wrapper for a database connection pool. `DB` is a marker type to distinguish different database instances.
pub struct SqlPool<P: Database, DB>(Pool<P>, PhantomData<DB>);

impl<P: Database, DB> SqlPool<P, DB> {
    /// Get the underlying sqlx::Pool
    pub fn pool(&self) -> &Pool<P> {
        &self.0
    }
}

/// implement Clone for SqlPool
/// This allows SqlPool to be cloned, which is useful for passing it around in async contexts
impl<P: Database, DB> Clone for SqlPool<P, DB> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<P: Database> SqlPool<P, ()> {
    /// Factory function to create a typed SqlPool from a raw pool.
    pub fn from_pool<DB>(pool: Pool<P>) -> SqlPool<P, DB> {
        SqlPool(pool, PhantomData)
    }
}

/// Type-safe handle for table operations. `Table` is a marker type for the table.
#[derive(Clone)]
pub struct SqlTable<P: Database, DB, Table>(SqlPool<P, DB>, PhantomData<Table>);

impl<P: Database, DB> SqlPool<P, DB> {
    /// Get a SqlTable handle for a specific table type
    pub fn get_table<Table>(&self) -> SqlTable<P, DB, Table> {
        SqlTable(self.clone(), PhantomData)
    }
}

impl<P: Database, DB, Table> SqlTable<P, DB, Table> {
    /// Get the underlying sqlx::Pool
    pub fn get_pool(&self) -> &Pool<P> {
        self.0.pool()
    }
}

/// Allow passing SqlTable as &Pool<P> to sqlx queries
impl<P: Database, DB, Table> AsRef<Pool<P>> for SqlTable<P, DB, Table>
where
    P: Database,
{
    fn as_ref(&self) -> &Pool<P> {
        self.get_pool()
    }
}

/// Allow using &SqlTable as &Pool<P> directly
impl<P: Database, DB, Table> Deref for SqlTable<P, DB, Table>
where
    P: Database,
{
    type Target = Pool<P>;

    fn deref(&self) -> &Self::Target {
        self.get_pool()
    }
}
