#![allow(unused_assignments)]
extern crate proc_macro;

mod scan;
mod crud;
use std::fs::File;
use std::io::Read;
use quote::quote;
use crate::proc_macro::TokenStream;

/// this Scan will find on Cargo.toml database driver to impl cdbc::impl_scan!(#db_type,#name{#fields});
#[proc_macro_derive(Scan)]
pub fn macro_derive_scan_all(input: TokenStream) -> TokenStream {
    let mut cargo_data = "".to_string();
    let mut f = File::open("Cargo.lock").unwrap();
    f.read_to_string(&mut cargo_data).unwrap();
    drop(f);

    let mut database = vec![];
    for line in cargo_data.lines() {
        if line.trim_start_matches(r#"name = ""#).starts_with("cdbc-mysql") {
            database.push(quote!(cdbc_mysql::MySqlRow));
        }
        if line.trim_start_matches(r#"name = ""#).starts_with("cdbc-pg") {
            database.push(quote!(cdbc_pg::PgRow));
        }
        if line.trim_start_matches(r#"name = ""#).starts_with("cdbc-sqlite") {
            database.push(quote!(cdbc_sqlite::SqliteRow));
        }
        if line.trim_start_matches(r#"name = ""#).starts_with("cdbc-mssql") {
            database.push(quote!(cdbc_mssql::MssqlRow));
        }
    }

    let ast = syn::parse(input).unwrap();
    let stream = scan::impl_scan(&ast, &database);
    #[cfg(feature = "debug_mode")]
    {
        println!("............gen impl Scan:\n {}", stream);
        println!("............gen impl Scan end............");
    }
    stream
}

#[proc_macro_derive(ScanSqlite)]
pub fn macro_derive_scan_sqlite(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let stream = scan::impl_scan(&ast, &[quote!(cdbc_sqlite::SqliteRow)]);
    #[cfg(feature = "debug_mode")]
    {
        println!("............gen impl Scan:\n {}", stream);
        println!("............gen impl Scan end............");
    }
    stream
}

#[proc_macro_derive(ScanMssql)]
pub fn macro_derive_scan_mssql(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let stream = scan::impl_scan(&ast, &[quote!(cdbc_mssql::MssqlRow)]);
    #[cfg(feature = "debug_mode")]
    {
        println!("............gen impl Scan:\n {}", stream);
        println!("............gen impl Scan end............");
    }
    stream
}

#[proc_macro_derive(ScanMysql)]
pub fn macro_derive_scan_mysql(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let stream = scan::impl_scan(&ast, &[quote!(cdbc_mysql::MySqlRow)]);
    #[cfg(feature = "debug_mode")]
    {
        println!("............gen impl Scan:\n {}", stream);
        println!("............gen impl Scan end............");
    }
    stream
}

#[proc_macro_derive(ScanPg)]
pub fn macro_derive_scan_pg(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let stream = scan::impl_scan(&ast, &[quote!(cdbc_pg::PgRow)]);
    #[cfg(feature = "debug_mode")]
    {
        println!("............gen impl Scan:\n {}", stream);
        println!("............gen impl Scan end............");
    }
    stream
}


#[proc_macro_attribute]
pub fn crud(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut cargo_data = "".to_string();
    let mut f = File::open("Cargo.lock").unwrap();
    f.read_to_string(&mut cargo_data).unwrap();
    drop(f);
    let mut database = vec![];
    for line in cargo_data.lines() {
        if line.trim_start_matches(r#"name = ""#).starts_with("cdbc-mysql") {
            database.push(vec![quote!(cdbc_mysql::MySqlPool),quote!(cdbc_mysql::MySqlConnection),quote!(cdbc::Transaction::<'_,cdbc_mysql::MySql>)]);
        }
        if line.trim_start_matches(r#"name = ""#).starts_with("cdbc-pg") {
            database.push(vec![quote!(cdbc_pg::PgPool),quote!(cdbc_pg::PgConnection),quote!(cdbc::Transaction::<'_,cdbc_pg::Postgres>)]);
        }
        if line.trim_start_matches(r#"name = ""#).starts_with("cdbc-sqlite") {
            database.push(vec![quote!(cdbc_sqlite::SqlitePool),quote!(cdbc_sqlite::SqliteConnection),quote!(cdbc::Transaction::<'_,cdbc_sqlite::Sqlite>)]);
        }
        if line.trim_start_matches(r#"name = ""#).starts_with("cdbc-mssql") {
            database.push(vec![quote!(cdbc_mssql::MssqlPool),quote!(cdbc_mssql::MssqlConnection),quote!(cdbc::Transaction::<'_,cdbc_mssql::Mssql>)]);
        }
    }
    let stream = crud::impl_crud(input, database);
    #[cfg(feature = "debug_mode")]
    {
        println!("............gen crud:\n {}", stream);
        println!("............gen crud end............");
    }
    stream
}