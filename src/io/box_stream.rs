pub trait Stream {
    type Item;
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
    fn next(&mut self) -> Option<Self::Item>;
}

pub type BoxStream<'a, T> = Box<dyn Stream<Item = T> + Send + 'a>;

