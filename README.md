# cdbc
Coroutine Database driver Connectivity.based on [may](https://github.com/Xudong-Huang/may)

* High concurrency，based on coroutine
* No ``` Future<'q,Output=*> ```，No ``` async fn ```, No ```.await ```, no Poll* func，No ```Pin``` 
* Optimize the trait system so that it has intelligent hints of the base method
* NativeTls and TCP connections are supported
* Low coupling，The database driver and the abstraction layer are designed separately
* Inspired by golang, [may](https://github.com/Xudong-Huang/may), sqlx

## Database Support:
* ```cdbc```         The driver abstraction lib.
* ```cdbc-mysql```   CDBC mysql driver library
* ```cdbc-pg```      CDBC postgres driver library
* ```cdbc-sqlite```  CDBC sqlite driver library


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
cdbc = {path = "../"}
cdbc-mysql = {path = "../cdbc-mysql"}
```
* row_scan macro
```rust
pub struct BizActivity {
    pub id: Option<String>,
    pub name: Option<String>,
    pub delete_flag: Option<i32>,
}
let pool = MySqlPool::connect("mysql://root:123456@localhost:3306/test")?;
let mut conn = pool.acquire()?;
//fetch one data
let row = conn.fetch_one("select * from biz_activity limit 1")?;
let data: BizActivity = cdbc::row_scan_struct!(row,BizActivity{id: None,name: None,delete_flag: None})?;
//fetch data array
let rows = conn.fetch_all("select * from biz_activity")?;
let datas:Vec<BizActivity> = cdbc::row_scan_structs!(rows,BizActivity{id: None,name: None,delete_flag: None})?;
```

* Processing read streams
> main.rs
```rust
use std::collections::{BTreeMap, HashMap};
use cdbc::database::Database;
use cdbc_mysql::{MySql, MySqlPool, MySqlRow};
use cdbc::column::Column;
use cdbc::decode::Decode;
use cdbc::executor::Executor;
use cdbc::io::chan_stream::{ChanStream, Stream, TryStream};
use cdbc::query::Query;
use cdbc::row::Row;

fn main() -> cdbc::Result<()> {
    let pool = MySqlPool::connect("mysql://root:123456@localhost:3306/test")?;
    let mut conn = pool.acquire()?;
    loop {
        let mut data: ChanStream<MySqlRow> = conn.fetch("select * from biz_activity;");
        data.try_for_each(|item| {
            let mut m = BTreeMap::new();
            for column in item.columns() {
                let v = item.try_get_raw(column.name())?;
                let r: Option<String> = Decode::<'_, MySql>::decode(v)?;
                m.insert(column.name().to_string(), r);
            }
            println!("{:?}", m);
            drop(m);
            Ok(())
        })?;
    }
}

```
