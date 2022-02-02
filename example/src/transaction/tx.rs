use std::fs::File;
use cogo::defer;
use cdbc::connection::Connection;
use cdbc::Executor;
use cdbc_sqlite::SqlitePool;

fn main() -> cdbc::Result<()> {
    let pool = make_sqlite()?;
    #[derive(Debug)]
    pub struct BizActivity {
        pub id: Option<String>,
        pub name: Option<String>,
        pub delete_flag: Option<i32>,
    }
    let mut conn = pool.acquire()?;
    let mut tx = conn.begin()?;
    let r = tx.execute("select count(1) from biz_activity limit 1")?;
    println!("rows_affected: {}", r.rows_affected());
    tx.commit()?;
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