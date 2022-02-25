#![allow(unused_assignments)]
extern crate proc_macro;

mod scan;

use std::fs::File;
use std::io::Read;
use quote::quote;
use crate::proc_macro::TokenStream;


#[proc_macro_derive(Scan)]
pub fn macro_derive_scan_all(input: TokenStream) -> TokenStream {
    let mut cargo_data = "".to_string();
    let mut f = File::open("Cargo.toml").unwrap();
    f.read_to_string(&mut cargo_data).unwrap();
    println!("read Cargo.toml: {}", cargo_data);

    let mut database = vec![];
    if cargo_data.contains("cdbc-mysql"){
        database.push(quote!(cdbc_mysql::MySqlRow));
    }
    if cargo_data.contains("cdbc-pg"){
        database.push(quote!(cdbc_pg::PgRow));
    }
    if cargo_data.contains("cdbc-sqlite"){
        database.push(quote!(cdbc_sqlite::SqliteRow));
    }
    if cargo_data.contains("cdbc-mssql"){
        database.push(quote!(cdbc_mssql::MssqlRow));
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