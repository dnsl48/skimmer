use symbol::Symbol;


use std::cmp::max;

use std::convert::From;

use std::ops::Deref;
use std::ops::Index;
use std::ops::Range;
use std::ops::RangeFrom;
use std::ops::RangeFull;
use std::ops::RangeTo;

use std::fs;
use std::io::Bytes;
use std::io;



pub trait IntoReader where Self::Reader: Read {
    type Reader;

    fn into_reader (self) -> Self::Reader;
}



impl IntoReader for fs::File {
    type Reader = StreamReader<io::BufReader<fs::File>>;

    fn into_reader (self) -> Self::Reader { io::BufReader::new (self).into_reader () }
}



impl<R> IntoReader for io::BufReader<R> where R: io::Read {
    type Reader = StreamReader<io::BufReader<R>>;

    fn into_reader (self) -> StreamReader<io::BufReader<R>> { StreamReader::new (self) }
}



impl IntoReader for &'static str {
    type Reader = SliceReader<'static>;

    fn into_reader (self) -> SliceReader<'static> { SliceReader::new (self.as_bytes ()) }
}



impl IntoReader for String {
    type Reader = BytesReader;

    fn into_reader (self) -> BytesReader { BytesReader::new (self.into_bytes ()) }
}



impl IntoReader for Vec<u8> {
    type Reader = BytesReader;

    fn into_reader (self) -> BytesReader { BytesReader::new (self) }
}




pub trait Read {
    fn consume (&mut self, len: usize) -> Chunk;

    fn has (&mut self, len: usize) -> bool;

    fn skip (&mut self, len: usize) -> usize;

    fn slice (&mut self, len: usize) -> Option<&[u8]> { self.slice_at (0, len) }

    fn slice_at (&mut self, at: usize, len: usize) -> Option<&[u8]>;

    fn contains_at<S: Symbol> (&mut self, symbol: &S, at: usize) -> bool;
}



pub struct SliceReader<'slf> {
    pointer: usize,
    content: &'slf [u8]
}


impl<'slf> SliceReader<'slf> {
    pub fn new (slice: &'slf [u8]) -> SliceReader {
        SliceReader {
            pointer: 0,
            content: slice
        }
    }
}


impl<'slf> Read for SliceReader<'slf> {
    fn consume (&mut self, len: usize) -> Chunk {
        let len = self.skip (len);

        let mut chunk = Chunk::with_capacity (len);
        chunk.push_slice (&self.content[self.pointer - len .. self.pointer]);

        chunk
    }


    fn skip (&mut self, len: usize) -> usize {
        if self.pointer < self.content.len () {
            if self.pointer + len < self.content.len () {
                self.pointer += len;
                return len;

            } else {
                let len = self.content.len () - self.pointer;
                self.pointer += len;

                return len;
            }
        } else { 0 }
    }


    fn has (&mut self, len: usize) -> bool {
        return self.pointer + len <= self.content.len ();
    }


    fn slice_at (&mut self, at: usize, len: usize) -> Option<&[u8]> {
        let from = self.pointer + at;
        let to = from + len;

        if from <= self.content.len () && to <= self.content.len () {
            Some (&self.content[from .. to])
        } else {
            None
        }
    }


    fn contains_at<S: Symbol> (&mut self, symbol: &S, at: usize) -> bool { symbol.contained_at (self.content, self.pointer + at) }
}




pub struct BytesReader {
    pointer: usize,
    content: Vec<u8>
}



impl BytesReader {
    pub fn new (data: Vec<u8>) -> BytesReader {
        BytesReader {
            pointer: 0,
            content: data
        }
    }
}



impl Read for BytesReader {
    fn consume (&mut self, len: usize) -> Chunk {
        let len = self.skip (len);

        let mut chunk = Chunk::with_capacity (len);
        chunk.push_slice (&self.content[self.pointer - len .. self.pointer]);

        chunk
    }


    fn skip (&mut self, len: usize) -> usize {
        if self.pointer < self.content.len () {
            if self.pointer + len < self.content.len () {
                self.pointer += len;
                return len;

            } else {
                let len = self.content.len () - self.pointer;
                self.pointer += len;

                return len;
            }
        } else { 0 }
    }


    fn has (&mut self, len: usize) -> bool {
        return self.pointer + len <= self.content.len ();
    }


