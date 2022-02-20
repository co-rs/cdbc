pub trait Table {
    fn table_name() -> &'static str;
    fn table_columns() -> &'static [&'static str];
}

/// Scan trait must be impl macro
pub trait Scan<Table> {
    fn scan(&mut self) -> crate::Result<Table>;
}

/// Scan trait must be impl macro
pub trait Scans<Table> {
    fn scan(&mut self) -> crate::Result<Vec<Table>>;
}

/// impl scan for table struct
/// for example:
/// ```rust
/// use cdbc::{impl_scan,query};
/// use cdbc::scan::{Scan,Scans};
/// pub struct BizActivity {
///     pub id: Option<String>,
///     pub name: Option<String>,
///     pub delete_flag: Option<i32>,
/// }
/// impl_scan!(SqliteRow,BizActivity{id:None,name:None,delete_flag:None});
///
///
///  let v:Vec<BizActivity > = query!("select * from biz_activity limit 1").fetch_all(pool)?.scan()?;
/// ```
#[macro_export]
macro_rules! impl_scan {
    ($db_row:path,$table:path{$($field_name:ident: $field_value:expr$(,)?)+}) => {
    impl $crate::scan::Table for $table{
    fn table_name() -> &'static str {
        stringify!($table)
    }

    fn table_columns() -> &'static [&'static str] {
        &[$(stringify!($field_name),)+]
     }
   }

    impl $crate::scan::Scan<$table> for $db_row{
      fn scan(&mut self) -> cdbc::Result<$table> {
          $crate::row_scan!(self,$table { $($field_name:$field_value,)+})
      }
    }
    impl $crate::scan::Scans<$table> for Vec<$db_row>{
      fn scan(&mut self) -> cdbc::Result<Vec<$table>> {
          $crate::row_scans!(self,$table { $($field_name:$field_value,)+})
      }
    }
  };
}


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