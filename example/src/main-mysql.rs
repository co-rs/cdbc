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
    fn test_mysql() {
        println!("conn");
        let pool = MySqlPool::connect("mysql://root:123456@localhost:3306/test").unwrap();
        println!("acq");
        let mut conn = pool.acquire().unwrap();
        let mut data: ChanStream<MySqlRow> = conn.fetch("select * from biz_activity;");
        data.try_for_each(|it| {
            let mut m = BTreeMap::new();
            for column in it.columns() {
                let v = it.try_get_raw(column.name()).unwrap();
                let r: Option<String> = Decode::<'_, MySql>::decode(v).unwrap();
                m.insert(column.name().to_string(), r);
            }
            println!("{:?}", m);
            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_mysql_fetch_all() {
        println!("conn");
        let pool = MySqlPool::connect("mysql://root:123456@localhost:3306/test").unwrap();
        println!("acq");
        let mut conn = pool.acquire().unwrap();
        let mut data: Vec<MySqlRow> = conn.fetch_all("select * from biz_activity;").unwrap();
        for it in data {
            let mut m = BTreeMap::new();
            for column in it.columns() {
                let v = it.try_get_raw(column.name()).unwrap();
                let r: Option<String> = Decode::<'_, MySql>::decode(v).unwrap();
                m.insert(column.name().to_string(), r);
            }
            println!("{:?}", m);
        }
    }

    #[test]
    fn test_mysql_pool() {
        may::config().set_stack_size(0x1000*2);
        let total=1000;
        let (s,r)=channel();
        let pool = Arc::new(MySqlPool::connect("mysql://root:123456@localhost:3306/test").unwrap());
        for idx in 0..total{
            let s1=s.clone();
            let p = pool.clone();
            sleep(Duration::from_millis(10));
            let f=move ||{
                let mut conn = p.acquire().unwrap();
                let mut data: Vec<MySqlRow> = conn.fetch_all("select * from biz_activity;").unwrap();
                for it in data {
                    let mut m = BTreeMap::new();
                    for column in it.columns() {
                        let v = it.try_get_raw(column.name()).unwrap();
                        let r: Option<String> = Decode::<'_, MySql>::decode(v).unwrap();
                        m.insert(column.name().to_string(), r);
                    }
                    //println!("{:?}", m);
                }
                println!("done:{}",idx);
                s1.send(1);
            };
            go!(f);
        }
        let mut recvs =0;
        for idx in 0..total{
            if let Ok(v)=r.recv(){
                recvs+=1;
            }
            if recvs==total{
                break;
            }
        }
    }
}