# cdbc
Database driver based on coroutine


use example:
```rust
use std::collections::{BTreeMap, HashMap};
use cdbc::database::Database;
use cdbc_mysql::{MySql, MySqlPool, MySqlRow};
use cdbc::column::Column;
use cdbc::decode::Decode;
use cdbc::executor::Executor;
use cdbc::io::chan_stream::{ChanStream, Stream};
use cdbc::query::Query;
use cdbc::row::Row;

fn main() {
    let pool = MySqlPool::connect("mysql://root:123456@localhost:3306/test").unwrap();
    let mut conn = pool.acquire().unwrap();
    loop{
        let mut data:ChanStream<_> = conn.fetch("select * from biz_activity;");
        data.for_each(|item |{
            let mut m=BTreeMap::new();
            let it:MySqlRow=item.unwrap();
            for column in it.columns() {
                // println!("{:?}",column.name());
                let v=it.try_get_raw(column.name()).unwrap();
                let r: Option<String> = Decode::<'_, MySql>::decode(v).unwrap();
                m.insert(column.name().to_string(),r);
                // println!("{:?}",r);
            }
            println!("{:?}",m);
            drop(m);
        });
    }
}
```
