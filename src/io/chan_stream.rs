use std::pin::Pin;
use std::sync::mpsc::RecvError;
use may::sync::mpsc::{Receiver, Sender};
use crate::error::Result;

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
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        return match self.recv.recv() {
            Ok(v) => { Some(v) }
            Err(e) => { None }
        };
    }

    fn try_next(&mut self) -> crate::error::Result<Option<Self::Item>> {
        return match self.recv.recv() {
            Ok(v) => { Ok(Some(v)) }
            Err(e) => { Err(e.into()) }
        };
    }

    fn try_collect(&mut self) -> crate::Result<Self::Item> {
        match self.recv.recv() {
            Ok(v) => { Ok(v) }
            Err(e) => { Err(e.into()) }
        }
    }

    fn try_filter_map<F>(&mut self, mut f: F) -> Result<ChanStream<Result<Self::Item>>> where F:FnMut(Self::Item)->Self::Item {
        let stream = ChanStream::<Result<Self::Item>>::new(|v| {});
        loop {
            let item = self.try_next()?;
            if let Some(item)=item{
                stream.send.send(Ok((f)(item)));
            }else{
                break;
            }
        }
        return Ok(stream);
    }
}


pub trait Stream {
    type Item;
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }

    /// Creates a future that attempts to resolve the next item in the stream.
    /// If an error is encountered before the next item, the error is returned
    /// instead.
    ///
    /// This is similar to the `Stream::next` combinator, but returns a
    /// `Result<Option<T>, E>` rather than an `Option<Result<T, E>>`, making
    /// for easy use with the `?` operator.
    fn try_next(&mut self) -> crate::error::Result<Option<Self::Item>>;

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

    fn try_filter_map<F>(&mut self,  f: F) -> Result<ChanStream<Result<Self::Item>>> where F:FnMut(Self::Item)->Self::Item;
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
    use crate::io::chan_stream::{ChanStream, Stream};

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
            println!("{}", v);
           });
         });
        sleep(Duration::from_secs(1));
    }
}