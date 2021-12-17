use std::pin::Pin;

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

pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item=T> + Send + 'a>>;


#[cfg(test)]
mod test {
    use crate::io::box_stream::{BoxStream, Stream};

    pub struct S {
        inner: Vec<String>,
    }

    impl Stream for S {
        type Item = String;

        fn next(&mut self) -> Option<Self::Item> {
            self.inner.pop()
        }

        fn try_collect(&mut self) -> crate::Result<Self::Item> {
            if let Some(v)=self.inner.pop(){
                return Ok(v)
            }
            return Err("none".into());
        }
    }
    #[test]
    fn test_collect() {
        let mut s = Box::pin(S { inner: vec!["1".to_string()] });
        s.for_each(|v| {
            println!("{}", v);
        });
    }

    #[test]
    fn test_for_each() {
        let mut s = Box::pin(S { inner: vec!["1".to_string()] });
        s.for_each(|v| {
            println!("{}", v);
        });
    }
}