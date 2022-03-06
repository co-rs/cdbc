use std::fs::File;
use fast_log::config::Config;
use log::Level;
use cdbc::{Either, Executor, query};
use cdbc::connection::Connection;
use cdbc::crud::{CRUD, Table};
use cdbc::database::Database;
use cdbc_sqlite::{Sqlite, SqlitePool};
use cdbc::scan::Scan;

/// or use this example
/// #[derive(Debug,cdbc::ScanSqlite,cdbc::ScanMssql,cdbc::ScanMysql,cdbc::ScanPg)]
#[cdbc::crud]
#[derive(Debug, Clone)]
pub struct BizActivity {
    pub id: Option<String>,
    pub name: Option<String>,
    pub age: Option<i32>,
    pub delete_flag: Option<i32>,
}

fn main() -> cdbc::Result<()> {
    fast_log::init(Config::new().console().level(Level::Trace));
    let mut pool = make_sqlite()?;

    let arg = BizActivity {
        id: Some("2".to_string()),
        name: Some("2".to_string()),
        age: Some(2),
        delete_flag: Some(1),
    };

    let r = CRUD::insert(&mut pool, arg.clone());
    println!("insert => {:?}", r);

    //pool.clone() also is support
    let mut arg1 = arg.clone();
    arg1.id = None;
    let r = CRUD::update(&mut pool.clone(), arg1, "id = '2'");
    println!("CRUD::update => {:?}", r);

    let mut conn = pool.acquire().unwrap();
    CRUD::insert(&mut conn, arg.clone());

    let mut tx = conn.begin().unwrap();
    let af = CRUD::insert(&mut tx, arg.clone()).unwrap();
    println!("tx CRUD::insert => {:?}", af);

    tx.commit().unwrap();

    let data = query!("select * from biz_activity limit 1")
        .fetch_one(pool.clone())
        .scan();
    println!("{:?}", data);

    let data = query!("select * from biz_activity")
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
