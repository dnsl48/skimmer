use data::Datum;
use marker::Marker;
use reader::Read;
use symbol::{ Symbol, CopySymbol };

use std::sync::Arc;




pub struct SliceReader {
    pointer: usize,
    content: &'static [u8]
}



impl SliceReader {
    pub fn new (slice: &'static [u8]) -> SliceReader {
        SliceReader {
            pointer: 0,
            content: slice
        }
    }
}



impl Read for SliceReader {
    fn consume (&mut self, len: usize) -> Marker {
        let len = self.skip (len);

        Marker::new ((0, self.pointer - len), (0, self.pointer))
    }


    fn get_datum (&mut self, index: usize) -> Option<Arc<Datum>> {
        if index == 0 { Some ( Arc::new (self.content) ) } else { None }
    }


    fn skip (&mut self, len: usize) -> usize {
        if self.pointer < self.content.len () {
            if self.pointer + len < self.content.len () {
                self.pointer += len;
                len

            } else {
                let len = self.content.len () - self.pointer;
                self.pointer += len;

                len
            }
        } else { 0 }
    }


    fn has (&mut self, len: usize) -> bool {
        return self.pointer + len <= self.content.len ();
    }


    fn contains_at<S: Symbol> (&mut self, symbol: &S, at: usize) -> bool { symbol.contained_at (self.content, self.pointer + at) }


    #[inline (always)]
    fn contains_copy_at<S: CopySymbol> (&mut self, symbol: S, at: usize) -> bool { CopySymbol::contained_at (symbol, self.content, self.pointer + at) }


    #[inline (always)]
    fn contains_copy_at_start<S: CopySymbol> (&mut self, symbol: S) -> bool { CopySymbol::contained_at (symbol, self.content, self.pointer) }


    fn slice_at (&mut self, at: usize, len: usize) -> Option<&[u8]> {
        let from = self.pointer + at;
        let to = from + len;

        if from <= self.content.len () && to <= self.content.len () {
            Some (&self.content[from .. to])
        } else {
            None
        }
    }
}



impl Datum for &'static [u8] {
    fn len (&self) -> usize { <[u8]>::len (self) }

    fn as_slice (&self) -> &[u8] { self }
}
