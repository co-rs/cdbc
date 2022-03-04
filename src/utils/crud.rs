use crate::error::Result;
use crate::Executor;

pub trait Table {
    fn table() -> &'static str;
    fn columns() -> &'static [&'static str];
    fn columns_str() -> String{
        let mut s = String::new();
        for x in Self::columns() {
            s.push_str(x);
            s.push_str(",");
        }
        s.pop();
        return s;
    }

    fn insert<DB, E>(e: E, arg: Self) -> Result<u64> where E: Executor<Database=DB>,DB: crate::database::Database;
    fn inserts<DB, E>(e: E, arg: Vec<Self>) -> Result<u64> where E: Executor<Database=DB>, Self: Sized,DB: crate::database::Database;
    fn update<DB, E>(e: E, arg: Self) -> Result<u64> where E: Executor<Database=DB>,DB: crate::database::Database;
    fn updates<DB, E>(e: E, arg: Vec<Self>) -> Result<u64> where E: Executor<Database=DB>, Self: Sized,DB: crate::database::Database;
    fn find<DB, E>(e: E, arg: &str) -> Result<Self> where E: Executor<Database=DB>, Self: Sized,DB: crate::database::Database;
    fn finds<DB, E>(e: E, arg: &str) -> Result<Self> where E: Executor<Database=DB>, Self: Sized,DB: crate::database::Database;
    fn delete<DB, E>(e: E, arg: &str) -> Result<u64> where E: Executor<Database=DB>,DB: crate::database::Database;
}