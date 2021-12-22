use cdbc::Executor;
use cdbc_pg::PgPool;

fn main() -> cdbc::Result<()> {
    #[derive(Debug)]
    pub struct BizActivity {
        pub id: Option<String>,
        pub name: Option<String>,
        pub delete_flag: Option<i32>,
    }
    let pool = PgPool::connect("postgres://postgres:123456@localhost:5432/postgres")?;
    let data = cdbc::row_scans!(
        cdbc::query("select * from biz_activity where id = $1")
        .bind("1")
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
    use cdbc_pg::{PgPool, PgRow, Postgres};

    #[test]
    fn test_stream_pg() -> cdbc::Result<()> {
        let pool = PgPool::connect("postgres://postgres:123456@localhost:5432/postgres")?;
        let mut conn = pool.acquire()?;
        loop {
            let mut data: ChanStream<PgRow> = conn.fetch("select * from biz_activity;");
            data.try_for_each(|item| {
                let mut m = BTreeMap::new();
                for column in item.columns() {
                    let v = item.try_get_raw(column.name())?;
                    let r: Option<String> = Decode::<'_, Postgres>::decode(v)?;
                    m.insert(column.name().to_string(), r);
                }
                println!("{:?}", m);
                drop(m);
                Ok(())
            })?;
        }
    }
}