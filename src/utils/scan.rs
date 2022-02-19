
/// scan CDBC Row to an Table,return cdbc::Result<Table>
/// for example:
///
///  pub struct BizActivity {
///     pub id: Option<String>,
///     pub name: Option<String>,
///     pub delete_flag: Option<i32>,
///   }
///
///   // fetch one row
///    let mut row = conn.fetch_one("select * from biz_activity limit 1")?;
///    let biz_activity = cdbc::row_scan!(row,BizActivity{id: None,name: None,delete_flag: None})
///
///    //fetch row vec
///    let mut data = conn.fetch_all("select * from biz_activity;")?;
///    for row in data {
///       let item = cdbc::row_scan!(x,BizActivity{id: None,name: None,delete_flag: None})
///    }
#[macro_export]
macro_rules! row_scan {
    ($row:expr,$target:path{$($field_name:ident: $field_value:expr$(,)?)+}) => {
        {
            //logic code
        let mut table = {
            $target {
               $(
                    $field_name:$field_value,
               )+
            }
        };
        let row = $row;
        use cdbc::row::Row;
        for _column in row.columns(){
             use cdbc::row::Row;use cdbc::column::Column;
             $(
                  if stringify!($field_name).trim_start_matches("r#").eq(_column.name()){
                     let v = row.try_get_raw(_column.name())?;
                     table.$field_name = cdbc::decode::Decode::decode(v)?;
                   }
             )+
        }
          cdbc::Result::Ok(table)
        }
    }
}

/// scan CDBC Row to Table,return cdbc::Result<Vec<Table>>
/// for example:
///
///  pub struct BizActivity {
///     pub id: Option<String>,
///     pub name: Option<String>,
///     pub delete_flag: Option<i32>,
///   }
///
///    let mut rows = conn.fetch_all("select * from biz_activity;")?;
///    let biz_activitys = cdbc::row_scans!(rows,BizActivity{id: None,name: None,delete_flag: None})
#[macro_export]
macro_rules! row_scans {
   ($rows:expr,$target:path{$($field_name:ident: $field_value:expr$(,)?)+}) => {
        {
           let mut result_datas = vec![];
           for r in $rows{
             let table = cdbc::row_scan!(r, $target { $($field_name:$field_value,)+})?;
             result_datas.push(table);
           }
          cdbc::Result::Ok(result_datas)
        }
   }
}

/// for example:
///  let v = fetch_all!(&*Pool,query!("select * from biz_activity"), Table{
///             id: None,
///             name: None,
///             delete_flag: None
///         })?;
#[macro_export]
macro_rules! fetch_all {
    ($pool:expr,$query:expr,$target:path{$($field_name:ident: $field_value:expr$(,)?)+}) => {
        $crate::row_scans!($query.fetch_all($pool)?,$target { $($field_name:$field_value,)+})
    }
}

/// for example:
/// let v = fetch_one!(&*Pool,query!("select * from biz_activity limit 1"), Table{
///             id: None,
///             name: None,
///             delete_flag: None
///         })?;
#[macro_export]
macro_rules! fetch_one {
    ($pool:expr,$query:expr,$target:path{$($field_name:ident: $field_value:expr$(,)?)+}) => {
        $crate::row_scan!($query.fetch_one($pool)?,$target { $($field_name:$field_value,)+})
    }
}

/// for example:
///
/// let v = execute!(&*Pool,query!("select * from biz_activity limit 1"))?;
/// Ok(v.rows_affected())
#[macro_export]
macro_rules! execute {
    ($pool:expr,$query:expr) => {
        $query.execute($pool)
    }
}