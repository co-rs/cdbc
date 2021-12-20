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


#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, HashMap};
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::Duration;
    use may::go;
    use may::sync::mpmc::channel;
    use cdbc::database::Database;
    use cdbc::column::Column;
    use cdbc::decode::Decode;
    use cdbc::executor::Executor;
    use cdbc::io::chan_stream::{ChanStream, Stream, TryStream};
    use cdbc::query::Query;
    use cdbc::row::Row;
    use cdbc_mysql::{MySql, MySqlPool, MySqlRow};

    #[test]
    fn test_prepare_sql() -> cdbc::Result<()> {
        #[derive(Debug)]
        pub struct BizActivity {
            pub id: Option<String>,
            pub name: Option<String>,
            pub delete_flag: Option<i32>,
        }
        let pool = MySqlPool::connect("mysql://root:123456@localhost:3306/test")?;
        let mut conn = pool.acquire()?;
        let mut q = cdbc::query::query("select * from biz_activity where id = ?");
        q = q.bind(1);
        let r = conn.fetch_one(q)?;
        let data = cdbc::row_scan_struct!(r,BizActivity{id:None,name:None,delete_flag:None})?;
        println!("{:?}", data);
        Ok(())
    }
}