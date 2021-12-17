use std::mem;

fn main() {
    let mut empty=vec![];


   let mut i=0;
   let mut vec = &mut vec![];
    let mut items = &mut vec;
   let v={
       loop {
          match poll(&mut i) {
               Some(x) => items.extend(Some(x)),
               None => break mem::replace(items, &mut empty),
          }
       }
   };
    println!("{:?}",v);
}

fn poll(i:&mut i32)->Option<i32>{
    *i+=1;
    if *i==4{
        return None;
    }
    return Some(2);
}
