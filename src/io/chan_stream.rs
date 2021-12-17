use std::pin::Pin;
use std::sync::mpsc::RecvError;
use may::sync::mpsc::{Receiver, Sender};
use crate::error::Result;

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
}

pub trait TryStream: Stream {
    type Ok;
    fn try_next(&mut self) -> crate::error::Result<Option<Self::Ok>>;

    fn try_collect(&mut self) -> crate::error::Result<Self::Ok>;

    fn try_filter_map<F>(&mut self, f: F) -> ChanStream<Self::Item> where F: FnMut(Self::Ok) -> Self::Item;
}


/// Channel Stream
pub struct ChanStream<T> {
    pub recv: Receiver<T>,
    pub send: Sender<T>,
}

impl<T> ChanStream<T> {
    pub fn new<F>(f: F) -> Self where F: FnOnce(Sender<T>) {
        let (s, r) = may::sync::mpsc::channel();
        f(s.clone());
        Self {
            recv: r,
            send: s,
        }
    }
}

impl<T> Stream for ChanStream<T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.recv.recv() {
            Ok(v) => { Some(Ok(v)) }
            Err(e) => { None }
        }
    }
}


impl<T> TryStream for ChanStream<T> {
    type Ok = T;

    fn try_next(&mut self) -> crate::error::Result<Option<Self::Ok>> {
        return match self.recv.recv() {
            Ok(v) => { Ok(Some(v)) }
            Err(e) => { Err(e.into()) }
        };
    }

    fn try_collect(&mut self) -> crate::Result<Self::Ok> {
        match self.recv.recv() {
            Ok(v) => { Ok(v) }
            Err(e) => { Err(e.into()) }
        }
    }

    fn try_filter_map<F>(&mut self, mut f: F) -> ChanStream<Self::Item> where F: FnMut(Self::Ok) -> Self::Item {
        let stream = ChanStream::<Self::Item>::new(|v| {});
        loop {
            match self.try_next() {
                Ok(v) => {
                    if let Some(item) = v {
                        stream.send.send((f)(item));
                    } else {
                        break;
                    }
                }
                Err(e) => {
                    stream.send.send(Err(e.into()));
                    break;
                }
            }
        }
        return stream;
    }
}




macro_rules! try_stream {
    ($($block:tt)*) => {
        crate::io::chan_stream::ChanStream::new(move |sender| {
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
    use crate::io::chan_stream::{ChanStream, Stream, TryStream};

    #[test]
    fn test_try_stream() {
        let mut s = try_stream!({
              println!("start");
              r#yield!(1);
        });
        go!(move ||{
            s.for_each(|item|{
            println!("{:?}",item);
        });
       });
    }

    #[test]
    fn test_collect() {
        let mut s = ChanStream::new(|sender| {
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
        let mut s = ChanStream::new(|sender| {
            sender.send(1);
        });
        go!(move ||{
          s.for_each(|v| {
            println!("{:?}", v);
           });
         });
        sleep(Duration::from_secs(1));
    }
}