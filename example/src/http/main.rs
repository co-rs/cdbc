extern crate may_minihttp;

use std::collections::BTreeMap;
use std::io;
use may_minihttp::{HttpServer, HttpService, Request, Response};
use cdbc::column::Column;
use cdbc::decode::Decode;
use cdbc::executor::Executor;
use cdbc::row::Row;

use cdbc::pool::Pool;
use cdbc_mysql::{MySqlPool,MySql,MySqlRow};

#[macro_use]
extern crate lazy_static;



lazy_static!(
    pub static ref POOL: Pool<MySql> = MySqlPool::connect("mysql://root:123456@localhost:3306/test").unwrap();
);

// implement the `HttpService` trait for your service
#[derive(Clone)]
struct HelloWorld;

impl HttpService for HelloWorld {
    fn call(&mut self, req: Request, resp: &mut Response) -> io::Result<()> {
        let r:Result<Vec<BTreeMap<String,Option<String>>>,std::io::Error>={
            let mut conn =POOL.acquire()?;
            let mut data = conn.fetch_all("select * from biz_activity;")?;
            let mut vec =vec![];
            for it in data {
                let mut m = BTreeMap::new();
                for column in it.columns() {
                    let v = it.try_get_raw(column.name()).unwrap();
                    let r: Option<String> = Decode::<'_, MySql>::decode(v).unwrap();
                    m.insert(column.name().to_string(), r);
                }
                println!("{:?}", m);
                vec.push(m);
            }
            Ok(vec)
        };
        if let Err(e) = r{
            resp.body_vec(format!("{:?}",e).into_bytes());
            return  Ok(());
        }
        resp.body_vec(format!("{:?}",r.unwrap()).into_bytes());
        Ok(())
    }
}

// start the server in main
fn main() {
    may::config().set_stack_size(0x2000);
    let server = HttpServer(HelloWorld).start("0.0.0.0:8000").unwrap();
    println!("http start on http://localhost:8000");
    server.join().unwrap();
}