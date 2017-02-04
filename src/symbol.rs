use reader::Read;

use std::ptr;



pub trait Symbol {
    fn len (&self) -> usize;

    fn len_chars (&self) -> usize;

    fn contained_at (&self, value: &[u8], index: usize) -> bool;

    fn same_as_slice (&self, &[u8]) -> bool;

    fn as_slice (&self) -> &[u8];

    fn as_ptr (&self) -> *const u8;

    fn new_vec (&self) -> Vec<u8>;

    fn to_vec (self) -> Vec<u8>;

    fn read<Reader: Read> (&self, reader: &mut Reader) -> Option<usize> where Self: Sized { self.read_at (0, reader) }

    fn read_at<Reader: Read> (&self, at: usize, reader: &mut Reader) -> Option<usize> where Self: Sized {
        if reader.contains_at (self, at) {
            Some (self.len ())
        } else {
            None
        }
    }

    unsafe fn copy_to_ptr (&self, dst: *mut u8) -> *mut u8 {
        let src = self.as_ptr ();
        let len = self.len ();

        ptr::copy_nonoverlapping (src, dst, len);

        dst.offset (len as isize)
    }

    unsafe fn copy_to_ptr_times (&self, mut dst: *mut u8, times: usize) -> *mut u8 {
        let src = self.as_ptr ();
        let len = self.len ();

        for _ in 0 .. times {
            ptr::copy_nonoverlapping (src, dst, len);
            dst = dst.offset (len as isize);
        }

        dst
    }
}




#[derive (Clone)]
pub struct Char {
    dt: [u8; 9]
}



impl Char {
    pub fn new (src: &[u8]) -> Char {
        Char { dt: match src.len () {
            1  => [src[0], 0, 0, 0, 0, 0, 0, 0, 1],
            2  => [src[0], src[1], 0, 0, 0, 0, 0, 0, 2],
            3  => [src[0], src[1], src[2], 0, 0, 0, 0, 0, 3],
            4  => [src[0], src[1], src[2], src[3], 0, 0, 0, 0, 4],
            5  => [src[0], src[1], src[2], src[3], src[4], 0, 0, 0, 5],
            6  => [src[0], src[1], src[2], src[3], src[4], src[5], 0, 0, 6],
            7  => [src[0], src[1], src[2], src[3], src[4], src[5], src[6], 0, 7],
            8  => [src[0], src[1], src[2], src[3], src[4], src[5], src[6], src[7], 8],
            _ => panic! ("Invalid number of bytes for a Char: 0 > {} < 9", src.len ())
        }}
    }

    pub fn new_word (&self) -> Word { Word::combine (&[self]) }

    pub fn to_word (self) -> Word { Word::combine (&[&self]) }
}



impl Symbol for Char {
    fn len (&self) -> usize { self.dt[8] as usize }

    fn len_chars (&self) -> usize { 1 }

    fn contained_at (&self, value: &[u8], index: usize) -> bool {
        value.len () >= index + self.len () && unsafe {
            let mut v = value.as_ptr ().offset (index as isize);
            let mut s = self.dt.as_ptr ();

            for _ in 0..self.len () {
                if *v != *s { return false; }
                v = v.offset (1);
                s = s.offset (1);
            }

            true
        }
    }

    fn as_slice (&self) -> &[u8] { &self.dt[0 .. self.len ()] }

    fn as_ptr (&self) -> *const u8 { self.dt.as_ptr () }

    fn same_as_slice (&self, src: &[u8]) -> bool { self.as_slice () == src }

    fn to_vec (self) -> Vec<u8> { self.new_vec () }

    fn new_vec (&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity (self.len ());
        vec.extend (self.as_slice ());
        vec
    }
}




#[derive (Clone)]
pub struct Word {
    dt: Vec<u8>,
    clen: usize
}



impl Word {
    pub fn empty () -> Word { Word { dt: Vec::with_capacity (0), clen: 0 } }


