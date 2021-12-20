extern crate may_minihttp;

use std::collections::BTreeMap;
use std::io;
use may_minihttp::{HttpServer, HttpService, Request, Response};
use cdbc::column::Column;
use cdbc::decode::Decode;
use cdbc::executor::Executor;
use cdbc::row::Row;
use cdbc::pool::{Pool, PoolConnection};
use cdbc_mysql::{MySqlPool, MySql, MySqlRow};


#[macro_use]
extern crate lazy_static;
lazy_static!(
    pub static ref POOL: Pool<MySql> = MySqlPool::connect("mysql://root:123456@localhost:3306/test").unwrap();
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
        cdbc::row_scan_struct!(row,BizActivity{id: None,name: None,delete_flag: None})
    }
    pub fn fetch_all() -> cdbc::Result<Vec<BizActivity>> {
        let mut conn = POOL.acquire()?;
        let row = conn.fetch_all("select * from biz_activity limit 1")?;
        cdbc::row_scan_structs!(row,BizActivity{id: None,name: None,delete_flag: None})
    }
}

impl HttpService for HelloWorld {
    fn call(&mut self, req: Request, resp: &mut Response) -> io::Result<()> {
        match BizActivity::fetch_one() {
            Ok(v) => {
                resp.body_vec(serde_json::to_string(&v).unwrap().into_bytes());
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