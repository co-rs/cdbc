use crate::error::Result;

pub trait Table {
    fn table() -> &'static str;
    fn columns() -> &'static [&'static str];

    fn insert(arg: Self) -> Result<u64>;
    fn inserts(arg: Vec<Self>) -> Result<u64>;
    fn update(arg: Self) -> Result<u64>;
    fn updates(arg: Vec<Self>) -> Result<u64>;
    fn find(arg: &str) -> Result<Self>;
    fn finds(arg: &str) -> Result<Self>;
    fn delete(arg: &str) -> Result<u64>;
}