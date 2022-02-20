use cdbc::{Executor, impl_scan};
use cdbc_mssql::MssqlPool;

//docker run  --name mssql -e "ACCEPT_EULA=Y" -e "SA_PASSWORD=TestPass!123456" -p 1433:1433 -d genschsa/mssql-server-linux
fn main() -> cdbc::Result<()> {
    #[derive(Debug, serde::Serialize)]
    pub struct BizActivity {
        pub id: Option<String>,
        pub name: Option<String>,
        pub delete_flag: Option<i32>,
    }
    let mut pool = MssqlPool::connect("mssql://SA:TestPass!123456@localhost:1433/test")?;

    //create table
    pool.execute("create table biz_activity
(
    id          varchar(256),
    name        varchar(256),
    delete_flag int,
    create_time datetime
)");

    let data = cdbc::row_scans!(
        cdbc::query("select * from biz_activity")
        .fetch_all(pool)?,
        BizActivity{id:None,name:None,delete_flag:None})?;
    println!("{:?}", data);
    println!("{}", serde_json::to_string(&data).unwrap());
    Ok(())
}