    fn slice_at (&mut self, at: usize, len: usize) -> Option<&[u8]> {
        let from = self.pointer + at;
        let to = from + len;

        if from <= self.content.len () && to <= self.content.len () {
            Some (&self.content[from .. to])
        } else {
            None
        }
    }


    fn contains_at<S: Symbol> (&mut self, symbol: &S, at: usize) -> bool { symbol.contained_at (self.content.as_ref (), self.pointer + at) }
}




pub struct StreamReader<R: io::Read> {
    content: Bytes<R>,
    feed: Feed
}


impl<R: io::Read> StreamReader<R> {
    pub fn new (reader: R) -> StreamReader<R> { StreamReader::with_capacity (reader, 1024) }

    pub fn with_capacity (reader: R, capacity: usize) -> StreamReader<R> { StreamReader { content: reader.bytes (), feed: Feed::with_capacity (capacity) } }
}


impl<R: io::Read> Read for StreamReader<R> {
    fn consume (&mut self, len: usize) -> Chunk {
        if self.feed.len () < len {
            let increment = len - self.feed.len ();
            self.feed.push_stream (&mut self.content, increment);
        }

        self.feed.shift (len)
    }


    fn has (&mut self, len: usize) -> bool {
        if self.feed.len () >= len { return true; }

        let increment = len - self.feed.len ();
        self.feed.push_stream (&mut self.content, increment) == increment
    }


    fn slice_at (&mut self, at: usize, len: usize) -> Option<&[u8]> {
        if self.feed.len () < at + len {
            let increment = len + at - self.feed.len ();

            if self.feed.push_stream (&mut self.content, increment) < increment { return None; }
        }

        Some (&&self.feed[at .. len])
    }


    fn skip (&mut self, len: usize) -> usize {
        if len <= self.feed.len () { return self.feed.skip (len); }

        let mut skipped = self.feed.skip (len);

        for _ in 0 .. len - skipped {
            match self.content.next () {
                Some ( Ok ( _ ) ) => skipped += 1,
                _ => break
            }
        }

        skipped
    }


    fn contains_at<S: Symbol> (&mut self, symbol: &S, at: usize) -> bool {
        if self.feed.len () < at + symbol.len () {
            let increment = symbol.len () + at - self.feed.len ();

            if self.feed.push_stream (&mut self.content, increment) < increment { return false; }
        }

        (*self.feed).contains_at (symbol, at)
    }
}




#[derive (Clone, Debug)]
pub struct Chunk {
    capa: usize,
    skip: usize,
    cell: Option<Vec<u8>>
}


impl Chunk {
    pub fn with_capacity (capacity: usize) -> Chunk {
        Chunk { capa: capacity, skip: 0, cell: Some (Vec::with_capacity (capacity)) }
    }


    pub fn capacity (&self) -> usize { self.cell.as_ref ().unwrap ().capacity () }


    pub fn clear (&mut self) { self.skip = 0; self.cell.as_mut ().unwrap ().clear (); }


    pub fn len (&self) -> usize { self.cell.as_ref ().unwrap ().len () - self.skip }


    pub fn skipped (&self) -> usize { self.skip }


    pub fn skip (&mut self, skip: usize) { self.skip = skip; }


    pub fn push_byte (&mut self, byte: u8) { self.cell.as_mut ().unwrap ().push (byte) }


    pub fn push_slice (&mut self, slice: &[u8]) {
        let mut buffer = &mut self.cell.as_mut ().unwrap ();

        buffer.extend (slice);
    }


    pub fn push_stream<Reader: io::Read> (&mut self, stream: &mut Bytes<Reader>, len: usize) -> usize {
        let buffer = self.cell.as_mut ().unwrap ();

        buffer.reserve (len);
        let mut counter = 0;

        for _ in 0 .. len {
            match stream.next () {
                Some ( Ok (b) ) => {
                    buffer.push (b);
                    counter += 1;
                }

                _ => break
            };
        }

        counter
    }


    pub fn push_vec (&mut self, mut slice: Vec<u8>) { self.cell.as_mut ().unwrap ().append (&mut slice); }


