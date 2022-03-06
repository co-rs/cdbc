use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use mco::err;
use crate::database::{Database, HasArguments};
use crate::error::Result;
use crate::{Encode, Executor, Query};
use crate::arguments::Arguments;
use crate::scan::Scan;
use crate::types::Type;


pub trait Table {
    fn table() -> &'static str;
    fn columns() -> &'static [&'static str];
    fn columns_str() -> String {
        let mut s = String::new();
        for x in Self::columns() {
            s.push_str(x);
            s.push_str(",");
        }
        s.pop();
        return s;
    }
    fn values_str(p: &str, index: &mut i32) -> String {
        let mut s = String::new();
        if *index < 1 {
            *index = 1;
        }
        for x in Self::columns() {
            s.push_str(p);
            if p != "?" {
                s.push_str(index.to_string().as_str());
            }
            s.push_str(",");
            *index += 1;
        }
        s.pop();
        return s;
    }

    fn p(p: &str, index: &mut i32) -> String {
        if *index < 1 {
            *index = 1;
        }
        let mut s = String::new();
        s.push_str(p);
        if p != "?" {
            s.push_str(index.to_string().as_str());
        }
        *index += 1;
        s
    }
}


pub trait CRUD<T: Table> {
    fn insert(&mut self, arg: T) -> Result<(String, u64)> {
        self.inserts(vec![arg])
    }
    fn inserts(&mut self, arg: Vec<T>) -> Result<(String, u64)> where T: Sized;
    fn update(&mut self, arg: T, r#where: &str) -> Result<u64> {
        self.updates(vec![arg], r#where)
    }
    fn updates(&mut self, arg: Vec<T>, r#where: &str) -> Result<u64> where T: Sized;
    fn find(&mut self, r#where: &str) -> Result<T> where T: Sized;
    fn finds(&mut self, r#where: &str) -> Result<Vec<T>> where T: Sized;
    fn delete(&mut self, r#where: &str) -> Result<u64> where;
}