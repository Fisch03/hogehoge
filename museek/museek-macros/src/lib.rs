extern crate proc_macro;
use self::proc_macro::TokenStream;

use quote::quote;

use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

fn dollar_values(max: usize) -> String {
    let itr = 1..max + 1;
    itr.into_iter()
        .map(|s| format!("${}", s))
        .collect::<Vec<String>>()
        .join(",")
}

#[proc_macro_derive(InsertRow, attributes(table))]
pub fn derive_from_struct_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let table_name = input
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.path().is_ident("table") {
                attr.parse_args::<syn::LitStr>().ok()
            } else {
                None
            }
        })
        .expect("expected #[table = \"...\"] attribute")
        .value();

    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };
    // COMMON Atrributes
    let struct_name = &input.ident;

    // INSERT Attributes -> field names
    let attributes = fields.iter().map(|field| &field.ident);
    let attributes_vec: Vec<String> = fields
        .iter()
        .map(|field| {
            field
                .ident
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default()
        })
        .collect();

    // ( id, name, hostname .. )
    let columns = attributes_vec.join(",");
    // ( $1, $2)
    let dollars = dollar_values(attributes_vec.len());

    // // UPDATE Attributes -> field names for
    // let attributes_update = fields.iter().map(|field| &field.ident);
    // // name = $2, hostname = $3
    // let pairs: String = attributes_vec
    //     .iter()
    //     .enumerate()
    //     .skip(1) // Skip the first element
    //     .map(|(index, value)| {
    //         let number = index + 1; // Start with $2
    //         format!("{} = ${}", value, number)
    //     })
    //     .collect::<Vec<String>>()
    //     .join(",");

    let sql = format!("INSERT INTO {table_name} ( {columns} ) VALUES ( {dollars} )",);

    TokenStream::from(quote! {

        impl #struct_name {
            // pub fn insert(&self, table: &str) -> String
            // {
            //     let sqlquery = format!("insert into {} ( {} ) values ( {} )", table, #columns, #dollars);
            //     sqlquery
            // }

            pub async fn insert(&self, pool: &sqlx::SqlitePool) -> sqlx::Result<sqlx::sqlite::SqliteQueryResult> {
                sqlx::query!(#sql, #(self.#attributes),*).execute(pool).await
            }
        }
    })
}
