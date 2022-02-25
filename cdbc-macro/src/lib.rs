#![allow(unused_assignments)]
extern crate proc_macro;

mod scan;

use crate::proc_macro::TokenStream;


#[proc_macro_derive(Scan)]
pub fn macro_derive_scan(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let stream = scan::impl_scan(&ast);
    #[cfg(feature = "debug_mode")]
    {
        println!("............gen impl Scan:\n {}", stream);
        println!("............gen impl Scan end............");
    }
    stream
}
