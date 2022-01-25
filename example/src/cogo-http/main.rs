use std::fs::File;
use std::io;
use std::ops::Deref;
use cogo::std::http::server::{HttpServer, HttpService, Request, Response};
use cogo::std::lazy::sync::Lazy;
use cdbc::executor::Executor;
use cdbc::pool::Pool;
use cdbc::PoolOptions;
use cdbc_mysql::{MySql, MySqlPool};

pub static  POOL:Lazy<Pool<MySql>>= Lazy::new(||{
     make_pool().unwrap()
});


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
        match BizActivity::count() {
            Ok(v) => {
                resp.body_vec(v.to_string().into_bytes());
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
    //if use ssl,or debug. Release mode doesn't require that much stack memory
    //cogo::config().set_stack_size(2*0x1000);//8kb

    //check and init pool
    POOL.deref();
    let server = HttpServer(HelloWorld).start("0.0.0.0:8000").unwrap();
    println!("http start on http://127.0.0.1:8000");
    server.join().unwrap();
}

fn make_pool() -> cdbc::Result<MySqlPool> {
    //Concurrent reads and writes to SQLite limit set connection number to 1
    let pool = MySqlPool::connect("mysql://root:123456@127.0.0.1:3306/test")?;
    Ok(pool)
}