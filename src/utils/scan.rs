
/// scan CDBC Row to an Table,return cdbc::Result<Table>
/// for example:
///
///  pub struct BizActivity {
///     pub id: Option<String>,
///     pub name: Option<String>,
///     pub delete_flag: Option<i32>,
///   }
///
///    let mut data = conn.fetch_all("select * from biz_activity;")?;
///    for row in data {
///       let biz_activity = cdbc::row_scan_struct!(x,BizActivity{id: None,name: None,delete_flag: None})
///    }
#[macro_export]
macro_rules! row_scan_struct {
    ($row:ident,$target:path{$($field_name:ident: $field_value:expr$(,)?)+}) => {
        {
            //logic code
        let mut table = {
            $target {
               $(
                    $field_name:$field_value,
               )+
            }
        };
        use cdbc::row::Row;
        for _column in $row.columns(){
             use cdbc::row::Row;use cdbc::column::Column;
             $(
                  if stringify!($field_name).eq(_column.name()){
                     let v = $row.try_get_raw(_column.name())?;
                     table.$field_name = cdbc::decode::Decode::decode(v)?;
                   }
             )+
        }
          cdbc::Result::Ok(table)
        }
    }
}

#[macro_export]
macro_rules! row_scan_structs {
   ($rows:ident,$target:path{$($field_name:ident: $field_value:expr$(,)?)+}) => {
        {
           let mut result_datas = vec![];
           for r in $rows{
             let table = cdbc::row_scan_struct!(r, $target { $($field_name:$field_value,)+})?;
             result_datas.push(table);
           }
          cdbc::Result::Ok(result_datas)
        }
   }
}