#[macro_use]
extern crate lazy_static;
#[deny(unused_variables)]
extern crate mco_http;

use std::fs::File;
use std::ops::Deref;
use cdbc::Executor;
use mco::std::lazy::sync::Lazy;
use mco_http::route::Route;
use mco_http::server::{Handler, Request, Response};
use cdbc_mysql::MySqlPool;

lazy_static!(
    pub static ref POOL: MySqlPool = make_pool().unwrap();
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
        let c = cdbc::row_scan!(row,BizActivityCount{count: 0})?;
        Ok(c.count)
    }
}

pub static Pool: Lazy<MySqlPool> = Lazy::new(|| { make_pool().unwrap() });

fn make_pool() -> cdbc::Result<MySqlPool> {
    //Concurrent reads and writes to SQLite limit set connection number to 1
    let pool = MySqlPool::connect("mysql://root:123456@127.0.0.1:3306/test")?;
    Ok(pool)
}

fn hello(req: Request, res: Response) {
    let records = BizActivity::count().unwrap();
    res.send(records.to_string().as_bytes());
}

fn main() {
    //fast_log::init_log();
    let router = Route::new();
    router.handle_fn("/", hello);
    let _listening = mco_http::Server::http("0.0.0.0:3000").unwrap()
        .handle(router);
    println!("Listening on http://127.0.0.1:3000");
}



