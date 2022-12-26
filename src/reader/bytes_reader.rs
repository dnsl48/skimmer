use crate::data::Datum;
use crate::marker::Marker;
use crate::reader::Read;

use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct BytesDatum {
    content: Rc<Vec<u8>>,
}

impl Datum for BytesDatum {
    fn len(&self) -> usize {
        Vec::len(&*self.content)
    }

    fn as_slice(&self) -> &[u8] {
        Vec::as_slice(&*self.content)
    }
}

pub struct BytesReader {
    pointer: usize,
    content: BytesDatum,
}

impl BytesReader {
    pub fn new(data: Vec<u8>) -> BytesReader {
        BytesReader {
            pointer: 0,
            content: BytesDatum {
                content: Rc::new(data),
            },
        }
    }
}

impl Read for BytesReader {
    type Datum = BytesDatum;

    fn consume(&mut self, len: u8) -> Marker {
        let len = self.skip(len);
        Marker::new((0, self.pointer - len as usize), (0, self.pointer))
    }

    fn consume_long(&mut self, len: usize) -> Marker {
        let len = self.skip_long(len);
        Marker::new((0, self.pointer - len), (0, self.pointer))
    }

    #[inline(always)]
    fn get_datum(&mut self, index: usize) -> Option<BytesDatum> {
        if index == 0 {
            Some(self.content.clone())
        } else {
            None
        }
    }

    fn skip(&mut self, len: u8) -> u8 {
        if self.pointer < self.content.len() {
            if self.pointer + (len as usize) < self.content.len() {
                self.pointer += len as usize;
                len
            } else {
                let len = self.content.len() - self.pointer;
                self.pointer += len;

                len as u8
            }
        } else {
            0
        }
    }

    fn skip_long(&mut self, len: usize) -> usize {
        if self.pointer < self.content.len() {
            if self.pointer + len < self.content.len() {
                self.pointer += len;
                len
            } else {
                let len = self.content.len() - self.pointer;
                self.pointer += len;

                len
            }
        } else {
            0
        }
    }

    #[inline(always)]
    fn has(&mut self, len: u8) -> bool {
        self.pointer + len as usize <= self.content.len()
    }

    #[inline(always)]
    fn has_long(&mut self, len: usize) -> bool {
        self.pointer + len <= self.content.len()
    }

    #[inline(always)]
    fn byte_at_start(&mut self, byte: u8) -> bool {
        self.content.content.get(self.pointer) == Some(&byte)
    }

    #[inline(always)]
    fn byte_at(&mut self, byte: u8, at: usize) -> bool {
        self.content.content.get(self.pointer + at) == Some(&byte)
    }

    #[inline(always)]
    fn bytes_2_at_start(&mut self, bytes: [u8; 2]) -> bool {
        self.content.content.get(self.pointer..self.pointer + 2) == Some(&bytes)
    }

    #[inline(always)]
    fn bytes_2_at(&mut self, bytes: [u8; 2], at: usize) -> bool {
        self.content
            .content
            .get(self.pointer + at..self.pointer + at + 2)
            == Some(&bytes)
    }

    #[inline(always)]
    fn bytes_3_at_start(&mut self, bytes: [u8; 3]) -> bool {
        self.content.content.get(self.pointer..self.pointer + 3) == Some(&bytes)
    }

    #[inline(always)]
    fn bytes_3_at(&mut self, bytes: [u8; 3], at: usize) -> bool {
        self.content
            .content
            .get(self.pointer + at..self.pointer + at + 3)
            == Some(&bytes)
    }

    #[inline(always)]
    fn bytes_4_at_start(&mut self, bytes: [u8; 4]) -> bool {
        self.content.content.get(self.pointer..self.pointer + 4) == Some(&bytes)
    }

    #[inline(always)]
    fn bytes_4_at(&mut self, bytes: [u8; 4], at: usize) -> bool {
        self.content
            .content
            .get(self.pointer + at..self.pointer + at + 4)
            == Some(&bytes)
    }

    #[inline(always)]
    fn bytes_at_start(&mut self, bytes: &[u8]) -> bool {
        self.content
            .content
            .get(self.pointer..self.pointer + bytes.len())
            == Some(bytes)
    }

    #[inline(always)]
    fn bytes_at(&mut self, bytes: &[u8], at: usize) -> bool {
        self.content
            .content
            .get(self.pointer + at..self.pointer + at + bytes.len())
            == Some(bytes)
    }

    #[inline(always)]
    fn slice_at(&mut self, at: usize, len: usize) -> Option<&[u8]> {
        self.content
            .content
            .get(self.pointer + at..self.pointer + at + len)
    }

    #[inline(always)]
    fn get_byte_at(&mut self, at: usize) -> Option<u8> {
        self.content.content.get(self.pointer + at).map(|b| *b)
    }

    #[inline(always)]
    fn get_byte_at_start(&mut self) -> Option<u8> {
        self.content.content.get(self.pointer).map(|b| *b)
    }

    #[inline(always)]
    fn get_bytes_2_at(&mut self, at: usize) -> Option<(u8, u8)> {
        self.content
            .content
            .get(self.pointer + at..self.pointer + at + 2)
            .map(|ref bs| (bs[0], bs[1]))
    }

    #[inline(always)]
    fn get_bytes_2_at_start(&mut self) -> Option<(u8, u8)> {
        self.content
            .content
            .get(self.pointer..self.pointer + 2)
            .map(|ref bs| (bs[0], bs[1]))
    }

    #[inline(always)]
    fn get_bytes_3_at(&mut self, at: usize) -> Option<(u8, u8, u8)> {
        self.content
            .content
            .get(self.pointer + at..self.pointer + at + 3)
            .map(|ref bs| (bs[0], bs[1], bs[2]))
    }

    #[inline(always)]
    fn get_bytes_3_at_start(&mut self) -> Option<(u8, u8, u8)> {
        self.content
            .content
            .get(self.pointer..self.pointer + 3)
            .map(|ref bs| (bs[0], bs[1], bs[2]))
    }

    #[inline(always)]
    fn get_bytes_4_at(&mut self, at: usize) -> Option<(u8, u8, u8, u8)> {
        self.content
            .content
            .get(self.pointer + at..self.pointer + at + 4)
            .map(|ref bs| (bs[0], bs[1], bs[2], bs[3]))
    }

    #[inline(always)]
    fn get_bytes_4_at_start(&mut self) -> Option<(u8, u8, u8, u8)> {
        self.content
            .content
            .get(self.pointer..self.pointer + 4)
            .map(|ref bs| (bs[0], bs[1], bs[2], bs[3]))
    }
}