    pub fn combine (src: &[&Char]) -> Word {
        if src.len () == 0 { panic! ("Word length must be > 0") }

        let mut vlen: usize = 0;

        for idx in 0 .. src.len () { vlen += src[idx].len () }

        let mut vec = Vec::with_capacity (vlen);

        for i in 0 .. src.len () {
            vec.extend (src[i].as_slice ());
        }

        Word { dt: vec, clen: src.len () }
    }


    pub fn concat (src: &[&Word]) -> Word {
        let mut v_len: usize = 0;
        let mut c_len: usize = 0;

        for idx in 0 .. src.len () {
            v_len += src[idx].len ();
            c_len += src[idx].len_chars ();
        }

        let mut vec = Vec::with_capacity (v_len);

        for i in 0 .. src.len () {
            for j in 0 .. src[i].len () {
                vec.push (src[i].dt[j])
            }
        }

        Word { dt: vec, clen: c_len }
    }

    pub fn as_vec (&self) -> &Vec<u8> { &self.dt }
}



impl Symbol for Word {
    fn len (&self) -> usize { self.dt.len () }

    fn len_chars (&self) -> usize { self.clen }

    fn contained_at (&self, value: &[u8], index: usize) -> bool {
        value.len () >= index + self.len () && unsafe {
            let mut v = value.as_ptr ().offset (index as isize);
            let mut s = self.dt.as_ptr ();

            for _ in 0..self.len () {
                if *v != *s { return false; }
                v = v.offset (1);
                s = s.offset (1);
            }

            true
        }
    }

    fn as_slice (&self) -> &[u8] { &self.dt[..] }

    fn as_ptr (&self) -> *const u8 { self.dt.as_ptr () }

    fn same_as_slice (&self, src: &[u8]) -> bool { &self.dt[..] == src }

    fn new_vec (&self) -> Vec<u8> { self.dt.clone () }

    fn to_vec (self) -> Vec<u8> { self.dt }
}



#[derive (Clone)]
pub enum Rune {
    Char (Char),
    Word (Word)
}


impl Symbol for Rune {
    fn len (&self) -> usize {
        match *self {
            Rune::Char (ref c) => c.len (),
            Rune::Word (ref w) => w.len ()
        }
    }

    fn len_chars (&self) -> usize {
        match *self {
            Rune::Char (ref c) => c.len_chars (),
            Rune::Word (ref w) => w.len_chars ()
        }
    }

    fn contained_at (&self, value: &[u8], index: usize) -> bool {
        match *self {
            Rune::Char (ref c) => c.contained_at (value, index),
            Rune::Word (ref w) => w.contained_at (value, index)
        }
    }

    fn as_slice (&self) -> &[u8] {
        match *self {
            Rune::Char (ref c) => c.as_slice (),
            Rune::Word (ref w) => w.as_slice ()
        }
    }

    fn as_ptr (&self) -> *const u8 {
        match *self {
            Rune::Char (ref c) => c.as_ptr (),
            Rune::Word (ref w) => w.as_ptr ()
        }
    }

    fn same_as_slice (&self, src: &[u8]) -> bool {
        match *self {
            Rune::Char (ref c) => c.same_as_slice (src),
            Rune::Word (ref w) => w.same_as_slice (src)
        }
    }

    fn new_vec (&self) -> Vec<u8> {
        match *self {
            Rune::Char (ref c) => c.new_vec (),
            Rune::Word (ref w) => w.new_vec ()
        }
    }

    fn to_vec (self) -> Vec<u8> {
        match self {
            Rune::Char (c) => c.to_vec (),
            Rune::Word (w) => w.to_vec ()
        }
    }
}


impl From<Char> for Rune {
    fn from (c: Char) -> Rune { Rune::Char (c) }
}


impl From<Word> for Rune {
    fn from (w: Word) -> Rune { Rune::Word (w) }
}




#[cfg (test)]
mod tests {
    use super::*;


