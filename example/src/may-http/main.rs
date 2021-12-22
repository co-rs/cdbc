extern crate may_minihttp;
#[macro_use]
extern crate lazy_static;

use std::fs::File;
use std::io;
use may_minihttp::{HttpServer, HttpService, Request, Response};
use cdbc::executor::Executor;
use cdbc::pool::Pool;
use cdbc::PoolOptions;
use cdbc_sqlite::{Sqlite, SqlitePool};

lazy_static!(
    pub static ref POOL: Pool<Sqlite> = make_sqlite_pool().unwrap();
);

// implement the `HttpService` trait for your service
#[derive(Clone)]
struct HelloWorld;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BizActivity {
    pub id: Option<String>,
    pub name: Option<String>,
    pub delete_flag: Option<i32>,
}

impl BizActivity {
    pub fn fetch_one() -> cdbc::Result<BizActivity> {
        let mut conn = POOL.acquire()?;
        let row = conn.fetch_one("select * from biz_activity limit 1")?;
        cdbc::row_scan!(row,BizActivity{id: None,name: None,delete_flag: None})
    }
    pub fn fetch_all() -> cdbc::Result<Vec<BizActivity>> {
        let mut conn = POOL.acquire()?;
        let row = conn.fetch_all("select * from biz_activity")?;
        cdbc::row_scans!(row,BizActivity{id: None,name: None,delete_flag: None})
    }
    pub fn count() -> cdbc::Result<i64> {
        pub struct BizActivityCount {
            pub count: i64,
        }
        let mut conn = POOL.acquire()?;
        let row = conn.fetch_one("select count(1) as count from biz_activity")?;
        let c=cdbc::row_scan!(row,BizActivityCount{count: 0})?;
        Ok(c.count)
    }
}

impl HttpService for HelloWorld {
    fn call(&mut self, req: Request, resp: &mut Response) -> io::Result<()> {
        match BizActivity::fetch_one() {
            Ok(v) => {
                resp.body_vec(format!("{}",serde_json::to_string(&v).unwrap()).into_bytes());
                Ok(())
            }
            Err(e) => {
                resp.body_vec(e.to_string().into_bytes());
                Ok(())
            }
        }
    }
}

// start the server in main
fn main() {
    ///if use ssl,or debug. Release mode doesn't require that much stack memory
    may::config().set_stack_size(8 * 1024);//8kb
    let server = HttpServer(HelloWorld).start("0.0.0.0:8000").unwrap();
    println!("http start on http://127.0.0.1:8000");
    server.join().unwrap();
}

fn make_sqlite_pool() -> cdbc::Result<SqlitePool> {
    //first. create sqlite dir/file
    std::fs::create_dir_all("target/db/");
    File::create("target/db/sqlite.db");
    //Concurrent reads and writes to SQLite limit set connection number to 1
    let pool = PoolOptions::<Sqlite>::new().max_connections(1).connect("sqlite://target/db/sqlite.db")?;
    let mut conn = pool.acquire()?;
    conn.execute("CREATE TABLE biz_activity(  id string, name string,age int, delete_flag int) ");
    conn.execute("INSERT INTO biz_activity (id,name,age,delete_flag) values (\"1\",\"1\",1,0)");
    Ok(pool)
}