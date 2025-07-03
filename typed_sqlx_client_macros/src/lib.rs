use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Meta};

/// Derive macro for ToRow trait
///
/// # Attributes
/// - `#[torow(table = "table_name")]` - Specify table name (defaults to struct name)
/// - `#[torow(primary_key)]` - Mark field as primary key
/// - `#[torow(rename = "column_name")]` - Rename field to different column name
///
/// # Example
/// ```rust
/// #[derive(ToRow)]
/// #[torow(table = "users")]
/// struct User {
///     #[torow(primary_key)]
///     id: i64,
///     #[torow(rename = "user_name")]
///     name: String,
///     email: String,
/// }
/// ```
#[proc_macro_derive(ToRow, attributes(torow))]
pub fn derive_to_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();

    // Parse table name from struct attributes
    let table_name = parse_table_name(&input.attrs, &struct_name_str);

    // Parse fields
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("ToRow can only be derived for structs with named fields"),
        },
        _ => panic!("ToRow can only be derived for structs"),
    };

    // Find primary key and build field mappings
    let mut primary_key_field = None;
    let mut field_mappings = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        // Check if this field is marked as primary key
        let is_primary_key = has_primary_key_attr(&field.attrs);
        if is_primary_key {
            primary_key_field = Some(field_name_str.clone());
        }

        // Get column name (either renamed or same as field name)
        let column_name = parse_column_name(&field.attrs, &field_name_str);

        field_mappings.push((field_name_str, column_name));
    }

    let primary_key_field = primary_key_field.unwrap_or_else(|| "id".to_string());

    // Generate the field mappings as tokens
    let mapping_tokens = field_mappings.iter().map(|(field, column)| {
        quote! { (#field, #column) }
    });

    // Debug output to see what we're generating
    eprintln!(
        "Generating ToRow for {}: table={}, pk={}",
        struct_name, table_name, primary_key_field
    );

    let expanded = quote! {
        impl typed_sqlx_client::ToRow for #struct_name {
            const TABLE_NAME: &'static str = #table_name;
            const PRIMARY_KEY_FIELD: &'static str = #primary_key_field;

            fn field_column_mappings() -> Vec<(&'static str, &'static str)> {
                vec![#(#mapping_tokens),*]
            }
        }
    };

    TokenStream::from(expanded)
}

// Helper function to parse table name from struct attributes
fn parse_table_name(attrs: &[Attribute], default: &str) -> String {
    for attr in attrs {
        if attr.path().is_ident("torow") {
            // Direct match for Meta::List, no need for Ok()
            if let Meta::List(meta_list) = &attr.meta {
                let tokens_str = meta_list.tokens.to_string();
                eprintln!("Parsing torow attribute: {}", tokens_str);

                // Simple string parsing approach
                if tokens_str.contains("table") {
                    // Find content within quotes
                    if let Some(start) = tokens_str.find('"') {
                        if let Some(end) = tokens_str.rfind('"') {
                            if start < end {
                                let table_name = &tokens_str[start + 1..end];
                                eprintln!("Found table name: {}", table_name);
                                return table_name.to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    eprintln!("Using default table name: {}", default);
    default.to_string()
}

// Helper function to check if field has primary_key attribute
fn has_primary_key_attr(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("torow") {
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

// Helper function to parse column name from field attributes
fn parse_column_name(attrs: &[Attribute], default: &str) -> String {
    for attr in attrs {
        if attr.path().is_ident("torow") {
            if let Meta::List(meta_list) = &attr.meta {
                let tokens_str = meta_list.tokens.to_string();
                if tokens_str.contains("rename") {
                    // Find content within quotes
                    if let Some(start) = tokens_str.find('"') {
                        if let Some(end) = tokens_str.rfind('"') {
                            if start < end {
                                let column_name = &tokens_str[start + 1..end];
                                return column_name.to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    default.to_string()
}
