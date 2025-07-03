use typed_sqlx_client::ToRow;

// Test basic struct without any attributes
#[derive(ToRow)]
struct BasicUser {
    id: i64,
    name: String,
    email: String,
}

// Test struct with table name specified
#[derive(ToRow)]
#[torow(table = "users")]
struct User {
    #[torow(primary_key)]
    id: i64,
    #[torow(rename = "user_name")]
    name: String,
    email: String,
}

// Test struct with different primary key
#[derive(ToRow)]
#[torow(table = "products")]
struct Product {
    #[torow(primary_key)]
    product_id: String,
    #[torow(rename = "product_name")]
    name: String,
    price: f64,
}

// Test struct with multiple renamed fields
#[derive(ToRow)]
#[torow(table = "orders")]
struct Order {
    #[torow(primary_key)]
    order_id: i64,
    #[torow(rename = "customer_id")]
    customer: i64,
    #[torow(rename = "order_date")]
    date: String,
    #[torow(rename = "total_amount")]
    total: f64,
}

#[test]
fn test_basic_user_to_row() {
    // Test default table name (struct name)
    assert_eq!(BasicUser::TABLE_NAME, "BasicUser");
    
    // Test default primary key field
    assert_eq!(BasicUser::PRIMARY_KEY_FIELD, "id");
    
    // Test field mappings (all fields should map to themselves)
    let mappings = BasicUser::field_column_mappings();
    assert_eq!(mappings.len(), 3);
    assert!(mappings.contains(&("id", "id")));
    assert!(mappings.contains(&("name", "name")));
    assert!(mappings.contains(&("email", "email")));
}

#[test]
fn test_user_to_row() {
    // Test custom table name
    assert_eq!(User::TABLE_NAME, "users");
    
    // Test primary key field
    assert_eq!(User::PRIMARY_KEY_FIELD, "id");
    
    // Test field mappings with rename
    let mappings = User::field_column_mappings();
    assert_eq!(mappings.len(), 3);
    assert!(mappings.contains(&("id", "id")));
    assert!(mappings.contains(&("name", "user_name")));
    assert!(mappings.contains(&("email", "email")));
}

#[test]
fn test_product_to_row() {
    // Test custom table name
    assert_eq!(Product::TABLE_NAME, "products");
    
    // Test custom primary key field
    assert_eq!(Product::PRIMARY_KEY_FIELD, "product_id");
    
    // Test field mappings
    let mappings = Product::field_column_mappings();
    assert_eq!(mappings.len(), 3);
    assert!(mappings.contains(&("product_id", "product_id")));
    assert!(mappings.contains(&("name", "product_name")));
    assert!(mappings.contains(&("price", "price")));
}

#[test]
fn test_order_to_row() {
    // Test custom table name
    assert_eq!(Order::TABLE_NAME, "orders");
    
    // Test custom primary key field
    assert_eq!(Order::PRIMARY_KEY_FIELD, "order_id");
    
    // Test field mappings with multiple renames
    let mappings = Order::field_column_mappings();
    assert_eq!(mappings.len(), 4);
    assert!(mappings.contains(&("order_id", "order_id")));
    assert!(mappings.contains(&("customer", "customer_id")));
    assert!(mappings.contains(&("date", "order_date")));
    assert!(mappings.contains(&("total", "total_amount")));
}

#[test]
fn test_field_mappings_order() {
    // Test that field mappings maintain the order of struct fields
    let mappings = User::field_column_mappings();
    
    // The order should match the order of fields in the struct definition
    assert_eq!(mappings[0], ("id", "id"));
    assert_eq!(mappings[1], ("name", "user_name"));
    assert_eq!(mappings[2], ("email", "email"));
}

#[test]
fn test_compilation_check() {
    // Compile-time check to ensure the trait is properly implemented
    fn assert_implements_to_row<T: ToRow>() {}
    
    assert_implements_to_row::<BasicUser>();
    assert_implements_to_row::<User>();
    assert_implements_to_row::<Product>();
    assert_implements_to_row::<Order>();
}

#[test]
fn test_trait_methods_accessible() {
    // Test that all trait methods are accessible and return expected types
    let _table_name: &'static str = User::TABLE_NAME;
    let _pk_field: &'static str = User::PRIMARY_KEY_FIELD;
    let _mappings: Vec<(&'static str, &'static str)> = User::field_column_mappings();
    
    // This test passes if it compiles successfully
    assert!(true);
}