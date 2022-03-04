use mco::err;
use crate::database::Database;
use crate::error::Result;
use crate::Executor;


pub trait Table{
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
}


pub trait CRUD<T:Table> {
    fn insert(&mut self, arg: T) -> Result<u64>{
        self.inserts(vec![arg])
    }
    fn inserts(&mut self, arg: Vec<T>) -> Result<u64> where T: Sized;
    fn update(&mut self, arg: T) -> Result<u64>{
        self.updates(vec![arg])
    }
    fn updates(&mut self, arg: Vec<T>) -> Result<u64> where T: Sized;
    fn find(&mut self, arg: &str) -> Result<Option<T>> where T: Sized;
    fn finds(&mut self, arg: &str) -> Result<Vec<T>> where T: Sized;
    fn delete(&mut self, arg: &str) -> Result<u64> where;
}