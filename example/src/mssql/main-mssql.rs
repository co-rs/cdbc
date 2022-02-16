use mco::std::time::time::Time;
use cdbc::Executor;
use cdbc_mssql::MssqlPool;

//docker run  --name mssql -e "ACCEPT_EULA=Y" -e "SA_PASSWORD=TestPass!123456" -p 1433:1433 -d genschsa/mssql-server-linux
fn main() -> cdbc::Result<()> {
    #[derive(Debug, serde::Serialize)]
    pub struct BizActivity {
        pub id: Option<String>,
        pub name: Option<String>,
        pub delete_flag: Option<i32>,
        pub create_time: Option<Time>,
    }
    let pool = MssqlPool::connect("mssql://SA:TestPass!123456@localhost:1433/test")?;
    let data = cdbc::row_scans!(
        cdbc::query("select * from biz_activity limit 1")
        .fetch_all(pool)?,
        BizActivity{id:None,name:None,delete_flag:None,create_time:None})?;
    println!("{:?}", data);
    println!("{}", serde_json::to_string(&data).unwrap());
    Ok(())
}


#[cfg(test)]
mod test {
    #[test]
    fn test_stream_mysql() -> cdbc::Result<()> {
        Ok(())
    }
}