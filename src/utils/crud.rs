use crate::database::Database;
use crate::error::Result;
use crate::Executor;

pub trait Table<DB: Database> {
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
    fn insert<E>(e: E, arg: Self) -> Result<u64> where E: Executor<Database=DB>;
    fn inserts<E>(e: E, arg: Vec<Self>) -> Result<u64> where E: Executor<Database=DB>, Self: Sized;
    fn update<E>(e: E, arg: Self) -> Result<u64> where E: Executor<Database=DB>;
    fn updates<E>(e: E, arg: Vec<Self>) -> Result<u64> where E: Executor<Database=DB>, Self: Sized;
    fn find<E>(e: E, arg: &str) -> Result<Self> where E: Executor<Database=DB>, Self: Sized;
    fn finds<E>(e: E, arg: &str) -> Result<Self> where E: Executor<Database=DB>, Self: Sized;
    fn delete<E>(e: E, arg: &str) -> Result<u64> where E: Executor<Database=DB>;
}