    pub fn split_off (&mut self, at: usize) -> Chunk {
        let chunk_len = self.len ();
        let mut buf = self.cell.take ().unwrap ();

        if chunk_len <= at {
            self.cell = Some (buf);

            Chunk::with_capacity (0)

        } else if at <= chunk_len / 2 {
            self.cell = Some (Vec::with_capacity (max (self.capa, at)));

            {
                let slice = &buf[self.skip .. self.skip + at];
                self.push_slice (slice);
            }

            let mut off = Chunk::with_capacity (0);
            off.cell = Some (buf);
            off.capa = self.capa;
            off.skip = self.skip + at;

            self.skip = 0;

            off

        } else {
            let mut off = Chunk::with_capacity (0);
            off.cell = Some (buf.split_off (self.skip + at));
            off.capa = off.cell.as_ref ().unwrap ().capacity ();

            self.cell = Some (buf);

            off
        }
    }


    pub fn truncate (&mut self, len: usize) {
        self.cell.as_mut ().unwrap ().truncate (self.skip + len);
    }


    pub fn wipe_skipped (&mut self) {
        if self.skip == 0 { return; }

        let buf = self.cell.as_mut ().unwrap ();

        let len = buf.len ();
        let skp = self.skip;

        if skp >= len {
            buf.clear ();
        } else if skp > len / 2 || (len % 2 == 0 && skp == len / 2) {
            for i in skp .. len { buf.swap_remove (len - i - 1); }
            buf.truncate (len - skp);
        } else {
            for i in 0 .. len - skp { buf.swap (i, skp + i); }
            buf.truncate (len - skp);
        }

        self.skip = 0;
    }


    pub fn to_vec (mut self) -> Vec<u8> { self.cell.take ().unwrap ().split_off (self.skip) }


    pub fn contains_at<S: Symbol> (&self, symbol: &S, at: usize) -> bool {
        let vec = self.cell.as_ref ().unwrap ();

        vec.len () >= self.skip + at + symbol.len () && unsafe {
            let mut vec_ptr = vec.as_ptr ().offset ((self.skip + at) as isize);
            let mut sym_ptr = symbol.as_ptr ();

            for _ in 0..symbol.len () {
                if *vec_ptr != *sym_ptr { return false; }
                vec_ptr = vec_ptr.offset (1);
                sym_ptr = sym_ptr.offset (1);
            }

            true
        }
    }
}



impl Deref for Chunk {
    type Target = [u8];

    fn deref (&self) -> &[u8] { &self[..] }
}



impl Index<usize> for Chunk {
    type Output = u8;

    fn index (&self, idx: usize) -> &u8 {
        &self.cell.as_ref ().unwrap ()[self.skip + idx]
    }
}



impl Index<Range<usize>> for Chunk {
    type Output = [u8];

    fn index (&self, idx: Range<usize>) -> &[u8] {
        &self.cell.as_ref ().unwrap ()[self.skip + idx.start .. idx.end + self.skip]
    }
}



impl Index<RangeFrom<usize>> for Chunk {
    type Output = [u8];

    fn index (&self, idx: RangeFrom<usize>) -> &[u8] {
        &self.cell.as_ref ().unwrap ()[self.skip + idx.start ..]
    }
}



impl Index<RangeFull> for Chunk {
    type Output = [u8];

    fn index (&self, _idx: RangeFull) -> &[u8] {
        &self.cell.as_ref ().unwrap ()[self.skip ..]
    }
}



impl Index<RangeTo<usize>> for Chunk {
    type Output = [u8];

    fn index (&self, idx: RangeTo<usize>) -> &[u8] {
        &self.cell.as_ref ().unwrap ()[.. idx.end + self.skip]
    }
}



impl From<Vec<u8>> for Chunk {
    fn from (vec: Vec<u8>) -> Chunk { Chunk { capa: vec.len (), skip: 0, cell: Some (vec) } }
}




pub struct Feed {
    wipe_max_threshold: usize,
    wipe_min_threshold: usize,
    base_capacity: usize,

    cell: Option<Chunk>
}



impl Feed {
    pub fn new () -> Feed { Feed::with_capacity (1024) }


    pub fn with_capacity (capacity: usize) -> Feed {
        Feed {
            cell: Some (Chunk::with_capacity (capacity)),
            base_capacity: capacity,
            wipe_min_threshold: 32,
            wipe_max_threshold: capacity * 32
        }
    }


    pub fn wipe_min_threshold (mut self, value: usize) -> Feed {
        self.wipe_min_threshold = value;
        self
    }


    pub fn wipe_max_threshold (mut self, value: usize) -> Feed {
        self.wipe_max_threshold = value;
        self
    }


    pub fn len (&self) -> usize { self.cell.as_ref ().unwrap ().len () }


