use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Meta};

/// Derive macro for automatically implementing the CrudOpsRef trait.
///
/// This macro is the cornerstone of `typed_sqlx_client` v0.2.0+, replacing the old `CrudOps` trait
/// with a more powerful and flexible derive-based approach. It generates a complete implementation
/// of the `CrudOpsRef` trait for structs that implement `sqlx::FromRow`, providing automatic
/// CRUD operations with support for field renaming, custom table names, and multiple database backends.
///
/// ## 🚀 What's New in v0.2.0
/// - **Replaced** the old `CrudOps` trait entirely
/// - **Enhanced** field mapping with `#[crud(rename = "...")]`
/// - **Improved** primary key detection and handling (including `Option<T>` unwrapping)
/// - **Unified** API across MySQL, PostgreSQL, and SQLite
/// - **Better** error messages and IDE support
///
/// ## 📋 Requirements
/// - The struct must implement `sqlx::FromRow` (usually via `#[derive(FromRow)]`)
/// - All field types must implement `sqlx::Encode` and `sqlx::Type` for the target database
/// - The struct must have at least one field
///
/// ## 🗄️ Supported Databases
/// | Database   | Identifier | Parameter Style | Status |
/// |------------|------------|----------------|---------|
/// | MySQL      | `db = "mysql"` | `?` placeholders | ✅ Stable |
/// | PostgreSQL | `db = "postgres"` | `$1, $2, ...` placeholders | ✅ Stable |
/// | SQLite     | `db = "sqlite"` | `?` placeholders | ✅ Stable |
///
/// ## 🏷️ Attributes Reference
///
/// ### Struct-level Attributes
/// ```rust
/// #[crud(table = "table_name")]          // Specify table name (defaults to struct name)
/// #[crud(db = "database_type")]           // Specify database type (mysql/postgres/sqlite)
/// #[crud(table = "users", db = "postgres")]  // Combined syntax
/// ```
///
/// ### Field-level Attributes
/// ```rust
/// #[crud(primary_key)]                   // Mark field as primary key (defaults to first field)
/// #[crud(rename = "column_name")]         // Map field to different column name
/// ```
///
/// ## 🔧 Generated Operations
/// The macro implements these methods on `SqlTable<P, DB, YourStruct>`:
/// - `insert(&self, entity: &T) -> Result<(), sqlx::Error>`
/// - `insert_batch(&self, entities: &[T]) -> Result<(), sqlx::Error>`
/// - `get_by_id(&self, id: &ID) -> Result<Option<T>, sqlx::Error>`
/// - `update_by_id(&self, id: &ID, entity: &T) -> Result<(), sqlx::Error>`
/// - `delete_by_id(&self, id: &ID) -> Result<(), sqlx::Error>`
///
/// ## 📚 Usage Examples
///
/// ### Basic Entity
/// ```rust
/// use typed_sqlx_client::{CrudOpsRef, SqlPool};
/// use sqlx::FromRow;
///
/// #[derive(FromRow, CrudOpsRef, Debug)]
/// #[crud(table = "users", db = "postgres")]
/// struct User {
///     #[crud(primary_key)]
///     id: Option<i64>,  // Option<T> is automatically unwrapped for key type
///     name: String,
///     email: String,
/// }
///
/// struct MainDB;
///
/// # async fn example() -> Result<(), sqlx::Error> {
/// let pool = SqlPool::from_pool::<MainDB>(pg_pool);
/// let user_table = pool.get_table::<User>();
///
/// // All CRUD operations are now available
/// let user = User { id: None, name: "Alice".to_string(), email: "alice@example.com".to_string() };
/// user_table.insert(&user).await?;
/// # Ok(())
/// # }
/// ```
///
/// ### Advanced Field Mapping
/// ```rust
/// use uuid::Uuid;
///
/// #[derive(FromRow, CrudOpsRef)]
/// #[crud(table = "user_profiles", db = "postgres")]
/// struct UserProfile {
///     #[crud(primary_key)]
///     id: Option<Uuid>,
///     
///     #[crud(rename = "full_name")]        // Database column: full_name
///     name: String,                        // Rust field: name
///     
///     #[crud(rename = "email_address")]    // Database column: email_address  
///     email: String,                       // Rust field: email
///     
///     #[crud(rename = "birth_date")]       // Database column: birth_date
///     birthday: Option<chrono::NaiveDate>, // Rust field: birthday
///     
///     // Field without rename uses the same name
///     is_active: bool,
/// }
/// ```
///
/// ### Multi-Database Support
/// ```rust
/// // Same struct, different databases
/// #[derive(FromRow, CrudOpsRef)]
/// #[crud(table = "products", db = "mysql")]
/// struct MySQLProduct {
///     #[crud(primary_key)]
///     id: Option<i64>,
///     name: String,
///     price: f64,
/// }
///
/// #[derive(FromRow, CrudOpsRef)]
/// #[crud(table = "products", db = "sqlite")]
/// struct SQLiteProduct {
///     #[crud(primary_key)]
///     id: Option<i64>,
///     name: String,
///     price: f64,
/// }
/// ```
///
/// ### Complex Types and Custom Fields
/// ```rust
/// use serde_json::Value;
///
/// #[derive(FromRow, CrudOpsRef)]
/// #[crud(table = "advanced_users", db = "postgres")]
/// struct AdvancedUser {
///     #[crud(primary_key)]
///     id: Option<Uuid>,
///     
///     name: String,
///     
///     #[crud(rename = "metadata")]
///     user_metadata: Value,  // JSON field
///     
///     #[crud(rename = "tags")]
///     user_tags: Vec<String>,  // Array field (PostgreSQL)
///     
///     created_at: chrono::DateTime<chrono::Utc>,
///     updated_at: Option<chrono::DateTime<chrono::Utc>>,
/// }
/// ```
///
/// ## 🔑 Primary Key Handling
/// The macro intelligently handles primary keys:
///
/// 1. **Explicit marking**: Use `#[crud(primary_key)]` on any field
/// 2. **Automatic detection**: If no field is marked, the first field is used
/// 3. **Option unwrapping**: `Option<T>` fields are automatically unwrapped to `T` for key operations
/// 4. **Type safety**: The key type is properly extracted for all CRUD operations
///
/// ```rust
/// // These are equivalent for key operations:
/// struct User1 { id: i64, name: String }      // Key type: i64
/// struct User2 { id: Option<i64>, name: String }  // Key type: i64 (unwrapped)
/// ```
///
/// ## ⚠️ Error Handling
/// All generated methods return `Result<T, sqlx::Error>`. Common error scenarios:
/// - **Constraint violations**: Unique key, foreign key, check constraints
/// - **Connection errors**: Database unavailable, timeout
/// - **Type conversion errors**: Incompatible Rust ↔ SQL type mapping
/// - **SQL syntax errors**: Invalid table/column names
///
/// ## 🔧 Troubleshooting
///
/// ### Common Issues:
/// 1. **"trait bound not satisfied"**: Ensure your types implement `sqlx::Encode + sqlx::Type`
/// 2. **"column not found"**: Check `#[crud(rename = "...")]` matches database schema
/// 3. **"table not found"**: Verify `#[crud(table = "...")]` matches actual table name
/// 4. **Type conversion errors**: Ensure Rust types match database column types
///
/// ### Debugging Tips:
/// ```rust
/// // Use cargo expand to see generated code
/// // cargo install cargo-expand
/// // cargo expand --lib
/// ```
#[proc_macro_derive(CrudOpsRef, attributes(crud))]
pub fn derive_crud_ops_ref(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();

    // Parse table name from struct attributes
    let table_name = parse_table_name(&input.attrs, &struct_name_str);

    // Parse fields
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("CrudOpsRef can only be derived for structs with named fields"),
        },
        _ => panic!("CrudOpsRef can only be derived for structs"),
    };

    // Find primary key
    let mut primary_key_field = None;
    let mut primary_key_type = None;

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        // Check if this field is marked as primary key
        if has_primary_key_attr(&field.attrs) {
            primary_key_field = Some(field_name.to_string());
            primary_key_type = Some(field_type.clone());
            break;
        }
    }

    // Default to first field if no primary key is marked
    let (primary_key_field, primary_key_type) =
        if let (Some(field), Some(typ)) = (primary_key_field, primary_key_type) {
            let pk_ty = extract_option_inner_type_deep(&typ);
            (field, pk_ty.clone())
        } else {
            let first_field = fields
                .iter()
                .next()
                .expect("Struct must have at least one field");
            let field_name = first_field.ident.as_ref().unwrap().to_string();
            let field_type = &first_field.ty;
            let pk_ty = extract_option_inner_type_deep(field_type);
            (field_name, pk_ty.clone())
        };

    // Generate field idents, field names, and placeholders
    let field_idents: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
    let field_names: Vec<String> = fields
        .iter()
        .map(|f| get_crud_rename(&f.attrs).unwrap_or_else(|| f.ident.as_ref().unwrap().to_string()))
        .collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    let non_pk_idents: Vec<_> = fields
        .iter()
        .filter(|f| f.ident.as_ref().unwrap().to_string() != primary_key_field)
        .map(|f| f.ident.as_ref().unwrap())
        .collect();
    let non_pk_names: Vec<String> = fields
        .iter()
        .filter(|f| f.ident.as_ref().unwrap().to_string() != primary_key_field)
        .map(|f| get_crud_rename(&f.attrs).unwrap_or_else(|| f.ident.as_ref().unwrap().to_string()))
        .collect();

    let db_type = parse_db_type(&input.attrs);

    let expanded = match db_type.as_str() {
        "postgres" => {
            let pg_placeholders: Vec<String> =
                (1..=field_names.len()).map(|i| format!("${}", i)).collect();
            let pg_placeholders_tokens: Vec<_> = pg_placeholders
                .iter()
                .map(|s| syn::LitStr::new(s, proc_macro2::Span::call_site()))
                .collect();
            let pg_set_exprs: Vec<String> = non_pk_names
                .iter()
                .enumerate()
                .map(|(i, name)| format!("{} = ${}", name, i + 1))
                .collect();
            let pg_set_sql = pg_set_exprs.join(", ");
            let update_pk_index = non_pk_names.len() + 1;

            quote! {
                impl<DB> typed_sqlx_client::CrudOpsRef<#primary_key_type, #struct_name>
                    for typed_sqlx_client::SqlTable<sqlx::Postgres, DB, #struct_name>
                where
                    DB: Send + Sync,
                    #struct_name: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Sync,
                    for<'a> &'a str: sqlx::ColumnIndex<sqlx::postgres::PgRow>,
                    sqlx::postgres::PgArguments: for<'q> sqlx::IntoArguments<'q, sqlx::Postgres>,
                    for<'c> &'c sqlx::Pool<sqlx::Postgres>: sqlx::Executor<'c, Database = sqlx::Postgres>,
                    #(
                        #field_types: for<'r> sqlx::Encode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
                    )*
                {
                    type Error = sqlx::Error;

                    fn delete_by_id(&self, id: &#primary_key_type) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                        async move {
                            let sql = format!("DELETE FROM {} WHERE {} = $1", #table_name, #primary_key_field);
                            sqlx::query(&sql).bind(id).execute(self.get_pool()).await?;
                            Ok(())
                        }
                    }

                    fn get_by_id(&self, id: &#primary_key_type) -> impl std::future::Future<Output = Result<Option<#struct_name>, Self::Error>> + Send {
                        async move {
                            let sql = format!("SELECT * FROM {} WHERE {} = $1", #table_name, #primary_key_field);
                            let result = sqlx::query_as::<sqlx::Postgres, #struct_name>(&sql)
                                .bind(id)
                                .fetch_optional(self.get_pool())
                                .await?;
                            Ok(result)
                        }
                    }

                    fn insert(&self, entity: &#struct_name) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                        async move {
                            let fields = [#(#field_names),*].join(", ");
                            let placeholders = [#(#pg_placeholders_tokens),*].join(", ");
                            let sql = format!(
                                "INSERT INTO {} ({}) VALUES ({})",
                                #table_name,
                                fields,
                                placeholders
                            );
                            let mut query = sqlx::query(&sql);
                            #(
                                query = query.bind(&entity.#field_idents);
                            )*
                            query.execute(self.get_pool()).await?;
                            Ok(())
                        }
                    }

                    fn update_by_id(&self, id: &#primary_key_type, entity: &#struct_name) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                        async move {
                            let sql = format!(
                                "UPDATE {} SET {} WHERE {} = ${}",
                                #table_name,
                                #pg_set_sql,
                                #primary_key_field,
                                #update_pk_index
                            );
                            let mut query = sqlx::query(&sql);
                            #(
                                query = query.bind(&entity.#non_pk_idents);
                            )*
                            query = query.bind(id);
                            query.execute(self.get_pool()).await?;
                            Ok(())
                        }
                    }

                    fn insert_batch(&self, entities: &[#struct_name]) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                        async move {
                            let fields = [#(#field_names),*].join(", ");
                            let placeholders = [#(#pg_placeholders_tokens),*].join(", ");
                            for entity in entities {
                                let sql = format!(
                                    "INSERT INTO {} ({}) VALUES ({})",
                                    #table_name,
                                    fields,
                                    placeholders
                                );
                                let mut query = sqlx::query(&sql);
                                #(
                                    query = query.bind(&entity.#field_idents);
                                )*
                                query.execute(self.get_pool()).await?;
                            }
                            Ok(())
                        }
                    }
                }
            }
        }
        "sqlite" => {
            let placeholders: Vec<_> = (0..field_names.len()).map(|_| "?").collect::<Vec<_>>();
            let set_exprs: Vec<_> = non_pk_names
                .iter()
                .map(|name| format!("{} = ?", name))
                .collect();
            let set_sql = set_exprs.join(", ");
            quote! {
                impl<DB> typed_sqlx_client::CrudOpsRef<#primary_key_type, #struct_name>
                    for typed_sqlx_client::SqlTable<sqlx::Sqlite, DB, #struct_name>
                where
                    DB: Send + Sync,
                    #struct_name: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Sync,
                    for<'a> &'a str: sqlx::ColumnIndex<sqlx::sqlite::SqliteRow>,
                    for<'q> sqlx::sqlite::SqliteArguments<'q>: sqlx::IntoArguments<'q, sqlx::Sqlite>,
                    for<'c> &'c sqlx::Pool<sqlx::Sqlite>: sqlx::Executor<'c, Database = sqlx::Sqlite>,
                    #(
                        #field_types: for<'r> sqlx::Encode<'r, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite>,
                    )*
                {
                    type Error = sqlx::Error;

                    fn delete_by_id(&self, id: &#primary_key_type) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                        async move {
                            let sql = format!("DELETE FROM {} WHERE {} = ?", #table_name, #primary_key_field);
                            sqlx::query(&sql).bind(id).execute(self.get_pool()).await?;
                            Ok(())
                        }
                    }

                    fn get_by_id(&self, id: &#primary_key_type) -> impl std::future::Future<Output = Result<Option<#struct_name>, Self::Error>> + Send {
                        async move {
                            let sql = format!("SELECT * FROM {} WHERE {} = ?", #table_name, #primary_key_field);
                            let result = sqlx::query_as::<sqlx::Sqlite, #struct_name>(&sql)
                                .bind(id)
                                .fetch_optional(self.get_pool())
                                .await?;
                            Ok(result)
                        }
                    }

                    fn insert(&self, entity: &#struct_name) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                        async move {
                            let fields = [#(#field_names),*].join(", ");
                            let placeholders = [#(#placeholders),*].join(", ");
                            let sql = format!(
                                "INSERT INTO {} ({}) VALUES ({})",
                                #table_name,
                                fields,
                                placeholders
                            );
                            let mut query = sqlx::query(&sql);
                            #(
                                query = query.bind(&entity.#field_idents);
                            )*
                            query.execute(self.get_pool()).await?;
                            Ok(())
                        }
                    }

                    fn update_by_id(&self, id: &#primary_key_type, entity: &#struct_name) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                        async move {
                            let sql = format!(
                                "UPDATE {} SET {} WHERE {} = ?",
                                #table_name,
                                #set_sql,
                                #primary_key_field
                            );
                            let mut query = sqlx::query(&sql);
                            #(
                                query = query.bind(&entity.#non_pk_idents);
                            )*
                            query = query.bind(id);
                            query.execute(self.get_pool()).await?;
                            Ok(())
                        }
                    }

                    fn insert_batch(&self, entities: &[#struct_name]) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                        async move {
                            let fields = [#(#field_names),*].join(", ");
                            let placeholders = [#(#placeholders),*].join(", ");
                            for entity in entities {
                                let sql = format!(
                                    "INSERT INTO {} ({}) VALUES ({})",
                                    #table_name,
                                    fields,
                                    placeholders
                                );
                                let mut query = sqlx::query(&sql);
                                #(
                                    query = query.bind(&entity.#field_idents);
                                )*
                                query.execute(self.get_pool()).await?;
                            }
                            Ok(())
                        }
                    }
                }
            }
        }
        _ => {
            // default to MySQL
            let placeholders: Vec<_> = (0..field_names.len()).map(|_| "?").collect::<Vec<_>>();
            let set_exprs: Vec<_> = non_pk_names
                .iter()
                .map(|name| format!("{} = ?", name))
                .collect();
            let set_sql = set_exprs.join(", ");
            quote! {
                impl<DB> typed_sqlx_client::CrudOpsRef<#primary_key_type, #struct_name>
                    for typed_sqlx_client::SqlTable<sqlx::MySql, DB, #struct_name>
                where
                    DB: Send + Sync,
                    #struct_name: for<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> + Send + Sync,
                    for<'a> &'a str: sqlx::ColumnIndex<sqlx::mysql::MySqlRow>,
                    sqlx::mysql::MySqlArguments: for<'q> sqlx::IntoArguments<'q, sqlx::MySql>,
                    for<'c> &'c sqlx::Pool<sqlx::MySql>: sqlx::Executor<'c, Database = sqlx::MySql>,
                    #(
                        #field_types: for<'r> sqlx::Encode<'r, sqlx::MySql> + sqlx::Type<sqlx::MySql>,
                    )*
                {
                    type Error = sqlx::Error;

                    fn delete_by_id(&self, id: &#primary_key_type) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                        async move {
                            let sql = format!("DELETE FROM {} WHERE {} = ?", #table_name, #primary_key_field);
                            sqlx::query(&sql).bind(id).execute(self.get_pool()).await?;
                            Ok(())
                        }
                    }

                    fn get_by_id(&self, id: &#primary_key_type) -> impl std::future::Future<Output = Result<Option<#struct_name>, Self::Error>> + Send {
                        async move {
                            let sql = format!("SELECT * FROM {} WHERE {} = ?", #table_name, #primary_key_field);
                            let result = sqlx::query_as::<sqlx::MySql, #struct_name>(&sql)
                                .bind(id)
                                .fetch_optional(self.get_pool())
                                .await?;
                            Ok(result)
                        }
                    }

                    fn insert(&self, entity: &#struct_name) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                        async move {
                            let fields = [#(#field_names),*].join(", ");
                            let placeholders = [#(#placeholders),*].join(", ");
                            let sql = format!(
                                "INSERT INTO {} ({}) VALUES ({})",
                                #table_name,
                                fields,
                                placeholders
                            );
                            let mut query = sqlx::query(&sql);
                            #(
                                query = query.bind(&entity.#field_idents);
                            )*
                            query.execute(self.get_pool()).await?;
                            Ok(())
                        }
                    }

                    fn update_by_id(&self, id: &#primary_key_type, entity: &#struct_name) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                        async move {
                            let sql = format!(
                                "UPDATE {} SET {} WHERE {} = ?",
                                #table_name,
                                #set_sql,
                                #primary_key_field
                            );
                            let mut query = sqlx::query(&sql);
                            #(
                                query = query.bind(&entity.#non_pk_idents);
                            )*
                            query = query.bind(id);
                            query.execute(self.get_pool()).await?;
                            Ok(())
                        }
                    }

                    fn insert_batch(&self, entities: &[#struct_name]) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                        async move {
                            let fields = [#(#field_names),*].join(", ");
                            let placeholders = [#(#placeholders),*].join(", ");
                            for entity in entities {
                                let sql = format!(
                                    "INSERT INTO {} ({}) VALUES ({})",
                                    #table_name,
                                    fields,
                                    placeholders
                                );
                                let mut query = sqlx::query(&sql);
                                #(
                                    query = query.bind(&entity.#field_idents);
                                )*
                                query.execute(self.get_pool()).await?;
                            }
                            Ok(())
                        }
                    }
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn parse_db_type(attrs: &[syn::Attribute]) -> String {
    for attr in attrs {
        if attr.path().is_ident("crud") {
            let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
            if let Ok(meta_list) = attr.parse_args_with(parser) {
                for meta in meta_list {
                    if let syn::Meta::NameValue(nv) = meta {
                        if nv.path.is_ident("db") {
                            if let syn::Expr::Lit(expr_lit) = &nv.value {
                                if let syn::Lit::Str(litstr) = &expr_lit.lit {
                                    return litstr.value();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    "mysql".to_string()
}

fn get_crud_rename(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("crud") {
            let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
            if let Ok(meta_list) = attr.parse_args_with(parser) {
                for meta in meta_list {
                    if let syn::Meta::NameValue(nv) = meta {
                        if nv.path.is_ident("rename") {
                            if let syn::Expr::Lit(expr_lit) = &nv.value {
                                if let syn::Lit::Str(litstr) = &expr_lit.lit {
                                    return Some(litstr.value());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn parse_table_name(attrs: &[syn::Attribute], default: &str) -> String {
    for attr in attrs {
        if attr.path().is_ident("crud") {
            let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
            if let Ok(meta_list) = attr.parse_args_with(parser) {
                for meta in meta_list {
                    if let syn::Meta::NameValue(nv) = meta {
                        if nv.path.is_ident("table") {
                            if let syn::Expr::Lit(expr_lit) = &nv.value {
                                if let syn::Lit::Str(litstr) = &expr_lit.lit {
                                    return litstr.value();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    default.to_string()
}

// Helper function to check if field has primary_key attribute
fn has_primary_key_attr(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("crud") {
            if let Meta::List(meta_list) = &attr.meta {
                let tokens_str = meta_list.tokens.to_string();
                if tokens_str.contains("primary_key") {
                    return true;
                }
            }
        }
    }
    false
}

fn extract_option_inner_type_deep(ty: &syn::Type) -> &syn::Type {
    let mut t = ty;
    loop {
        if let syn::Type::Path(type_path) = t {
            if let Some(seg) = type_path.path.segments.first() {
                if seg.ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            t = inner_ty;
                            continue;
                        }
                    }
                }
            }
        }
        break;
    }
    t
}
