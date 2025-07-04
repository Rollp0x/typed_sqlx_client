use typed_sqlx_client::{CrudOpsRef, SqlPool, SelectOnlyQuery};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;
use ethereum_mysql::{SqlAddress,sqladdress};
use serde::{Serialize, Deserialize};

#[derive(FromRow, CrudOpsRef, Clone, Debug, Serialize, Deserialize)]
#[crud(table = "user_infos",db = "postgres")]
pub struct UserInfo {
    #[crud(primary_key)]
    pub id: Option<Uuid>,
    pub name: String,
    pub email: String,
    pub age: Option<i32>,
    pub is_active: bool,
    #[crud(rename = "address")]
    #[sqlx(rename = "address")]
    pub user_address: SqlAddress,
}
pub struct TestDB;

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgres://postgres:123456@localhost:5432/test_db")
        .await
        .expect("Failed to connect to PostgreSQL");
    let pool = SqlPool::from_pool::<TestDB>(pool);
    let user_info_table = pool.get_table::<UserInfo>();
    // Drop table if exists to ensure a fresh table each time
    let _ = sqlx::query("DROP TABLE IF EXISTS user_infos").execute(user_info_table.get_pool()).await.unwrap();
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS user_infos (
            id UUID PRIMARY KEY,
            name VARCHAR(25) NOT NULL,
            email VARCHAR(255) NOT NULL,
            age INT,
            is_active BOOLEAN NOT NULL,
            address VARCHAR(42) NOT NULL
        )"
    ).execute(user_info_table.get_pool()).await.unwrap();

    let user_info = UserInfo {
        id: Some(Uuid::new_v4()),
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        age: Some(30),
        is_active: true,
        user_address: sqladdress!("0x1234567890abcdef1234567890abcdef12345678"),
    };
    user_info_table.insert(&user_info).await.unwrap();
    let sql = "select * from user_infos";
    let query = user_info_table.execute_select_as_only::<UserInfo>(sql).await.unwrap();
    assert!(query.len() == 1, "Expected one user info record");
    let into_of_value = query.into_iter().next().unwrap();
    println!("Query result: {:?}", into_of_value);
    let uuid = into_of_value.id.as_ref().unwrap().clone();
    let mut user_info = user_info_table.get_by_id(&uuid).await.unwrap().unwrap();
    assert!(user_info.age == Some(30), "Expected user_info.age to be Some(30)");
    user_info.age = Some(45);
    user_info_table.update_by_id(&uuid,&user_info).await.unwrap();
    user_info = user_info_table.get_by_id(&uuid).await.unwrap().unwrap();
    assert_eq!(user_info.age, Some(45), "Expected age to be updated to 45");
    user_info_table.delete_by_id(&uuid).await.unwrap();
    let deleted_user = user_info_table.get_by_id(&uuid).await.unwrap();
    assert!(deleted_user.is_none(), "Expected user info to be deleted");
    let user1_info = UserInfo {
        id: Some(Uuid::new_v4()),
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
        age: Some(25),
        is_active: true,
        user_address: sqladdress!("0x0000abcdefabcdefabcdefabcdefabcdefabcdef"),
    };
    let user_infos = vec![user_info, user1_info];
    user_info_table.insert_batch(&user_infos).await.unwrap();
    let sql = "SELECT count(*) FROM user_infos";
    let count_result = user_info_table.execute_select_only(sql).await.unwrap();
    assert!(count_result.len() == 1, "Expected one row in count result");
    if let Some(count_value) = count_result.into_iter().next() {
        if let Some(count) = count_value.get("count(*)") {
            if let Some(count_num) = count.as_u64() {
                assert_eq!(count_num, 2, "Expected 2 user infos after batch insert");
            } else {
                panic!("Expected count to be a number");
            }
        }
    }
    let count_result:Vec<(i64,)> = user_info_table.execute_select_as_only::<(i64,)>(sql).await.unwrap();
    assert!(count_result.len() == 1, "Expected one row in count result");
    let count = count_result[0].0;
    assert_eq!(count, 2, "Expected count to be 2 after batch insert");
    println!("All operations completed successfully!");
}