#![allow(unused_assignments)]
extern crate proc_macro;

mod scan;

use quote::quote;
use crate::proc_macro::TokenStream;


#[proc_macro_derive(ScanAll)]
pub fn macro_derive_scan_all(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let stream = scan::impl_scan(&ast,&[
        quote!(cdbc_pg::PgRow),
        quote!(cdbc_sqlite::SqliteRow),
        quote!(cdbc_mysql::MySqlRow),
        quote!(cdbc_mssql::MssqlRow)
    ]);
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
    let stream = scan::impl_scan(&ast,&[quote!(cdbc_sqlite::SqliteRow)]);
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
    let stream = scan::impl_scan(&ast,&[quote!(cdbc_mssql::MssqlRow)]);
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
    let stream = scan::impl_scan(&ast,&[quote!(cdbc_mysql::MySqlRow)]);
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
    let stream = scan::impl_scan(&ast,&[quote!(cdbc_pg::PgRow)]);
    #[cfg(feature = "debug_mode")]
    {
        println!("............gen impl Scan:\n {}", stream);
        println!("............gen impl Scan end............");
    }
    stream
}