#[deny(unused_variables)]
extern crate mco_http;

use std::fs::File;
use std::ops::Deref;
use std::sync::Arc;
use cdbc_sqlite::{Sqlite, SqlitePool, SqliteRow};
use mco::std::lazy::sync::{Lazy, OnceCell};
use mco_http::route::{MiddleWare, Route};
use mco_http::server::{Request, Response};
use cdbc::{Executor, impl_scan, query};
use cdbc::scan::{Scan, Scans, Table};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BizActivity {
    pub id: Option<String>,
    pub name: Option<String>,
    pub delete_flag: Option<i32>,
}

impl_scan!(SqliteRow,BizActivity{id:None,name:None,delete_flag:None});

impl BizActivity {
    pub fn fetch_all(pool: &SqlitePool) -> cdbc::Result<Vec<Self>> {
        println!("table:{}",BizActivity::table_name());
        let v = query!("select * from biz_activity limit 1")
            .fetch_all(pool)?.scan()?;
        Ok(v)
    }

    pub fn fetch_one(pool: &SqlitePool) -> cdbc::Result<Self> {
        let v = query!("select * from biz_activity limit 1")
            .fetch_one(pool)?.scan()?;
        Ok(v)
    }

    pub fn execute(pool: &SqlitePool) -> cdbc::Result<u64> {
        let v = query!("select * from biz_activity limit ?",1).execute(pool)?;
        Ok(v.rows_affected())
    }
}

fn hello(req: Request, res: Response) {
    let records = BizActivity::fetch_all(&req.extra.get::<Arc<SqlitePool>>().unwrap()).unwrap();
    res.send(serde_json::json!(records).to_string().as_bytes());
}

#[derive(Debug)]
pub struct MyMiddleWare {
    pool: Arc<SqlitePool>,
}

impl MiddleWare for MyMiddleWare {
    fn handle(&self, req: &mut Request, res: &mut Option<Response>) {
        //You can carry any data here
        req.extra.insert(self.pool.clone());
    }
}

fn main() {
    //or use  fast_log::init_log();
    let mut router = Arc::new(Route::new());

    router.add_middleware(MyMiddleWare {
        pool: Arc::new(make_sqlite().unwrap())
    });
    router.handle_fn("/", hello);
    router.handle_fn("/fetch_one", |req: Request, res: Response| {
        let pool = &req.extra.get::<Arc<SqlitePool>>().unwrap();
        res.send(serde_json::json!(BizActivity::fetch_one(pool).unwrap()).to_string().as_bytes());
    });
    router.handle_fn("/execute", |req: Request, res: Response| {
        let pool = &req.extra.get::<Arc<SqlitePool>>().unwrap();
        res.send(serde_json::json!(BizActivity::execute(pool).unwrap()).to_string().as_bytes());
    });
    let _listening = mco_http::Server::http("0.0.0.0:3000").unwrap()
        .handle(router);
    println!("Listening on http://127.0.0.1:3000");
    println!("Listening on http://127.0.0.1:3000/fetch_one");
    println!("Listening on http://127.0.0.1:3000/execute");
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
    conn.execute("INSERT INTO biz_activity (id,name,age,delete_flag) values (\"2\",\"2\",1,0)");
    Ok(pool)
}

