pub mod into_reader;
pub mod slice_reader;
pub mod bytes_reader;

pub use self::into_reader::IntoReader;
pub use self::slice_reader::SliceReader;
pub use self::bytes_reader::BytesReader;


use data::Datum;
use marker::Marker;
use symbol::{ Symbol, CopySymbol };

use std::sync::Arc;



pub trait Read {
    fn has (&mut self, len: usize) -> bool;

    fn skip (&mut self, len: usize) -> usize;

    fn contains_at<S: Symbol> (&mut self, symbol: &S, at: usize) -> bool;

    fn contains_copy_at<S: CopySymbol> (&mut self, symbol: S, at: usize) -> bool;

    fn contains_copy_at_start<S: CopySymbol> (&mut self, symbol: S) -> bool;

    fn consume (&mut self, len: usize) -> Marker;

    fn get_datum (&mut self, index: usize) -> Option<Arc<Datum>>;

    fn slice (&mut self, len: usize) -> Option<&[u8]> { self.slice_at (0, len) }

    fn slice_at (&mut self, at: usize, len: usize) -> Option<&[u8]>;
}
