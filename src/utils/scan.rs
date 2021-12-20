#[macro_export]
macro_rules! scan_row_struct {
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
        $row.columns().iter().for_each(|column|{
             $(
                  if stringify!($field_name).eq(column.name()){
                     let v = $row.try_get_raw(column.name()).unwrap();
                     table.$field_name = Decode::decode(v).unwrap();
                   }
             )+
        });
          cdbc::Result::Ok(table)
        }
    }
}