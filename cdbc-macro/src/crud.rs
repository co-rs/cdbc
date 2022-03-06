use proc_macro2::{Ident, TokenStream};
use quote::quote;
use quote::ToTokens;

pub(crate) fn impl_crud(ast: &syn::DeriveInput, db_type:&[TokenStream]) -> crate::proc_macro::TokenStream {
    let mut stream = quote! {
    };

    stream.into()
}