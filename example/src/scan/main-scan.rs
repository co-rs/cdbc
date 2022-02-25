use std::fs::File;
use cdbc::Executor;
use cdbc_sqlite::SqlitePool;
use cdbc::ScanAll;
use cdbc::scan::Scans;

#[derive(Debug,ScanAll)]
pub struct BizActivity {
    pub id: Option<String>,
    pub name: Option<String>,
    pub delete_flag: Option<i32>,
}

fn main() -> cdbc::Result<()> {
    let pool = make_sqlite()?;
    let data = cdbc::query("select * from biz_activity limit 1")
        .fetch_all(pool.clone())?.scan();
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
