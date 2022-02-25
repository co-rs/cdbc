use proc_macro2::{Ident, TokenStream};
use quote::quote;
use quote::ToTokens;

/// db_type: cdbc_sqlite::SqliteRow
pub(crate) fn impl_scan(ast: &syn::DeriveInput,db_type:&[TokenStream]) -> crate::proc_macro::TokenStream {
    let name = &ast.ident;
    let field_idents = gen_fields(&ast.data);
    let mut fields = quote! {};
    for x in &field_idents {
        fields = quote! {#fields #x:None,};
    }
    let mut get_matchs = quote! {
    };
    for db_type in db_type {
        get_matchs = quote! {
            #get_matchs
            cdbc::impl_scan!(#db_type,#name{#fields});
        }
    }
    get_matchs.into()
}

fn gen_fields(data: &syn::Data) -> Vec<Ident> {
    let mut fields = vec![];
    match &data {
        syn::Data::Struct(s) => {
            for field in &s.fields {
                match &field.ident {
                    None => {}
                    Some(v) => {
                        fields.push(v.clone());
                    }
                }
            }
        }
        _ => {
            panic!("[rbatis] #[crud_table] only support struct for crud_table's macro!")
        }
    }
    fields
}