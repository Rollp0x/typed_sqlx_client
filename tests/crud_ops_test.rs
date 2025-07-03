use typed_sqlx_client::CrudOpsRef;
use sqlx::FromRow;
use ethereum_mysql::SqlAddress;

#[derive(FromRow, CrudOpsRef)]
#[crud(table = "users")]
struct User {
    #[crud(primary_key)]
    id: Option<i64>,
    name: String,
    email: String,
    age: Option<i32>,
    is_active: bool,
    user_address: SqlAddress,
}



fn main() {
    println!("Macro compilation test passed!");
}