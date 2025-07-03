use crate::traits::SelectOnlyQuery;
use sqlx::{
    Column, ColumnIndex, Decode, Executor, IntoArguments, Pool, Row, Type, database::Database,
};
use std::marker::PhantomData;
use std::ops::Deref;

/// Type-safe wrapper for a database connection pool.
/// `DB` is a marker type to distinguish different database instances.
pub struct SqlPool<P: Database, DB>(Pool<P>, PhantomData<DB>);

impl<P: Database, DB> SqlPool<P, DB> {
    /// Get the underlying sqlx::Pool
    pub fn pool(&self) -> &Pool<P> {
        &self.0
    }
}

/// Implement Clone for SqlPool.
/// This allows SqlPool to be cloned, which is useful for passing it around in async contexts.
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

/// Type-safe handle for table operations.
/// `Table` is a marker type for the table.
#[derive(Clone)]
pub struct SqlTable<P: Database, DB, Table>(SqlPool<P, DB>, PhantomData<Table>);

impl<P: Database, DB> SqlPool<P, DB> {
    /// Get a SqlTable handle for a specific table type.
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
impl<P: Database, DB, Table> AsRef<Pool<P>> for SqlTable<P, DB, Table> {
    fn as_ref(&self) -> &Pool<P> {
        self.get_pool()
    }
}

/// Allow using &SqlTable as &Pool<P> directly
impl<P: Database, DB, Table> Deref for SqlTable<P, DB, Table> {
    type Target = Pool<P>;

    fn deref(&self) -> &Self::Target {
        self.get_pool()
    }
}

impl<P: Database, DB, Table> SelectOnlyQuery for SqlTable<P, DB, Table>
where
    DB: Sync + Send,
    Table: Sync + Send,
    P::Row: Row<Database = P>,
    P::Column: Column<Database = P>,
    for<'r> &'r Pool<P>: Executor<'r, Database = P>,
    for<'r> &'r str: ColumnIndex<P::Row>,
    for<'r> i64: Type<P> + Decode<'r, P>,
    for<'r> f64: Type<P> + Decode<'r, P>,
    for<'r> bool: Type<P> + Decode<'r, P>,
    for<'r> String: Type<P> + Decode<'r, P>,
    for<'q> P::Arguments<'q>: IntoArguments<'q, P>,
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
        let columns = if let Some(row) = rows.get(0) {
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
                } else {
                    serde_json::json!(null)
                };
                json_row.insert(column.clone(), json_value);
            }
            result.push(serde_json::Value::Object(json_row));
        }
        Ok(result)
    }
}
