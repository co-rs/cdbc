use std::fs::File;
use cdbc::executor::Executor;
use cdbc_sqlite::SqlitePool;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BizActivity {
    pub id: Option<String>,
    pub name: Option<String>,
    pub delete_flag: Option<i32>,
}

fn main() -> cdbc::Result<()> {
    let pool = make_sqlite()?;
    /// fetch one record
    let record = {
        let mut conn = pool.acquire()?;
        let row = conn.fetch_one("select * from biz_activity limit 1")?;
        cdbc::row_scan_struct!(row,BizActivity{id: None,name: None,delete_flag: None})
    }?;
    /// fetch  records vec
    println!("record: {}", serde_json::to_string(&record).unwrap());
    let records = {
        let mut conn = pool.acquire()?;
        let row = conn.fetch_all("select * from biz_activity limit 1")?;
        cdbc::row_scan_structs!(row,BizActivity{id: None,name: None,delete_flag: None})
    }?;
    println!("records: {}", serde_json::to_string(&records).unwrap());
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
