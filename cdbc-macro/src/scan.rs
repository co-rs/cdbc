use proc_macro2::{Ident};
use quote::quote;
use quote::ToTokens;

pub(crate) fn impl_scan(ast: &syn::DeriveInput) -> crate::proc_macro::TokenStream {
    let name = &ast.ident;
    let field_idents = gen_fields(&ast.data);
    let mut fields = quote! {};
    for x in &field_idents {
        fields = quote! {#fields #x:None,};
    }
    let get_matchs = quote! {
        cdbc::impl_scan!(SqliteRow,#name{#fields});
    };
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