use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Meta};

/// Derive macro for CrudOpsRef trait
///
/// # Attributes
/// - `#[crud(table = "table_name")]` - Specify table name (defaults to struct name)
/// - `#[crud(primary_key)]` - Mark field as primary key
/// - `#[crud(rename = "column_name")]` - Rename field to different column name
///
/// # Example
/// ```rust
/// #[derive(CrudOpsRef, sqlx::FromRow)]
/// #[crud(table = "users")]
/// struct User {
///     #[crud(primary_key)]
///     id: i64,
///     #[crud(rename = "user_name")]
///     name: String,
///     email: String,
/// }
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
        let field_name_str = field_name.to_string();
        let field_type = &field.ty;

        // Check if this field is marked as primary key
        if has_primary_key_attr(&field.attrs) {
            primary_key_field = Some(field_name_str.clone());
            primary_key_type = Some(field_type.clone());
            break;
        }
    }

    // Default to first field if no primary key is marked
    let (primary_key_field, primary_key_type) =
        if let (Some(field), Some(typ)) = (primary_key_field, primary_key_type) {
            (field, typ)
        } else {
            // Use first field as default primary key
            let first_field = fields
                .iter()
                .next()
                .expect("Struct must have at least one field");
            let field_name = first_field.ident.as_ref().unwrap().to_string();
            let field_type = &first_field.ty;
            (field_name, field_type.clone())
        };

    // Generate field idents, field names, and placeholders
    let field_idents: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
    let field_names: Vec<_> = field_idents.iter().map(|ident| ident.to_string()).collect();
    let placeholders: Vec<_> = (0..field_names.len()).map(|_| "?").collect::<Vec<_>>();
    // Collect all field types
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
    // Generate non-primary key fields
    let non_pk_idents: Vec<_> = fields
        .iter()
        .filter(|f| f.ident.as_ref().unwrap().to_string() != primary_key_field)
        .map(|f| f.ident.as_ref().unwrap())
        .collect();
    let non_pk_names: Vec<_> = non_pk_idents
        .iter()
        .map(|ident| ident.to_string())
        .collect();
    let set_exprs: Vec<_> = non_pk_names
        .iter()
        .map(|name| format!("{} = ?", name))
        .collect();
    let set_sql = set_exprs.join(", ");

    // Generate simplified implementation to let the compiler check constraints at use sites
    let expanded = quote! {
        impl<P, DB> typed_sqlx_client::CrudOpsRef<#primary_key_type, #struct_name>
            for typed_sqlx_client::SqlTable<P, DB, #struct_name>
        where
            P: sqlx::Database,
            DB: Send + Sync,
            #struct_name: for<'r> sqlx::FromRow<'r, P::Row> + Send + Sync,
            str: sqlx::ColumnIndex<P::Row>,
            for<'q> P::Arguments<'q>: sqlx::IntoArguments<'q, P>,
            for<'c> &'c sqlx::Pool<P>: sqlx::Executor<'c, Database = P>,
            // Add Encode/Type bounds for all field types
            #(
                #field_types: for<'r> sqlx::Encode<'r, P> + sqlx::Type<P>,
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
                    let result = sqlx::query_as::<P, #struct_name>(&sql)
                        .bind(id)
                        .fetch_optional(self.get_pool())
                        .await?;
                    Ok(result)
                }
            }

            fn insert(&self, entity: &#struct_name) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                async move {
                    let sql = concat!(
                        "INSERT INTO ",
                        #table_name,
                        " (",
                        #(#field_names),*,
                        ") VALUES (",
                        #(#placeholders),*,
                        ")"
                    );
                    let mut query = sqlx::query(sql);
                    #(
                        query = query.bind(&entity.#field_idents);
                    )*
                    query.execute(self.get_pool()).await?;
                    Ok(())
                }
            }

            fn update_by_id(&self, id: &#primary_key_type, entity: &#struct_name) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                async move {
                    let sql = concat!(
                        "UPDATE ",
                        #table_name,
                        " SET ",
                        #set_sql,
                        " WHERE ",
                        #primary_key_field,
                        " = ?"
                    );
                    let mut query = sqlx::query(sql);
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
                    let sql = concat!(
                        "INSERT INTO ",
                        #table_name,
                        " (",
                        #(#field_names),*,
                        ") VALUES (",
                        #(#placeholders),*,
                        ")"
                    );
                    for entity in entities {
                        let mut query = sqlx::query(sql);
                        #(
                            query = query.bind(&entity.#field_idents);
                        )*
                        query.execute(self.get_pool()).await?;
                    }
                    Ok(())
                }
            }
        }
    };

    TokenStream::from(expanded)
}

// Helper function to parse table name from struct attributes
fn parse_table_name(attrs: &[Attribute], default: &str) -> String {
    for attr in attrs {
        if attr.path().is_ident("crud") {
            if let Meta::List(meta_list) = &attr.meta {
                let tokens_str = meta_list.tokens.to_string();

                if tokens_str.contains("table") {
                    // Find content within quotes
                    if let Some(start) = tokens_str.find('"') {
                        if let Some(end) = tokens_str.rfind('"') {
                            if start < end {
                                let table_name = &tokens_str[start + 1..end];
                                return table_name.to_string();
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
