extern crate may_minihttp;

use std::collections::BTreeMap;
use std::io;
use may_minihttp::{HttpServer, HttpService, Request, Response};
use cdbc::column::Column;
use cdbc::decode::Decode;
use cdbc::executor::Executor;
use cdbc::row::Row;

use cdbc::pool::Pool;
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
impl BizActivity{
    fn scan(row: MySqlRow) -> BizActivity {
        let mut table = BizActivity {
            id: None,
            name: None,
            delete_flag: None,
        };
        for column in row.columns() {
            let v = row.try_get_raw(column.name()).unwrap();
            match column.name() {
                "id" => {
                    let r: Option<String> = Decode::<'_, MySql>::decode(v).unwrap();
                    table.id = r;
                }
                "name" => {
                    let r: Option<String> = Decode::<'_, MySql>::decode(v).unwrap();
                    table.name = r;
                }
                "delete_flag" => {
                    let r: Option<i32> = Decode::<'_, MySql>::decode(v).unwrap();
                    table.delete_flag = r;
                }
                _ => {}
            }
        }
        return table;
    }
}


impl HelloWorld {
    //query from database
    pub fn query(&self) -> Result<Vec<BizActivity>, std::io::Error> {
        let mut conn = POOL.acquire()?;
        let mut data = conn.fetch_all("select * from biz_activity;")?;
        let mut vec = vec![];
        for x in data {
            vec.push(BizActivity::scan(x));
        }
        Ok(vec)
    }
}

impl HttpService for HelloWorld {
    fn call(&mut self, req: Request, resp: &mut Response) -> io::Result<()> {
        let r = self.query();
        if let Err(e) = r {
            return {
                resp.body_vec(e.to_string().into_bytes());
                Ok(())
            };
        }
        resp.body_vec(serde_json::to_string(&r.unwrap()).unwrap().into_bytes());
        Ok(())
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