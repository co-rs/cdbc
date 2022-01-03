use cdbc::Executor;
use cdbc_mysql::MySqlPool;

fn main() -> cdbc::Result<()> {
    #[derive(Debug)]
    pub struct BizActivity {
        pub id: Option<String>,
        pub name: Option<String>,
        pub delete_flag: Option<i32>,
    }
    let pool = MySqlPool::connect("mysql://root:123456@localhost:3306/test")?;
    let data = cdbc::row_scans!(
        cdbc::query("select * from biz_activity limit 1")
        .fetch_all(pool)?,
        BizActivity{id:None,name:None,delete_flag:None})?;
    println!("{:?}", data);
    Ok(())
}


#[cfg(test)]
mod test {
    use std::collections::BTreeMap;
    use cdbc::{Column, Decode, Executor, Row};
    use cdbc::io::chan_stream::{ChanStream, TryStream};
    use cdbc_mysql::{MySql, MySqlPool, MySqlRow};

    #[test]
    fn test_stream_mysql() -> cdbc::Result<()> {
        let pool = MySqlPool::connect("mysql://root:123456@localhost:3306/test")?;
        let mut conn = pool.acquire()?;
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
        Ok(())
    }
}