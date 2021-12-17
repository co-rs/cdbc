use std::fmt::{Debug, Display};
use std::pin::Pin;
use std::sync::mpsc::RecvError;
use may::sync::mpsc::{Receiver, Sender};
use crate::Error;
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
}


/// Channel Stream
pub struct ChanStream<T> {
    pub recv: Receiver<Option<Result<T>>>,
    pub send: Sender<Option<Result<T>>>,
}



impl<T> Stream for ChanStream<T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.recv.recv() {
            Ok(v) => {
                match v {
                    None => { None }
                    Some(v) => {
                        if let Ok(v)=v{
                            Some(Ok(v))
                        }else{
                            None
                        }
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
                        if let Ok(v)=v{
                            Ok(Some(v))
                        }else{
                            Err(v.err().unwrap())
                        }
                    }
                }
            }
            Err(e) => { Err(e.into()) }
        };
    }

}



#[macro_export]
macro_rules! chan_stream {
    ($($block:tt)*) => {
        crate::io::chan_stream::ChanStream::new(move |sender| {
            macro_rules! r#yield {
                ($v:expr) => {{
                    may::sync::mpsc::Sender::send(&sender,Some(Ok($v)));
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


impl<T> ChanStream<T> {
    pub fn new<F>(f: F) -> Self where F: FnOnce(Sender<Option<Result<T>>>)-> Result<()> {
        let (s, r) = may::sync::mpsc::channel();
        let result=f(s.clone());
        //send none, make sure work is done
        if let Err(e)=result{
            s.send(Some(Err(e)));
        }
        s.send(None);
        Self {
            recv: r,
            send: s,
        }
    }


    pub fn collect<A,E>(&mut self, f:fn(T) -> Result<A>) -> Result<E>
    where E: Extend<A> + std::default::Default {
        let mut extend:E = Default::default();
        Ok(loop {
            match self.try_next()? {
                Some(x) => {
                    extend.extend(Some(f(x)?))
                }
                None => {
                    break extend;
                }
            }
        })
    }

    //try map
    pub fn map<O>(&mut self,f:fn(<ChanStream<T> as TryStream>::Ok)->Option<O>) -> ChanStream<Result<O>> {
        chan_stream!({
            loop{
                if let Some(either)=self.try_next()?{
                    match f(either){
                        Some(v)=>{
                             r#yield!(Ok(v));
                        }
                        None =>{
                             end!();
                        }
                    }
                }else {
                    break Ok(());
                }
            }
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
    fn test_chan_stream() {
        let mut s = chan_stream!({
              println!("start");
              r#yield!(1);
            Ok(())
        });
        s.for_each(|item|{
            println!("{:?}",item);
       });
    }


    #[test]
    fn test_for_each() {
        let mut s = chan_stream!({
             r#yield!(1);
             r#yield!(2);
             r#yield!(3);
             Ok(())
        });
        s.for_each(|v| {
            println!("{:?}", v);
        });
    }
}