# cdbc
Coroutine Database driver Connectivity.based on [mco](https://github.com/co-rs/mco)

* High concurrency，based on coroutine
* No ``` Future<'q,Output=*> ```，No ``` async fn ```, No ```.await ```, no Poll* func，No ```Pin``` 
* Optimize the trait system so that it has intelligent hints of the base method
* NativeTls and TCP connections are supported
* Low coupling，The database driver and the abstraction layer are designed separately
* Lightweight, no over-design, only have macro with intelligent tips
* Inspired by golang, [mco](https://github.com/co-rs/mco), sqlx


##### Why cdbc ？
| crates | Concurrency   | feature level | All Smart tips | ```Libc``` | have proc macro      | separation  driver | support env/crates                                                               |
|--------|---------------|---------------|----------------|------------|----------------------|----------------------|----------------------------------------------------------------------------------|
| cdbc   | CSP（mco）     | lower         | √              | only sqlite | Don't need           | √    | ```mco```,  ```mco/std/http```, ```native-thread```,```tokio-spawn_blocking``` |
| rbatis | Future（tokio） | heavy-weight  | √              | only sqlite | only py_sql,html_sql | x    | tokio, async_std, smol                                                           |
| sqlx   | Future（tokio） | lower         | x              | only sqlite | only derive(StructOpt) | x    | tokio, async_std, smol                                                           |
| diesel | Native Thread | lower         | x              | all-libc   | derive(Queryable)  | x    | native thread                                                                    |


#### concurrency benchmark performance(Compare the SQLX/Tokio/Async-std)

| crates           | Requests/sec | Mem  | CpuLoad |
|------------------|--------------|------|--------|
| cdbc-mco-http   | 4606      | 30MB | 6%     |
| sqlx-axum-tokio  | 4560      | 17MB | 8%     |                                                          
| sqlx-actix-async-std | 559.00       | 22MB | 2%     |                                                        
| diesel           | *            | *    |    *   |                                                             

* benchmark with wrk(docker run -it --net=host --rm williamyeh/wrk -t12 -c400 -d30s http://192.168.28.235:8000)
* see detail [benchmark](example/benchmark/wrk_result.md)


## Database Support:
* ```cdbc```         The driver abstraction lib.
* ```cdbc-mysql```   CDBC mysql driver library
* ```cdbc-pg```      CDBC postgres driver library
* ```cdbc-sqlite```  CDBC sqlite driver library
* ```cdbc-mssql```   CDBC microsoft mssql driver library

### Supported functions
* execute： Execute the query and return the total number of rows affected.
* execute_many： Execute multiple queries and return the rows affected from each query, in a stream.
* fetch：   Execute the query and return the generated results as a stream.
* fetch_many： Execute multiple queries and return the generated results as a stream，from each query, in a stream.
* fetch_all： Execute the query and return all the generated results, collected into a [`Vec`].
* fetch_one： Execute the query and returns exactly one row.
* fetch_optional： Execute the query and returns at most one row.
* prepare： Prepare the SQL query to inspect the type information of its parameters and results
* prepare_with: Prepare the SQL query, with parameter type information, to inspect the type information about its parameters and results.

### Supported transaction
* Pool:       begin(),commit(),rollback()
* Connection: begin(),commit(),rollback()



use example:

> cargo.toml
```toml
#must dep
cdbc = {version = "0.1"}
#optional dep
cdbc-mysql = {version = "0.1"}
cdbc-pg = {version = "0.1"}
cdbc-sqlite = {version = "0.1"}
```
* row_scan macro
```rust
use std::fs::File;
use cdbc::Executor;
use cdbc_sqlite::SqlitePool;

fn main() -> cdbc::Result<()> {
    let pool = make_sqlite()?;
    #[derive(Debug)]
    pub struct BizActivity {
        pub id: Option<String>,
        pub name: Option<String>,
        pub delete_flag: Option<i32>,
    }

    //execute
    let data = pool.acquire()?.execute("update biz_activity set delete_flag where id = \"1\"")?;
    println!("{:?}", data.rows_affected());
    
    //fetch_all
    let query = cdbc::query("select * from biz_activity where id = ?")
        .bind("1");
    let row = pool.acquire()?.fetch_all(query)?;
    let data = cdbc::row_scans!(row,BizActivity{id:None,name:None,delete_flag:None})?;
    println!("{:?}", data);
    
    //fetch_one
    let data = cdbc::row_scan!(
        cdbc::query("select * from biz_activity where id = ?")
        .bind("1")
        .fetch_one(pool)?,
        BizActivity{id:None,name:None,delete_flag:None})?;
    println!("{:?}", data);

    //transaction
    let mut tx = pool.acquire()?.begin()?;
    let data=tx.execute("update biz_activity set delete_flag where id = \"1\"")?;
    println!("{:?}", data.rows_affected());
    tx.commit()?;
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
```

* Processing read streams
> main.rs
```rust
use std::collections::BTreeMap;
use cdbc::{Column, Decode, Executor, Row};
use cdbc::io::chan_stream::{ChanStream, TryStream};
use cdbc_sqlite::{Sqlite, SqliteRow};
use crate::make_sqlite;
#[test]
fn test_stream_sqlite() -> cdbc::Result<()> {
    //first. create sqlite dir/file
    let pool = make_sqlite().unwrap();
    //next create table and query result
    let mut conn = pool.acquire()?;
    let mut data: ChanStream<SqliteRow> = conn.fetch("select * from biz_activity;");
    data.try_for_each(|item| {
        let mut m = BTreeMap::new();
        for column in item.columns() {
            let v = item.try_get_raw(column.name())?;
            let r: Option<String> = Decode::<'_, Sqlite>::decode(v)?;
            m.insert(column.name().to_string(), r);
        }
        println!("{:?}", m);
        drop(m);
        Ok(())
    })?;
    Ok(())
}
```
