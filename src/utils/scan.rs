use crate::column::{Column, ColumnIndex};
use crate::database::{Database, HasValueRef};
use crate::Error;
use crate::row::Row;
use crate::value::ValueRef;
//
// pub fn for_each_row<'a,R:Row,T,V:ValueRef<'a>>(row:R, f:fn(&mut T,V), t: &mut T)->crate::Result<T>
//     where <<R as Row>::Database as Database>::Column: ColumnIndex<R>,
//           <<R as Row>::Database as HasValueRef<'a>>::ValueRef: ValueRef<'a>
// {
//     for column in row.columns() {
//         let mut v = row.try_get_raw(column)?;
//         f(t, v);
//     };
//     return Ok(t);
// }

#[macro_export]
macro_rules! scan_struct {
    ($row:ident,$target:path{$($field_name:ident: $field_value:expr,)+}) => {
       {
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
        return cdbc::Result::<$target>::Ok(table);
      }
    }
}