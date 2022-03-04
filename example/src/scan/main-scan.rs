use std::fs::File;
use cdbc::{Executor, query};
use cdbc::crud::Table;
use cdbc::database::Database;
use cdbc_sqlite::{Sqlite, SqlitePool};
use cdbc::Scan;
use cdbc::scan::Scan;

/// or use this example
/// #[derive(Debug,cdbc::ScanSqlite,cdbc::ScanMssql,cdbc::ScanMysql,cdbc::ScanPg)]
#[derive(Debug, cdbc::Scan)]
pub struct BizActivity {
    pub id: Option<String>,
    pub name: Option<String>,
    pub age:Option<i32>,
    pub delete_flag: Option<i32>,
}

impl Table<Sqlite> for BizActivity {
    fn table() -> &'static str {
        "biz_activity"
    }

    fn columns() -> &'static [&'static str] {
        &["id", "name","age", "delete_flag"]
    }

    fn insert<E>(e: E, arg: Self) -> cdbc::Result<u64> where E: Executor<Database=Sqlite> {
        let sql = format!("insert into {} ({}) values (?,?,?,?)", Self::table(), Self::columns_str());
        query::<Sqlite>(sql.as_str())
            .bind(arg.id)
            .bind(arg.name)
            .bind(arg.age)
            .bind(arg.delete_flag)
            .execute(e).map(|r|{
            r.rows_affected()
        })
    }

    fn inserts<E>(e: E, arg: Vec<Self>) -> cdbc::Result<u64> where E: Executor<Database=Sqlite>, Self: Sized {
        todo!()
    }

    fn update<E>(e: E, arg: Self) -> cdbc::Result<u64> where E: Executor<Database=Sqlite> {
        todo!()
    }

    fn updates<E>(e: E, arg: Vec<Self>) -> cdbc::Result<u64> where E: Executor<Database=Sqlite>, Self: Sized {
        todo!()
    }

    fn find<E>(e: E, arg: &str) -> cdbc::Result<Self> where E: Executor<Database=Sqlite>, Self: Sized {
        todo!()
    }

    fn finds<E>(e: E, arg: &str) -> cdbc::Result<Self> where E: Executor<Database=Sqlite>, Self: Sized {
        todo!()
    }

    fn delete<E>(e: E, arg: &str) -> cdbc::Result<u64> where E: Executor<Database=Sqlite> {
        todo!()
    }
}

fn main() -> cdbc::Result<()> {
    let pool = make_sqlite()?;

    let arg = BizActivity {
        id: Some("1".to_string()),
        name: Some("1".to_string()),
        age:Some(1),
        delete_flag: Some(1),
    };
    BizActivity::insert(&pool,arg).unwrap();

    let data = query!("select * from biz_activity limit 1")
        .fetch_one(pool.clone())
        .scan();
    println!("{:?}", data);

    let data = query!("select * from biz_activity limit 1")
        .fetch_all(pool.clone())
        .scan();
    println!("{:?}", data);
    Ok(())
}

fn make_sqlite() -> cdbc::Result<SqlitePool> {
    //first. create sqlite dir/file
    std::fs::create_dir_all("target/db/");
    File::create("target/db/sqlite.db");
    //next create table and query result
    let pool = SqlitePool::connect("sqlite://target/db/sqlite.db")?;
    let mut conn = pool.acquire()?;
    conn.execute("CREATE TABLE biz_activity(  id string, name string,age int, delete_flag int) ");
    conn.execute("INSERT INTO biz_activity (id,name,age,delete_flag) values (\"1\",\"1\",1,0)");
    Ok(pool)
}
