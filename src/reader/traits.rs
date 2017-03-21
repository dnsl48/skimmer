pub trait Read {
    fn consume<'a> (&'a mut self, len: usize) -> Chunk<'a>;

    fn has (&mut self, len: usize) -> bool;

    fn skip (&mut self, len: usize) -> usize;

    fn slice (&mut self, len: usize) -> Option<&[u8]> { self.slice_at (0, len) }

    fn slice_at (&mut self, at: usize, len: usize) -> Option<&[u8]>;

    fn contains_at<S: Symbol> (&mut self, symbol: &S, at: usize) -> bool;
}



pub trait IntoReader where Self::Reader: Read {
    type Reader;

    fn into_reader (self) -> Self::Reader;
}
