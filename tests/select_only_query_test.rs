use typed_sqlx_client::tables::SqlTable;
use typed_sqlx_client::traits::SelectOnlyQuery;

pub struct TestDatabase;
pub struct TestTable;

#[test]
fn test_sql_table_mysql_implements_select_only_query() {
    use sqlx::MySql;

    // Compile-time check: ensure SqlTable implements SelectOnlyQuery
    fn assert_implements_select_only_query<T: SelectOnlyQuery>() {}

    // This function call will verify trait implementation at compile time
    assert_implements_select_only_query::<SqlTable<MySql, TestDatabase, TestTable>>();

    // If compilation passes, the trait implementation is correct
    println!("SqlTable<MySql, TestDatabase, TestTable> implements SelectOnlyQuery");
}

#[test]
fn test_sql_table_sqlite_implements_select_only_query() {
    use sqlx::Sqlite;

    // Compile-time check: ensure SqlTable implements SelectOnlyQuery
    fn assert_implements_select_only_query<T: SelectOnlyQuery>() {}

    // This function call will verify trait implementation at compile time
    assert_implements_select_only_query::<SqlTable<Sqlite, TestDatabase, TestTable>>();

    // If compilation passes, the trait implementation is correct
    println!("SqlTable<Sqlite, TestDatabase, TestTable> implements SelectOnlyQuery");
}

#[test]
fn test_sql_table_postgres_implements_select_only_query() {
    use sqlx::Postgres;

    // Compile-time check: ensure SqlTable implements SelectOnlyQuery
    fn assert_implements_select_only_query<T: SelectOnlyQuery>() {}

    // This function call will verify trait implementation at compile time
    assert_implements_select_only_query::<SqlTable<Postgres, TestDatabase, TestTable>>();

    // If compilation passes, the trait implementation is correct
    println!("SqlTable<Postgres, TestDatabase, TestTable> implements SelectOnlyQuery");
}
