pub mod bytes_reader;
pub mod into_reader;
pub mod ioreadeof_reader;
pub mod slice_reader;

pub use self::bytes_reader::BytesReader;
pub use self::into_reader::IntoReader;
pub use self::ioreadeof_reader::IoreadeofReader;
pub use self::slice_reader::SliceReader;

use crate::marker::Marker;

pub trait Read {
    type Datum;

    fn has(&mut self, len: u8) -> bool;

    fn has_long(&mut self, len: usize) -> bool;

    fn skip(&mut self, len: u8) -> u8;

    fn skip_long(&mut self, len: usize) -> usize;

    fn byte_at(&mut self, byte: u8, at: usize) -> bool;

    fn byte_at_start(&mut self, byte: u8) -> bool;

    fn bytes_2_at(&mut self, bytes: [u8; 2], at: usize) -> bool;

    fn bytes_2_at_start(&mut self, bytes: [u8; 2]) -> bool;

    fn bytes_3_at(&mut self, bytes: [u8; 3], at: usize) -> bool;

    fn bytes_3_at_start(&mut self, bytes: [u8; 3]) -> bool;

    fn bytes_4_at(&mut self, bytes: [u8; 4], at: usize) -> bool;

    fn bytes_4_at_start(&mut self, bytes: [u8; 4]) -> bool;

    fn bytes_at(&mut self, bytes: &[u8], at: usize) -> bool;

    fn bytes_at_start(&mut self, bytes: &[u8]) -> bool;

    fn get_byte_at(&mut self, at: usize) -> Option<u8>;

    fn get_byte_at_start(&mut self) -> Option<u8>;

    fn get_bytes_2_at(&mut self, at: usize) -> Option<(u8, u8)>;

    fn get_bytes_2_at_start(&mut self) -> Option<(u8, u8)>;

    fn get_bytes_3_at(&mut self, at: usize) -> Option<(u8, u8, u8)>;

    fn get_bytes_3_at_start(&mut self) -> Option<(u8, u8, u8)>;

    fn get_bytes_4_at(&mut self, at: usize) -> Option<(u8, u8, u8, u8)>;

    fn get_bytes_4_at_start(&mut self) -> Option<(u8, u8, u8, u8)>;

    fn consume(&mut self, len: u8) -> Marker;

    fn consume_long(&mut self, len: usize) -> Marker;

    fn get_datum(&mut self, index: usize) -> Option<Self::Datum>;

    fn slice(&mut self, len: usize) -> Option<&[u8]> {
        self.slice_at(0, len)
    }

    fn slice_at(&mut self, at: usize, len: usize) -> Option<&[u8]>;
}