    pub fn push_vec (&mut self, vec: Vec<u8>) { self.cell.as_mut ().unwrap ().push_vec (vec) }


    pub fn push_slice (&mut self, slice: &[u8]) { self.cell.as_mut ().unwrap ().push_slice (slice) }


    pub fn push_stream<Reader: io::Read> (&mut self, stream: &mut Bytes<Reader>, len: usize) -> usize { self.cell.as_mut ().unwrap ().push_stream (stream, len) }


    pub fn skip (&mut self, len: usize) -> usize {
        if len == 0 { return 0; }

        let chunk_len = self.cell.as_ref ().unwrap ().len ();

        if chunk_len <= len {
            self.cell.as_mut ().unwrap ().clear ();

            chunk_len
        } else {
            let beg = self.cell.as_mut ().unwrap ();

            beg.skip += len;

            if
                (self.wipe_min_threshold > 0 && beg.len () <= self.wipe_min_threshold)
                ||
                (self.wipe_max_threshold > 0 && beg.skip > self.wipe_max_threshold)
            { beg.wipe_skipped (); }

            len
        }
    }


    pub fn shift (&mut self, len: usize) -> Chunk {
        if len == 0 { return Chunk::with_capacity (0); }

        let chunk_len = self.cell.as_ref ().unwrap ().len ();

        if chunk_len <= len {
            let chunk = self.cell.take ().unwrap ();

            self.cell = Some (Chunk::with_capacity (self.base_capacity));

            chunk
        } else {
            let mut beg = self.cell.take ().unwrap ();
            let mut end = beg.split_off (len);

            if
                (self.wipe_min_threshold > 0 && end.len () <= self.wipe_min_threshold)
                ||
                (self.wipe_max_threshold > 0 && end.skip > self.wipe_max_threshold)
            { end.wipe_skipped (); }

            self.cell = Some (end);

            beg
        }
    }
}


impl Deref for Feed {
    type Target = Chunk;

    fn deref (&self) -> &Chunk { &self.cell.as_ref ().unwrap () }
}




#[cfg(test)]
mod tests {
    use super::*;

    use symbol::Char;

    #[test]
    fn test_chunk_contains () {
        let mut chunk = Chunk::with_capacity ("test string".len ());
        chunk.push_slice (&"test string".as_bytes ());

        let ch = Char::new (&[b's']);

        assert! (! chunk.contains_at (&ch, 0));
        assert! (! chunk.contains_at (&ch, 1));
        assert! (  chunk.contains_at (&ch, 2));
        assert! (! chunk.contains_at (&ch, 3));
        assert! (! chunk.contains_at (&ch, 4));
        assert! (  chunk.contains_at (&ch, 5));
        assert! (! chunk.contains_at (&ch, 6));
        assert! (! chunk.contains_at (&ch, 7));
        assert! (! chunk.contains_at (&ch, 8));
        assert! (! chunk.contains_at (&ch, 9));
        assert! (! chunk.contains_at (&ch, 10));
        assert! (! chunk.contains_at (&ch, 11));
        assert! (! chunk.contains_at (&ch, 12));
        assert! (! chunk.contains_at (&ch, 13));
    }


    #[test]
    fn test_chunk_skip () {
        let mut chunk = Chunk::with_capacity ("test string".len ());
        chunk.push_slice (&"test string".as_bytes ());

        assert_eq! (chunk.len (), "test string".len ());

        chunk.skip = "test ".len ();

        assert_eq! (chunk.len (), "string".len ());
    }


    #[test]
    fn test_chunk_to_vec () {
        let mut chunk = Chunk::with_capacity ("test string".len ());
        chunk.push_slice (&"test string".as_bytes ());

        assert_eq! (chunk.len (), "test string".len ());

        chunk.skip = "test ".len ();

        assert_eq! (chunk.len (), "string".len ());

        let v: Vec<u8> = chunk.to_vec ();
        assert_eq! (v.len (), "string".len ());
        assert_eq! (&v[..], "string".as_bytes ());
    }


    #[test]
    fn test_chunk_truncate () {
        let mut chunk = Chunk::with_capacity ("test string".len ());
        chunk.push_slice (&"test string".as_bytes ());

        assert_eq! (chunk.len (), "test string".len ());

        chunk.skip = "test ".len ();

        chunk.truncate (3);

        assert_eq! (chunk.len (), "str".len ());
        assert_eq! (&chunk[..], "str".as_bytes ());
    }


