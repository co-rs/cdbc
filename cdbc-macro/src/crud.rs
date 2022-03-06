use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use quote::ToTokens;
use syn::DeriveInput;

pub(crate) fn impl_crud(input: crate::proc_macro::TokenStream, db_type: Vec<Vec<TokenStream>>) -> crate::proc_macro::TokenStream {
    let driver_token = gen_driver_token(input.to_string());
    let ast:DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let field_idents = gen_fields(&ast.data);
    let table_name = to_snake_name(&name.to_string());
    let columns = gen_columns(&field_idents);
    let mut stream = quote! {
        #driver_token
        #ast
        impl cdbc::crud::Table for #name {
            fn table() -> &'static str {
                #table_name
            }

            fn columns() -> &'static [&'static str] {
                &[#columns]
            }
        }
    };
    for types in db_type {
        for t in types {
            let mut crud = do_impl_curd(name, &t);
            stream = quote!{#stream #crud};
        }
    }
   stream.into()
}

fn to_snake_name(name: &str) -> String {
    let chs = name.chars();
    let mut new_name = String::new();
    let mut index = 0;
    let chs_len = name.len();
    for x in chs {
        if x.is_uppercase() {
            if index != 0 && (index + 1) != chs_len {
                new_name.push_str("_");
            }
            new_name.push_str(x.to_lowercase().to_string().as_str());
        } else {
            new_name.push(x);
        }
        index += 1;
    }
    return new_name;
}

fn gen_columns(fields:&Vec<Ident>)-> TokenStream{
    let mut s = quote! {};
    for x in fields {
        s = quote! { #s stringify!(#x),}
    }
    s
}

fn gen_fields(data: &syn::Data) -> Vec<Ident> {
    let mut fields:Vec<Ident> = vec![];
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

fn gen_driver_token(mut token_string: String) -> proc_macro2::TokenStream {
    if token_string.contains("#[derive("){
        token_string = (&token_string[token_string.find("#[derive(").unwrap()as usize..token_string.len()]).to_string();
    }
    let have_ser_driver_macro = token_string.contains("cdbc::Scan)") ||  token_string.contains("cdbc::Scan,");
    let driver_token;
    if have_ser_driver_macro {
        driver_token = quote! {}
    } else {
        driver_token = quote! {
           #[derive(cdbc::Scan)]
        }
    }
    return driver_token;
}

///t:cdbc_sqlite::SqlitePool
/// name:table
fn do_impl_curd(name:&Ident,t: &TokenStream) -> TokenStream{
    let mut data = quote! (
      impl cdbc::crud::CRUD<#name> for #t {
        fn inserts(&mut self, arg: Vec<#name>) -> cdbc::Result<(String,u64)> where #name: Sized {
            use cdbc::{Either, Executor, query};
            use cdbc::scan::Scan;
            if arg.len() == 0 {
                return Ok((String::new(),0));
            }
            let mut arg_idx = 1;
            let mut sql = format!("insert into {} ({}) values ", #name::table(), #name::columns_str());
            let mut value_num = 0;
            for x in &arg {
                if value_num != 0 {
                    sql.push_str(",");
                }
                sql.push_str("(");
                sql.push_str(&#name::values_str("?", &mut arg_idx));
                sql.push_str(")");
                value_num += 1;
            }
            log::info!("sql=> {}",sql);
            let mut q = query(sql.as_str());
            for arg in arg {
                log::info!("arg=> {:?},{:?},{:?},{:?}",arg.id,arg.name,arg.age,arg.delete_flag);
                q = q.bind(arg.id)
                    .bind(arg.name)
                    .bind(arg.age)
                    .bind(arg.delete_flag);
            }
            self.execute(q).map(|r| {
                (r.last_insert_id().to_string(),r.rows_affected())
            })
        }

        fn updates(&mut self, args: Vec<#name>, r#where: &str) -> cdbc::Result<u64> where #name: Sized {
            use cdbc::{Either, Executor, query};
            use cdbc::scan::Scan;
            let mut num = 0;
            for arg in args {
                let mut q = query("");
                let mut arg_idx = 1;
                let mut sets = String::new();

                if arg.id.is_some() {
                    sets.push_str("id = ");
                    sets.push_str(&#name::p("?", &mut arg_idx));
                    sets.push_str(",");
                    q = q.bind(arg.id);
                }
                if arg.name.is_some() {
                    sets.push_str("name = ");
                    sets.push_str(&#name::p("?", &mut arg_idx));
                    sets.push_str(",");
                    q = q.bind(arg.name);
                }
                if arg.age.is_some() {
                    sets.push_str("age = ");
                    sets.push_str(&#name::p("?", &mut arg_idx));
                    sets.push_str(",");
                    q = q.bind(arg.age);
                }
                if arg.delete_flag.is_some() {
                    sets.push_str("delete_flag = ");
                    sets.push_str(&#name::p("?", &mut arg_idx));
                    sets.push_str(",");
                    q = q.bind(arg.delete_flag);
                }
                if sets.ends_with(",") {
                    sets.pop();
                }
                let mut w = r#where.to_string();
                if !w.trim().is_empty() {
                    w.insert_str(0, "where ");
                }
                let mut sql = format!("update {} set {} {}", #name::table(), sets, w);
                log::info!("sql=> {}",sql);
                q.statement = Either::Left(sql);
                self.execute(q).map(|r| {
                    num += r.rows_affected();
                })?;
            }
            return Ok(num);
        }

        fn find(&mut self, r#where: &str) -> cdbc::Result<#name> where #name: Sized {
            use cdbc::{Either, Executor, query};
            use cdbc::scan::Scan;
            let mut w = r#where.to_string();
            if !w.trim().is_empty() {
                w.insert_str(0, "where ");
            }
            let mut sql = format!("select * from {} {} ", #name::table(), w);
            let q = query(&sql);
            self.fetch_one(q)?.scan()
        }

        fn finds(&mut self, r#where: &str) -> cdbc::Result<Vec<#name>> where #name: Sized {
            use cdbc::{Either, Executor, query};
            use cdbc::scan::Scan;
            let mut w = r#where.to_string();
            if !w.trim().is_empty() {
                w.insert_str(0, "where ");
            }
            let mut sql = format!("select * from {} {} ", #name::table(), w);
            let q = query(&sql);
            self.fetch_all(q)?.scan()
        }

        fn delete(&mut self, r#where: &str) -> cdbc::Result<u64> where {
            use cdbc::{Either, Executor, query};
            use cdbc::scan::Scan;
            let mut w = r#where.to_string();
            if !w.trim().is_empty() {
                w.insert_str(0, "where ");
            }
            let mut sql = format!("delete from {} {} ", #name::table(), w);
            let q = query(&sql);
            self.execute(q).map(|r| {
                r.rows_affected()
            })
        }
    }
   );
    return data;
}