    #[test]
    #[should_panic (expected = "Invalid number of bytes for a Char: 0 > 0 < 9")]
    fn cha0 () {
        Char::new (&[]);
    }


    #[test]
    fn cha1 () {
        let cha = Char::new (&[b'a']);

        assert_eq! (cha.len (), 1);
        assert_eq! (cha.as_slice (), &[b'a']);
    }


    #[test]
    fn cha2 () {
        let cha = Char::new (&[b'a', b'b']);

        assert_eq! (cha.len (), 2);
        assert_eq! (cha.as_slice (), &[b'a', b'b']);
    }


    #[test]
    fn cha3 () {
        let cha = Char::new (&[b'a', b'b', b'c']);

        assert_eq! (cha.len (), 3);
        assert_eq! (cha.as_slice (), &[b'a', b'b', b'c']);
    }

    #[test]
    fn cha4 () {
        let cha = Char::new (&[b'a', b'b', b'c', b'd']);

        assert_eq! (cha.len (), 4);
        assert_eq! (cha.as_slice (), &[b'a', b'b', b'c', b'd']);
    }


    #[test]
    fn cha5 () {
        let cha = Char::new (&[b'a', b'b', b'c', b'd', b'e']);

        assert_eq! (cha.len (), 5);
        assert_eq! (cha.as_slice (), &[b'a', b'b', b'c', b'd', b'e']);
    }


    #[test]
    fn cha6 () {
        let cha = Char::new (&[b'a', b'b', b'c', b'd', b'e', b'f']);
        assert_eq! (cha.len (), 6);
        assert_eq! (cha.as_slice (), &[b'a', b'b', b'c', b'd', b'e', b'f']);
    }


    #[test]
    fn cha7 () {
        let cha = Char::new (&[b'a', b'b', b'c', b'd', b'e', b'f', b'g']);
        assert_eq! (cha.len (), 7);
        assert_eq! (cha.as_slice (), &[b'a', b'b', b'c', b'd', b'e', b'f', b'g']);
    }


    #[test]
    fn cha8 () {
        let cha = Char::new (&[b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h']);
        assert_eq! (cha.len (), 8);
        assert_eq! (cha.as_slice (), &[b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h']);
    }


    #[test]
    #[should_panic (expected = "Invalid number of bytes for a Char: 0 > 9 < 9")]
    fn cha9 () {
        Char::new (&[b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i']);
    }


    #[test]
    #[should_panic (expected = "Word length must be > 0")]
    fn word0 () {
        Word::combine (&[]);
    }


    #[test]
    fn word1 () {
        let word = Word::combine (&[
            &Char::new (&[b'a', b'b', b'c', b'd']),
            &Char::new (&[b'1'])
        ]);

        assert_eq! (word.len (), 5);
        assert_eq! (word.len_chars (), 2);
        assert_eq! (word.as_vec ()[..], vec! [b'a', b'b', b'c', b'd', b'1'][..]);

        assert! (word.same_as_slice (&[b'a', b'b', b'c', b'd', b'1']));

        assert! (! word.same_as_slice (&[b'a', b'b', b'c', b'd', b'2']));
        assert! (! word.same_as_slice (&[b'a', b'b', b'c', b'd', b'1', b'2']));
    }



    #[test]
    fn word2 () {
        let w1 = Word::combine (&[
            &Char::new (&[b'a', b'b', b'c']),
            &Char::new (&[b'1'])
        ]);

        let w2 = Char::new (&[b'd', b'e']).to_word ();

        let w = Word::concat (&[&w1, &w2]);

        assert_eq! (w.len (), 6);
        assert_eq! (w.len_chars (), 3);
        assert_eq! (w.as_vec()[..], vec! [b'a', b'b', b'c', b'1', b'd', b'e'][..]);

        assert! (w.same_as_slice (&[b'a', b'b', b'c', b'1', b'd', b'e']));
    }
}
