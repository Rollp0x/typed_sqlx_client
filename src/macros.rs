/// Macro to implement SelectOnlyQuery for a given sqlx database type (e.g. MySql, Postgres, Sqlite)
/// Usage: select_only_query!(MySql); select_only_query!(Postgres); select_only_query!(Sqlite);
#[macro_export]
macro_rules! select_only_query {
    ($db:ty) => {
        #[async_trait::async_trait]
        impl<DB, Table> $crate::SelectOnlyQuery for $crate::SqlTable<$db, DB, Table>
        where
            DB: Sync,
            Table: Sync,
        {
            type MError = $crate::private::sqlx::Error;
            type Output = Vec<$crate::private::serde_json::Value>;

            async fn execute_select_only(&self, query: &str) -> Result<Self::Output, Self::MError> {
                use $crate::private::serde_json;
                use $crate::private::sqlx::{self, Column, Row};
                let trimmed_query = query.trim().to_lowercase();
                if !trimmed_query.starts_with("select") {
                    return Err($crate::private::sqlx::Error::InvalidArgument(
                        "Only SELECT queries are allowed".into(),
                    ));
                }
                let rows = sqlx::query(query).fetch_all(self.as_ref()).await?;
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
                        // Try to get as i64, f64, bool, then String, fallback to null
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
    };
}
