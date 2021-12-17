use std::fmt::{Debug, Display};
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

    fn try_filter_map<F>(&mut self, f: F) -> ChanStream<Self::Item> where F: FnMut(Self::Ok) -> Option<Self::Item>;
}


/// Channel Stream
pub struct ChanStream<T> {
    pub recv: Receiver<Option<T>>,
    pub send: Sender<Option<T>>,
}

impl<T> ChanStream<T> {
    pub fn new<F>(f: F) -> Self where F: FnOnce(Sender<Option<T>>) {
        let (s, r) = may::sync::mpsc::channel();
        f(s.clone());
        //send none, make sure work is done
        s.send(None);
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
            Ok(v) => {
                match v {
                    None => { None }
                    Some(v) => {
                        Some(Ok(v))
                    }
                }
            }
            Err(e) => { None }
        }
    }
}


impl<T> TryStream for ChanStream<T> {
    type Ok = T;

    fn try_next(&mut self) -> crate::error::Result<Option<Self::Ok>> {
        return match self.recv.recv() {
            Ok(v) => {
                match v {
                    None => { Ok(None) }
                    Some(v) => {
                        Ok(Some(v))
                    }
                }
            }
            Err(e) => { Err(e.into()) }
        };
    }



    fn try_filter_map<F>(&mut self, mut f: F) -> ChanStream<Self::Item> where F: FnMut(Self::Ok) -> Option<Self::Item> {
        let stream = ChanStream::<Self::Item>::new(|v| {});
        loop {
            match self.try_next() {
                Ok(v) => {
                    match v {
                        None => { break; }
                        Some(v) => {
                            stream.send.send((f)(v));
                        }
                    }
                }
                Err(e) => {
                    stream.send.send(Some(Err(e.into())));
                    break;
                }
            }
        }
        return stream;
    }
}



#[macro_export]
macro_rules! chan_stream {
    ($($block:tt)*) => {
        crate::io::chan_stream::ChanStream::new(move |sender| {
            macro_rules! r#yield {
                ($v:expr) => {{
                    may::sync::mpsc::Sender::send(&sender,Some($v));
                }}
            }

            ///end loop
            macro_rules! end {
                () => {{
                    may::sync::mpsc::Sender::send(&sender,None);
                }}
            }

            $($block)*
        })
    }
}


/// stream , vec ident
#[macro_export]
macro_rules! collect {
    ($f:ident,$extend:expr) => {
        {
        Ok(loop {
            match $f.try_next()? {
                Some(x) => {
                    $extend.extend(Some(x?))
                }
                None => {
                    break $extend;
                }
            }
        })
        }
    };
}

#[cfg(test)]
mod test {
    use std::thread::sleep;
    use std::time::Duration;
    use may::go;
    use crate::io::chan_stream::{ChanStream, Stream, TryStream};

    #[test]
    fn test_chan_stream() {
        let mut s = chan_stream!({
              println!("start");
              r#yield!(1);
        });
        s.for_each(|item|{
            println!("{:?}",item);
       });
    }


    #[test]
    fn test_for_each() {
        let mut s = ChanStream::new(|sender| {
            sender.send(Some(1));
            sender.send(Some(2));
            sender.send(Some(3));
        });
        s.for_each(|v| {
            println!("{:?}", v);
        });
    }
}