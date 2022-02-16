#[deny(unused_variables)]
extern crate mco_http;

use std::fs::File;
use std::ops::Deref;
use cdbc::Executor;
use cdbc_sqlite::SqlitePool;
use mco::std::lazy::sync::Lazy;
use mco_http::server::{Request, Response};

#[derive(Debug,serde::Serialize,serde::Deserialize)]
pub struct BizActivity {
    pub id: Option<String>,
    pub name: Option<String>,
    pub delete_flag: Option<i32>,
}

impl BizActivity {
    pub fn find_all() -> cdbc::Result<Vec<Self>> {
        let data = cdbc::row_scans!(
        cdbc::query("select * from biz_activity limit 1")
        .fetch_all(&*Pool)?,
        BizActivity{id:None,name:None,delete_flag:None})?;
        Ok(data)
    }
}


fn hello(req: Request, res: Response) {
    let records = BizActivity::find_all().unwrap();
    res.send(serde_json::json!(records).to_string().as_bytes());
}

fn main() {
    //or use  fast_log::init_log();
    let _listening = mco_http::Server::http("0.0.0.0:3000").unwrap()
        .handle(hello);
    println!("Listening on http://127.0.0.1:3000");
}


pub static Pool: Lazy<SqlitePool> = Lazy::new(|| { make_sqlite().unwrap() });

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