    #[test]
    fn test_chunk_split_off () {
        let src = "test string";

        let mut chunk = Chunk::with_capacity (src.len ());
        chunk.push_slice (&src.as_bytes ());

        assert_eq! (&chunk[..], src.as_bytes ());


        let off = chunk.split_off ("test ".len ());

        assert_eq! ("test ".len (), chunk.len ());

        assert_eq! (&chunk[..], "test ".as_bytes ());
        assert_eq! (&off[..], "string".as_bytes ());

        let mut chunk = Chunk::with_capacity (3);
        chunk.push_vec (vec![1,2,3]);
        let chunk2 = chunk.split_off (1);

        assert_eq! (&chunk[..], [1]);
        assert_eq! (&chunk2[..], [2, 3]);


        let src = "subtle split_off test";
        let mut chunk = Chunk::with_capacity (src.len ());
        chunk.push_slice (&src.as_bytes ());

        chunk.skip = "subtle ".len ();

        let chunk2 = chunk.split_off ("split_off".len ());

        assert_eq! (&chunk[..], "split_off".as_bytes ());
        assert_eq! (&chunk2[..], " test".as_bytes ());



        let src = "123456789";

        let mut chunk = Chunk::with_capacity (src.len ());
        chunk.push_slice (&src.as_bytes ());
        let chunk2 = chunk.split_off (4);

        assert_eq! (&chunk[..], "1234".as_bytes ());
        assert_eq! (&chunk2[..], "56789".as_bytes ());


        let mut chunk = Chunk::with_capacity (src.len ());
        chunk.push_slice (&src.as_bytes ());
        let chunk2 = chunk.split_off (5);

        assert_eq! (&chunk[..], "12345".as_bytes ());
        assert_eq! (&chunk2[..], "6789".as_bytes ());


        let mut chunk = Chunk::with_capacity (src.len ());
        chunk.push_slice (&src.as_bytes ());
        chunk.skip = 4;
        let chunk2 = chunk.split_off (2);

        assert_eq! (&chunk[..], "56".as_bytes ());
        assert_eq! (&chunk2[..], "789".as_bytes ());


        let mut chunk = Chunk::with_capacity (src.len ());
        chunk.push_slice (&src.as_bytes ());
        chunk.skip = 4;
        let chunk2 = chunk.split_off (3);

        assert_eq! (&chunk[..], "567".as_bytes ());
        assert_eq! (&chunk2[..], "89".as_bytes ());
    }


    #[test]
    fn test_chunk_wipe_skipped () {
        let src = "123456789";

        let mut chunk = Chunk::with_capacity (src.len ());
        chunk.push_slice (src.as_bytes ());
        chunk.skip = 1;


        assert_eq! (1, chunk.skip);
        assert_eq! (&chunk[..], "23456789".as_bytes ());

        chunk.wipe_skipped ();
        assert_eq! (0, chunk.skip);
        assert_eq! (&chunk[..], "23456789".as_bytes ());


        let mut chunk = Chunk::with_capacity (src.len ());
        chunk.push_slice (src.as_bytes ());
        chunk.skip = 4;


        assert_eq! (4, chunk.skip);
        assert_eq! (&chunk[..], "56789".as_bytes ());

        chunk.wipe_skipped ();
        assert_eq! (0, chunk.skip);
        assert_eq! (&chunk[..], "56789".as_bytes ());



        let mut chunk = Chunk::with_capacity (src.len ());
        chunk.push_slice (src.as_bytes ());
        chunk.skip = 5;


        assert_eq! (5, chunk.skip);
        assert_eq! (&chunk[..], "6789".as_bytes ());

        chunk.wipe_skipped ();
        assert_eq! (0, chunk.skip);
        println! ("Chunk is '{}'", String::from_utf8 (chunk.clone ().to_vec ()).unwrap ());
        assert_eq! (&chunk[..], "6789".as_bytes ());
    }


