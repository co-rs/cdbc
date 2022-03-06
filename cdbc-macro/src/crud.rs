use proc_macro2::{Ident, TokenStream};
use quote::quote;
use quote::ToTokens;

pub(crate) fn impl_crud(ast: &syn::DeriveInput, db_type: &[TokenStream]) -> crate::proc_macro::TokenStream {
    let name = &ast.ident;
    let field_idents = gen_fields(&ast.data);
    let table_name = to_snake_name(&name.to_string());
    let columns = gen_columns(&field_idents);
    let mut stream = quote! {
        impl Table for #name {
            fn table() -> &'static str {
                #table_name
            }

            fn columns() -> &'static [&'static str] {
                &[#columns]
            }
        }
    };

    let mut curd = quote! (
      impl CRUD<BizActivity> for #name {
        fn inserts(&mut self, arg: Vec<BizActivity>) -> cdbc::Result<(String,u64)> where BizActivity: Sized {
            if arg.len() == 0 {
                return Ok((String::new(),0));
            }
            let mut arg_idx = 1;
            let mut sql = format!("insert into {} ({}) values ", BizActivity::table(), BizActivity::columns_str());
            let mut value_num = 0;
            for x in &arg {
                if value_num != 0 {
                    sql.push_str(",");
                }
                sql.push_str("(");
                sql.push_str(&BizActivity::values_str("?", &mut arg_idx));
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
                (r.last_insert_rowid().to_string(),r.rows_affected())
            })
        }

        fn updates(&mut self, args: Vec<BizActivity>, r#where: &str) -> cdbc::Result<u64> where BizActivity: Sized {
            let mut num = 0;
            for arg in args {
                let mut q = query("");
                let mut arg_idx = 1;
                let mut sets = String::new();

                if arg.id.is_some() {
                    sets.push_str("id = ");
                    sets.push_str(&BizActivity::p("?", &mut arg_idx));
                    sets.push_str(",");
                    q = q.bind(arg.id);
                }
                if arg.name.is_some() {
                    sets.push_str("name = ");
                    sets.push_str(&BizActivity::p("?", &mut arg_idx));
                    sets.push_str(",");
                    q = q.bind(arg.name);
                }
                if arg.age.is_some() {
                    sets.push_str("age = ");
                    sets.push_str(&BizActivity::p("?", &mut arg_idx));
                    sets.push_str(",");
                    q = q.bind(arg.age);
                }
                if arg.delete_flag.is_some() {
                    sets.push_str("delete_flag = ");
                    sets.push_str(&BizActivity::p("?", &mut arg_idx));
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
                let mut sql = format!("update {} set {} {}", BizActivity::table(), sets, w);
                log::info!("sql=> {}",sql);
                q.statement = Either::Left(sql);
                self.execute(q).map(|r| {
                    num += r.rows_affected();
                })?;
            }
            return Ok(num);
        }

        fn find(&mut self, r#where: &str) -> cdbc::Result<BizActivity> where BizActivity: Sized {
            let mut w = r#where.to_string();
            if !w.trim().is_empty() {
                w.insert_str(0, "where ");
            }
            let mut sql = format!("select * from {} {} ", BizActivity::table(), w);
            let q = query(&sql);
            self.fetch_one(q)?.scan()
        }

        fn finds(&mut self, r#where: &str) -> cdbc::Result<Vec<BizActivity>> where BizActivity: Sized {
            let mut w = r#where.to_string();
            if !w.trim().is_empty() {
                w.insert_str(0, "where ");
            }
            let mut sql = format!("select * from {} {} ", BizActivity::table(), w);
            let q = query(&sql);
            self.fetch_all(q)?.scan()
        }

        fn delete(&mut self, r#where: &str) -> cdbc::Result<u64> where {
            let mut w = r#where.to_string();
            if !w.trim().is_empty() {
                w.insert_str(0, "where ");
            }
            let mut sql = format!("delete from {} {} ", BizActivity::table(), w);
            let q = query(&sql);
            self.execute(q).map(|r| {
                r.rows_affected()
            })
        }
    }
   );
   stream = quote!{#stream #curd};
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
            for field in s.fields {
                match field.ident {
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
