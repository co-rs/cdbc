use std::pin::Pin;
use std::sync::mpsc::RecvError;
use may::sync::mpsc::{Receiver, Sender};


pub struct BoxStream<T> {
    pub recv: Receiver<T>,
    pub send: Sender<T>,
}

impl<T> BoxStream<T> {
    pub fn new<F>(f: F) -> Self where F: FnOnce(Sender<T>) {
        let (s, r) = may::sync::mpsc::channel();
        f(s.clone());
        Self {
            recv: r,
            send: s,
        }
    }
}

impl<T> Stream for BoxStream<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.recv.recv() {
            Ok(v) => { Some(v) }
            Err(_) => { None }
        }
    }

    fn try_collect(&mut self) -> crate::Result<Self::Item> {
        match self.recv.recv() {
            Ok(v) => { Ok(v) }
            Err(e) => { Err(e.into()) }
        }
    }
}


pub trait Stream {
    type Item;
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }

    fn next(&mut self) -> Option<Self::Item>;

    fn for_each(&mut self, f: fn(a: Self::Item)) where Self: Sized {
        loop {
            if let Some(v) = self.next() {
                f(v);
            } else {
                break;
            }
        }
    }

    fn try_collect(&mut self) -> crate::error::Result<Self::Item>;
}


macro_rules! try_stream {
    ($($block:tt)*) => {
        BoxStream::new(move |sender| {
            macro_rules! r#yield {
                ($v:expr) => {{
                    let _ = may::sync::mpsc::Sender::send(&sender,$v);
                }}
            }

            $($block)*
        })
    }
}


#[cfg(test)]
mod test {
    use std::thread::sleep;
    use std::time::Duration;
    use may::go;
    use crate::io::box_stream::{BoxStream, Stream};

    #[test]
    fn test_try_stream() {
        let mut s = try_stream!({
              println!("start");
              r#yield!(1);
        });
        go!(move ||{
            s.for_each(|item|{
            println!("{}",item);
        });
       });
    }

    #[test]
    fn test_collect() {
        let mut s = BoxStream::new(|sender| {
            sender.send(1);
        });
        go!(move ||{
           let v= s.try_collect();
            println!("{:?}",v);
        });
        sleep(Duration::from_secs(1));
    }

    #[test]
    fn test_for_each() {
        let mut s = BoxStream::new(|sender| {
            sender.send(1);
        });
        go!(move ||{
          s.for_each(|v| {
            println!("{}", v);
           });
         });
        sleep(Duration::from_secs(1));
    }
}