    #[test]
    fn test_slice_reader () {
        let src = "1234567890";

        let mut reader = SliceReader::new (&src.as_bytes ());


        let chunk = reader.consume (4);
        assert_eq! (&chunk[..], "1234".as_bytes ());


        let chunk = reader.consume (4);
        assert_eq! (&chunk[..], "5678".as_bytes ());


        let chunk = reader.consume (4);
        assert_eq! (&chunk[..], "90".as_bytes ());


        let mut reader = SliceReader::new (&src.as_bytes ());

        assert_eq! (2, reader.skip (2));
        let chunk = reader.consume (4);
        assert_eq! (&chunk[..], "3456".as_bytes ());


        assert_eq! (true, reader.has (1));
        assert_eq! (true, reader.has (2));
        assert_eq! (true, reader.has (4));
        assert_eq! (false, reader.has (5));


        assert_eq! (2, reader.skip (2));
        let chunk = reader.consume (4);
        assert_eq! (&chunk[..], "90".as_bytes ());

        assert_eq! (false, reader.has (1));

        assert_eq! (0, reader.skip (2));
        let chunk = reader.consume (4);
        assert_eq! (&chunk[..], "".as_bytes ());


        let mut reader = SliceReader::new (&src.as_bytes ());

        if let Some (slice) = reader.slice (2) {
            assert_eq! (slice, "12".as_bytes ());
        } else { assert! (false, "Unexpected result!"); }

        if let Some (slice) = reader.slice_at (2, 2) {
            assert_eq! (slice, "34".as_bytes ());
        } else { assert! (false, "Unexpected result!"); }

        assert_eq! (2, reader.skip (2));

        if let Some (slice) = reader.slice (4) {
            assert_eq! (slice, "3456".as_bytes ());
        } else { assert! (false, "Unexpected result!"); }

        if let Some (slice) = reader.slice_at (2, 2) {
            assert_eq! (slice, "56".as_bytes ());
        } else { assert! (false, "Unexpected result!"); }

        if let Some (slice) = reader.slice_at (7, 1) {
            assert_eq! (slice, "0".as_bytes ());
        } else { assert! (false, "Unexpected result!"); }

        if let Some (_) = reader.slice_at (8, 1) {
            assert! (false, "Unexpected result!")
        }
    }


    #[test]
    fn test_feed_shift () {
        let src = "1234567890";

        let mut feed = Feed::with_capacity (src.len ());
        feed.push_slice (src.as_bytes ());

        assert_eq! (&&feed.shift (10)[..], &"1234567890".as_bytes ());


        let mut feed = Feed::with_capacity (src.len ());
        feed.push_slice (src.as_bytes ());

        assert_eq! (&&feed.shift (4)[..], &"1234".as_bytes ());


        let mut feed = Feed::with_capacity (src.len ());
        feed.push_slice (src.as_bytes ());

        assert_eq! (&&feed.shift (8)[..], &"12345678".as_bytes ());


        let mut feed = Feed::with_capacity (src.len ());
        feed.push_slice (src.as_bytes ());

        assert_eq! (&&feed.shift (0)[..], &"".as_bytes ());
    }


    #[test]
    fn test_feed_skip () {
        let src = "1234567890";

        let mut feed = Feed::with_capacity (src.len ());
        feed.push_slice (src.as_bytes ());

        assert_eq! (&&feed[..], &"1234567890".as_bytes ());

        assert_eq! (1, feed.skip (1));
        assert_eq! (&&feed[..], &"234567890".as_bytes ());

        assert_eq! (9, feed.skip (20));
        assert_eq! (&&feed[..], &"".as_bytes ());

        assert_eq! (0, feed.skip (1));
        assert_eq! (&&feed[..], &"".as_bytes ());
    }


    #[test]
    fn test_stream_reader () {
        let src = "1234567890";

        let mut reader = StreamReader::new (src.as_bytes ());

        assert! (reader.has (2));

        if let Some (slice) = reader.slice (2) {
            assert_eq! (slice, "12".as_bytes ());
        } else { assert! (false, "Unexpected result!"); }


        assert_eq! (2, reader.skip (2));

        assert! (reader.has (2));

        if let Some (slice) = reader.slice (2) {
            assert_eq! (slice, "34".as_bytes ());
        } else { assert! (false, "Unexpected result!"); }


        assert_eq! (&reader.consume (4)[..], "3456".as_bytes ());


        assert! (reader.has (2));

        if let Some (slice) = reader.slice (2) {
            assert_eq! (slice, "78".as_bytes ());
        } else { assert! (false, "Unexpected result!"); }


        assert_eq! (&reader.consume (5)[..], "7890".as_bytes ());
        assert_eq! (0, reader.skip (4));
        assert_eq! (&reader.consume (4)[..], "".as_bytes ());
    }
}
