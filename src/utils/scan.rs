use crate::database::Database;

pub trait Table {
    fn table_name() -> &'static str;
    fn table_name_snake() -> String;
    fn table_columns() -> &'static [&'static str];
}

/// Scan trait must be impl macro
pub trait Scan<Table, DBType: Database> {
    fn scan(&mut self) -> crate::Result<Table>;
}

/// Scan trait must be impl macro
pub trait Scans<Table, DBType: Database> {
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
    ($row_type:path,$db_type:path,$table:path{$($field_name:ident: $field_value:expr$(,)?)+}) => {
    impl $crate::scan::Scan<$table,$db_type> for $row_type{
      fn scan(&mut self) -> $crate::Result<$table> {
           let mut table = {
             $table {
               $(
                    $field_name:$field_value,
               )+
             }
           };
           use $crate::Row;
           for _column in self.columns(){
             use $crate::row::Row;use $crate::column::Column;
             $(
                  if stringify!($field_name).trim_start_matches("r#").eq(_column.name()){
                     let v = self.try_get_raw(_column.name())?;
                     table.$field_name = $crate::decode::Decode::<$db_type>::decode(v)?;
                   }
             )+
           }
           $crate::Result::Ok(table)
      }
    }
    impl $crate::scan::Scans<$table,$db_type> for Vec<$row_type>{
      fn scan(&mut self) -> $crate::Result<Vec<$table>> {
          let mut result_datas = vec![];
           for r in self{
             let table = r.scan()?;
             result_datas.push(table);
           }
          $crate::Result::Ok(result_datas)
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
    ($row:expr,$table:path{$($field_name:ident: $field_value:expr$(,)?)+}) => {
        {
            //logic code
           let mut table = {
             $table {
               $(
                    $field_name:$field_value,
               )+
             }
           };
           let row = $row;
           for _column in row.columns.iter(){
             use $crate::row::Row;use $crate::column::Column;
             $(
                  if stringify!($field_name).trim_start_matches("r#").eq(_column.name()){
                     let v = row.try_get_raw(_column.name())?;
                     table.$field_name = $crate::decode::Decode::decode(v)?;
                   }
             )+
           }
           $crate::Result::Ok(table)
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
   ($rows:expr,$table:path{$($field_name:ident: $field_value:expr$(,)?)+}) => {
        {
           let mut result_datas = vec![];
           for r in $rows{
             let table = $crate::row_scan!(r, $table { $($field_name:$field_value,)+})?;
             result_datas.push(table);
           }
          $crate::Result::Ok(result_datas)
        }
   }
}