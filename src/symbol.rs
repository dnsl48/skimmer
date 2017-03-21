use reader::Read;

use std::ptr;
use std::mem;


pub trait Symbol {
    fn same_as_slice (&self, src: &[u8]) -> bool { self.as_slice () == src }

    fn as_slice (&self) -> &[u8];

    fn as_ptr (&self) -> *const u8;

    fn new_vec (&self) -> Vec<u8>;

    fn to_vec (self) -> Vec<u8>;

    fn len (&self) -> usize;

    fn len_chars (&self) -> usize;

    fn contained_at (&self, value: &[u8], index: usize) -> bool;

    fn contained_at_start (&self, value: &[u8]) -> bool { self.contained_at (value, 0) }

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



pub trait CopySymbol : Symbol + Copy + Send + Sync {
    fn len (self) -> usize;

    fn len_chars (self) -> usize;

    fn contained_at (self, value: &[u8], index: usize) -> bool;

    fn contained_at_start (self, value: &[u8]) -> bool { self.contained_at (value, 0) }

    fn read<Reader: Read> (self, reader: &mut Reader) -> Option<usize> where Self: Sized { self.read_at (0, reader) }

    fn read_at<Reader: Read> (self, at: usize, reader: &mut Reader) -> Option<usize> {
        if reader.contains_copy_at (self, at) {
            Some (self.len ())
        } else {
            None
        }
    }

    unsafe fn copy_to_ptr (self, dst: *mut u8) -> *mut u8;

    unsafe fn copy_to_ptr_times (self, dst: *mut u8, times: usize) -> *mut u8;
}




pub trait Combo {
    fn combine<S: Symbol> (from: &[S]) -> Self;
}




#[derive (Clone, Debug)]
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
    fn as_slice (&self) -> &[u8] { &self.dt[0 .. self.len ()] }

    fn as_ptr (&self) -> *const u8 { self.dt.as_ptr () }

    fn to_vec (self) -> Vec<u8> { self.new_vec () }

    fn new_vec (&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity (self.len ());
        vec.extend (self.as_slice ());
        vec
    }

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
}



#[derive (Copy, Clone, Debug)]
#[repr(C)]
pub struct Char1 {
    data: u8
}


impl Char1 {
    pub fn new (src: u8) -> Char1 { Char1 { data: src } }

    pub fn new_word (&self) -> Word { Word::combine (&[self]) }

    pub fn to_word (self) -> Word { Word::combine (&[&self]) }
}


impl Symbol for Char1 {
    fn as_slice (&self) -> &[u8] { unsafe { mem::transmute::<&u8, &[u8 ; 1]> (&self.data) } }

    fn as_ptr (&self) -> *const u8 { &self.data as *const u8 }

    fn to_vec (self) -> Vec<u8> { self.new_vec () }

    fn new_vec (&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity (self.len ());
        vec.extend (self.as_slice ());
        vec
    }

    fn len (&self) -> usize { 1 }

    fn len_chars (&self) -> usize { 1 }

    fn contained_at (&self, value: &[u8], index: usize) -> bool { CopySymbol::contained_at (*self, value, index) }

    fn contained_at_start (&self, value: &[u8]) -> bool { CopySymbol::contained_at_start (*self, value) }

    unsafe fn copy_to_ptr (&self, dst: *mut u8) -> *mut u8 { CopySymbol::copy_to_ptr (*self, dst) }

    unsafe fn copy_to_ptr_times (&self, dst: *mut u8, times: usize) -> *mut u8 { CopySymbol::copy_to_ptr_times (*self, dst, times) }
}


impl CopySymbol for Char1 {
    #[inline (always)]
    fn len (self) -> usize { 1 }

    #[inline (always)]
    fn len_chars (self) -> usize { 1 }

    #[inline (always)]
    fn contained_at (self, value: &[u8], index: usize) -> bool { value.len () >= index + 1 && value[index] == self.data }

    #[inline (always)]
    fn contained_at_start (self, value: &[u8]) -> bool { value.len () > 0 && value[0] == self.data }

    #[inline (always)]
    unsafe fn copy_to_ptr (self, dst: *mut u8) -> *mut u8 {
        *dst = self.data;
        dst.offset (1)
    }

    #[inline (always)]
    unsafe fn copy_to_ptr_times (self, mut dst: *mut u8, mut times: usize) -> *mut u8 {
        loop {
            if times == 0 { break }
            *dst = self.data;
            dst = dst.offset (1);
            times -= 1;
        }

        dst
    }
}





#[derive (Copy, Clone, Debug)]
#[repr(C)]
pub struct Char2 {
    unu: u8,
    dua: u8
}



impl Char2 {
    pub fn new (src: &[u8]) -> Char2 {
        match src.len () {
            2 => Char2 { unu: src[0], dua: src[1] },
            _ => panic! ("Invalid number of bytes for a Char2: {} != 2", src.len ())
        }
    }

    pub fn new_word (&self) -> Word { Word::combine (&[self]) }

    pub fn to_word (self) -> Word { Word::combine (&[&self]) }
}



impl Symbol for Char2 {
    fn as_slice (&self) -> &[u8] { unsafe { mem::transmute::<&Char2, &[u8 ; 2]> (self) } }

    fn as_ptr (&self) -> *const u8 { &self.unu as *const u8 }

    fn to_vec (self) -> Vec<u8> { self.new_vec () }

    fn new_vec (&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity (self.len ());
        vec.extend (self.as_slice ());
        vec
    }

    fn len (&self) -> usize { 2 }

    fn len_chars (&self) -> usize { 1 }

    fn contained_at (&self, value: &[u8], index: usize) -> bool { CopySymbol::contained_at (*self, value, index) }

    fn contained_at_start (&self, value: &[u8]) -> bool { CopySymbol::contained_at_start (*self, value) }
}



impl CopySymbol for Char2 {
    #[inline (always)]
    fn len (self) -> usize { 2 }

    #[inline (always)]
    fn len_chars (self) -> usize { 1 }

    #[inline (always)]
    fn contained_at (self, value: &[u8], index: usize) -> bool {
        value.len () >= index + 2 && unsafe {
            let v = value.as_ptr ().offset (index as isize);

            *v == self.unu && *v.offset (1) == self.dua
        }
    }

    #[inline (always)]
    fn contained_at_start (self, value: &[u8]) -> bool {
        value.len () >= 2 && unsafe {
            *value.as_ptr () == self.unu &&
            *value.as_ptr ().offset (1) == self.dua
        }
    }

    #[inline (always)]
    unsafe fn copy_to_ptr (self, dst: *mut u8) -> *mut u8 {
        *dst = self.unu;
        dst.offset (1);
        *dst = self.dua;
        dst.offset (1)
    }

    #[inline (always)]
    unsafe fn copy_to_ptr_times (self, mut dst: *mut u8, mut times: usize) -> *mut u8 {
        loop {
            if times == 0 { break }

            *dst = self.unu;
            dst = dst.offset (1);
            *dst = self.dua;
            dst = dst.offset (1);

            times -= 1;
        }

        dst
    }
}



impl Combo for Char2 {
    fn combine<S: Symbol> (from: &[S]) -> Self {
        if from.len () != 2 { panic! ("Combo length should be = 2, {} given", from.len ()) }
        if from[0].len () != 1 || from[1].len () != 1 { panic! ("Combo length should be = 2, {} given", from[0].len () + from[1].len ()) }
        Char2::new (&[from[0].as_slice ()[0], from[1].as_slice ()[0]])
    }
}





#[derive (Copy, Clone, Debug)]
#[repr(C)]
pub struct Char3 {
    unu: u8,
    dua: u8,
    tri: u8
}



impl Char3 {
    pub fn new (src: &[u8]) -> Char3 {
        match src.len () {
            3 => Char3 { unu: src[0], dua: src[1], tri: src[2] },
            _ => panic! ("Invalid number of bytes for a Char3: {} != 3", src.len ())
        }
    }

    pub fn new_word (&self) -> Word { Word::combine (&[self]) }

    pub fn to_word (self) -> Word { Word::combine (&[&self]) }
}



impl Symbol for Char3 {
    fn as_slice (&self) -> &[u8] { unsafe { mem::transmute::<&Char3, &[u8 ; 3]> (self) } }

    fn as_ptr (&self) -> *const u8 { &self.unu as *const u8 }

    fn to_vec (self) -> Vec<u8> { self.new_vec () }

    fn new_vec (&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity (self.len ());
        vec.extend (self.as_slice ());
        vec
    }

    fn len (&self) -> usize { 3 }

    fn len_chars (&self) -> usize { 1 }

    fn contained_at (&self, value: &[u8], index: usize) -> bool { CopySymbol::contained_at (*self, value, index) }

    fn contained_at_start (&self, value: &[u8]) -> bool { CopySymbol::contained_at_start (*self, value) }
}



impl CopySymbol for Char3 {
    #[inline (always)]
    fn len (self) -> usize { 3 }

    #[inline (always)]
    fn len_chars (self) -> usize { 1 }

    #[inline (always)]
    fn contained_at (self, value: &[u8], index: usize) -> bool {
        value.len () >= index + 3 && unsafe {
            let v = value.as_ptr ().offset (index as isize);

            *v == self.unu &&
            *v.offset (1) == self.dua &&
            *v.offset (2) == self.tri
        }
    }

    #[inline (always)]
    fn contained_at_start (self, value: &[u8]) -> bool {
        value.len () >= 3 && unsafe {
            *value.as_ptr () == self.unu &&
            *value.as_ptr ().offset (1) == self.dua &&
            *value.as_ptr ().offset (2) == self.tri
        }
    }

    #[inline (always)]
    unsafe fn copy_to_ptr (self, dst: *mut u8) -> *mut u8 {
        *dst = self.unu;
        dst.offset (1);
        *dst = self.dua;
        dst.offset (1);
        *dst = self.tri;
        dst.offset (1)
    }

    #[inline (always)]
    unsafe fn copy_to_ptr_times (self, mut dst: *mut u8, mut times: usize) -> *mut u8 {
        loop {
            if times == 0 { break }

            *dst = self.unu;
            dst = dst.offset (1);
            *dst = self.dua;
            dst = dst.offset (1);
            *dst = self.tri;
            dst = dst.offset (1);

            times -= 1;
        }

        dst
    }
}


impl Combo for Char3 {
    fn combine<S: Symbol> (from: &[S]) -> Self {
        let len = from.iter ().fold (0, |t, s| t + s.len ());
        if len != 3 { panic! ("Combo length should be = 3, {} given", len) }
        let vec: Vec<u8> = from.iter ().flat_map (|s| s.as_slice ()).map (|s| *s).collect ();
        Char3::new (&vec)
    }
}



#[derive (Copy, Clone, Debug)]
#[repr(C)]
pub struct Char4 {
    unu: u8,
    dua: u8,
    tri: u8,
    kvar: u8
}



impl Char4 {
    pub fn new (src: &[u8]) -> Char4 {
        match src.len () {
            4 => Char4 { unu: src[0], dua: src[1], tri: src[2], kvar: src[3] },
            _ => panic! ("Invalid number of bytes for a Char4: {} != 4", src.len ())
        }
    }

    pub fn new_word (&self) -> Word { Word::combine (&[self]) }

    pub fn to_word (self) -> Word { Word::combine (&[&self]) }
}



impl Symbol for Char4 {
    fn as_slice (&self) -> &[u8] { unsafe { mem::transmute::<&Char4, &[u8 ; 4]> (self) } }

    fn as_ptr (&self) -> *const u8 { &self.unu as *const u8 }

    fn to_vec (self) -> Vec<u8> { self.new_vec () }

    fn new_vec (&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity (self.len ());
        vec.extend (self.as_slice ());
        vec
    }

    fn len (&self) -> usize { 4 }

    fn len_chars (&self) -> usize { 1 }

    fn contained_at (&self, value: &[u8], index: usize) -> bool { CopySymbol::contained_at (*self, value, index) }

    fn contained_at_start (&self, value: &[u8]) -> bool { CopySymbol::contained_at_start (*self, value) }
}



impl CopySymbol for Char4 {
    #[inline (always)]
    fn len (self) -> usize { 4 }

    #[inline (always)]
    fn len_chars (self) -> usize { 1 }

    #[inline (always)]
    fn contained_at (self, value: &[u8], index: usize) -> bool {
        value.len () >= index + 4 && unsafe {
            let v = value.as_ptr ().offset (index as isize);

            *v == self.unu &&
            *v.offset (1) == self.dua &&
            *v.offset (2) == self.tri &&
            *v.offset (3) == self.kvar
        }
    }

    #[inline (always)]
    fn contained_at_start (self, value: &[u8]) -> bool {
        value.len () >= 4 && unsafe {
            *value.as_ptr () == self.unu &&
            *value.as_ptr ().offset (1) == self.dua &&
            *value.as_ptr ().offset (2) == self.tri &&
            *value.as_ptr ().offset (3) == self.kvar
        }
    }

    #[inline (always)]
    unsafe fn copy_to_ptr (self, dst: *mut u8) -> *mut u8 {
        *dst = self.unu;
        dst.offset (1);
        *dst = self.dua;
        dst.offset (1);
        *dst = self.tri;
        dst.offset (1);
        *dst = self.kvar;
        dst.offset (1)
    }

    #[inline (always)]
    unsafe fn copy_to_ptr_times (self, mut dst: *mut u8, mut times: usize) -> *mut u8 {
        loop {
            if times == 0 { break }

            *dst = self.unu;
            dst = dst.offset (1);
            *dst = self.dua;
            dst = dst.offset (1);
            *dst = self.tri;
            dst = dst.offset (1);
            *dst = self.kvar;
            dst = dst.offset (1);

            times -= 1;
        }

        dst
    }
}


impl Combo for Char4 {
    fn combine<S: Symbol> (from: &[S]) -> Self {
        let len = from.iter ().fold (0, |t, s| t + s.len ());
        if len != 4 { panic! ("Combo length should be = 3, {} given", len) }
        let vec: Vec<u8> = from.iter ().flat_map (|s| s.as_slice ()).map (|s| *s).collect ();
        Char4::new (&vec)
    }
}




#[derive (Clone, Debug)]
pub struct Word {
    dt: Vec<u8>,
    clen: usize
}



impl Word {
    pub fn empty () -> Word { Word { dt: Vec::with_capacity (0), clen: 0 } }


    pub fn combine (src: &[&Symbol]) -> Word {
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
    fn as_slice (&self) -> &[u8] { &self.dt[..] }

    fn as_ptr (&self) -> *const u8 { self.dt.as_ptr () }

    // fn same_as_slice (&self, src: &[u8]) -> bool { &self.dt[..] == src }

    fn new_vec (&self) -> Vec<u8> { self.dt.clone () }

    fn to_vec (self) -> Vec<u8> { self.dt }

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

    fn contained_at_start (&self, value: &[u8]) -> bool {
        value.len () >= self.len () && unsafe {
            let mut v = value.as_ptr ();
            let mut s = self.dt.as_ptr ();

            for _ in 0..self.len () {
                if *v != *s { return false; }
                v = v.offset (1);
                s = s.offset (1);
            }

            true
        }
    }
}



impl Combo for Word {
    fn combine<S: Symbol> (from: &[S]) -> Self {
        let len = from.iter ().fold (0, |t, s| t + s.len ());
        let clen = from.iter ().fold (0, |t, s| t + s.len_chars ());

        if len == 0 { panic! ("Combo length should be > 0, {} given", len) }
        let vec: Vec<u8> = from.iter ().flat_map (|s| s.as_slice ()).map (|s| *s).collect ();

        Word { dt: vec, clen: clen }
    }
}



#[derive (Clone, Debug)]
pub enum Rune {
    Char (Char),
    Chr1 (Char1),
    Chr2 (Char2),
    Chr3 (Char3),
    Chr4 (Char4),
    Word (Word)
}



impl Symbol for Rune {
    fn as_slice (&self) -> &[u8] {
        match *self {
            Rune::Char (ref c) => c.as_slice (),
            Rune::Chr1 (ref c) => c.as_slice (),
            Rune::Chr2 (ref c) => c.as_slice (),
            Rune::Chr3 (ref c) => c.as_slice (),
            Rune::Chr4 (ref c) => c.as_slice (),
            Rune::Word (ref w) => w.as_slice ()
        }
    }

    fn as_ptr (&self) -> *const u8 {
        match *self {
            Rune::Char (ref c) => c.as_ptr (),
            Rune::Chr1 (ref c) => c.as_ptr (),
            Rune::Chr2 (ref c) => c.as_ptr (),
            Rune::Chr3 (ref c) => c.as_ptr (),
            Rune::Chr4 (ref c) => c.as_ptr (),
            Rune::Word (ref w) => w.as_ptr ()
        }
    }

    fn same_as_slice (&self, src: &[u8]) -> bool {
        match *self {
            Rune::Char (ref c) => c.same_as_slice (src),
            Rune::Chr1 (ref c) => c.same_as_slice (src),
            Rune::Chr2 (ref c) => c.same_as_slice (src),
            Rune::Chr3 (ref c) => c.same_as_slice (src),
            Rune::Chr4 (ref c) => c.same_as_slice (src),
            Rune::Word (ref w) => w.same_as_slice (src)
        }
    }

    fn new_vec (&self) -> Vec<u8> {
        match *self {
            Rune::Char (ref c) => c.new_vec (),
            Rune::Chr1 (ref c) => c.new_vec (),
            Rune::Chr2 (ref c) => c.new_vec (),
            Rune::Chr3 (ref c) => c.new_vec (),
            Rune::Chr4 (ref c) => c.new_vec (),
            Rune::Word (ref w) => w.new_vec ()
        }
    }

    fn to_vec (self) -> Vec<u8> {
        match self {
            Rune::Char (c) => c.to_vec (),
            Rune::Chr1 (c) => c.to_vec (),
            Rune::Chr2 (c) => c.to_vec (),
            Rune::Chr3 (c) => c.to_vec (),
            Rune::Chr4 (c) => c.to_vec (),
            Rune::Word (w) => w.to_vec ()
        }
    }

        fn len (&self) -> usize {
        match *self {
            Rune::Char (ref c) => c.len (),
            Rune::Chr1 (ref c) => c.len (),
            Rune::Chr2 (ref c) => c.len (),
            Rune::Chr3 (ref c) => c.len (),
            Rune::Chr4 (ref c) => c.len (),
            Rune::Word (ref w) => w.len ()
        }
    }

    fn len_chars (&self) -> usize {
        match *self {
            Rune::Char (ref c) => c.len_chars (),
            Rune::Chr1 (ref c) => c.len_chars (),
            Rune::Chr2 (ref c) => c.len_chars (),
            Rune::Chr3 (ref c) => c.len_chars (),
            Rune::Chr4 (ref c) => c.len_chars (),
            Rune::Word (ref w) => w.len_chars ()
        }
    }

    fn contained_at (&self, value: &[u8], index: usize) -> bool {
        match *self {
            Rune::Char (ref c) => c.contained_at (value, index),
            Rune::Chr1 (ref c) => c.contained_at (value, index),
            Rune::Chr2 (ref c) => c.contained_at (value, index),
            Rune::Chr3 (ref c) => c.contained_at (value, index),
            Rune::Chr4 (ref c) => c.contained_at (value, index),
            Rune::Word (ref w) => w.contained_at (value, index)
        }
    }

    fn contained_at_start (&self, value: &[u8]) -> bool {
        match *self {
            Rune::Char (ref c) => c.contained_at_start (value),
            Rune::Chr1 (ref c) => c.contained_at_start (value),
            Rune::Chr2 (ref c) => c.contained_at_start (value),
            Rune::Chr3 (ref c) => c.contained_at_start (value),
            Rune::Chr4 (ref c) => c.contained_at_start (value),
            Rune::Word (ref w) => w.contained_at_start (value)
        }
    }
}



impl From<Char> for Rune {
    fn from (c: Char) -> Rune { Rune::Char (c) }
}


impl From<Char1> for Rune {
    fn from (c: Char1) -> Rune { Rune::Chr1 (c) }
}


impl From<Char2> for Rune {
    fn from (c: Char2) -> Rune { Rune::Chr2 (c) }
}


impl From<Char3> for Rune {
    fn from (c: Char3) -> Rune { Rune::Chr3 (c) }
}


impl From<Char4> for Rune {
    fn from (c: Char4) -> Rune { Rune::Chr4 (c) }
}


impl From<Word> for Rune {
    fn from (w: Word) -> Rune { Rune::Word (w) }
}




impl<C1, C2> Symbol for Result<C1, C2> where C1: Symbol, C2: Symbol {
    fn as_slice (&self) -> &[u8] { match *self {
        Ok (ref c) => c.as_slice (),
        Err (ref c) => c.as_slice ()
    } }

    fn as_ptr (&self) -> *const u8 { match *self {
        Ok (ref c) => c.as_ptr (),
        Err (ref c) => c.as_ptr ()
    } }

    fn new_vec (&self) -> Vec<u8> { match *self {
        Ok (ref c) => c.new_vec (),
        Err (ref c) => c.new_vec ()
    } }

    fn to_vec (self) -> Vec<u8> { match self {
        Ok (c) => c.to_vec (),
        Err (c) => c.to_vec ()
    } }

    fn len (&self) -> usize { match *self {
        Ok (ref c) => c.len (),
        Err (ref c) => c.len ()
    } }

    fn len_chars (&self) -> usize { match *self {
        Ok (ref c) => c.len_chars (),
        Err (ref c) => c.len_chars ()
    } }

    fn contained_at (&self, value: &[u8], index: usize) -> bool { match *self {
        Ok (ref c) => c.contained_at (value, index),
        Err (ref c) => c.contained_at (value, index)
    } }
}




impl<C1, C2> CopySymbol for Result<C1, C2> where C1: CopySymbol, C2: CopySymbol {
    #[inline (always)]
    fn len (self) -> usize { match self {
        Ok (c) => c.len (),
        Err (c) => c.len ()
    } }

    #[inline (always)]
    fn len_chars (self) -> usize { match self {
        Ok (c) => c.len_chars (),
        Err (c) => c.len_chars ()
    } }

    #[inline (always)]
    fn contained_at (self, value: &[u8], index: usize) -> bool { match self {
        Ok (c) => c.contained_at (value, index),
        Err (c) => c.contained_at (value, index)
    } }

    #[inline (always)]
    fn contained_at_start (self, value: &[u8]) -> bool { match self {
        Ok (c) => c.contained_at_start (value),
        Err (c) => c.contained_at_start (value)
    } }

    #[inline (always)]
    fn read<Reader: Read> (self, reader: &mut Reader) -> Option<usize> where Self: Sized { match self {
        Ok (c) => c.read (reader),
        Err (c) => c.read (reader)
    } }

    #[inline (always)]
    fn read_at<Reader: Read> (self, at: usize, reader: &mut Reader) -> Option<usize> { match self {
        Ok (c) => c.read_at (at, reader),
        Err (c) => c.read_at (at, reader)
    } }

    #[inline (always)]
    unsafe fn copy_to_ptr (self, dst: *mut u8) -> *mut u8 { match self {
        Ok (c) => c.copy_to_ptr (dst),
        Err (c) => c.copy_to_ptr (dst)
    } }

    #[inline (always)]
    unsafe fn copy_to_ptr_times (self, dst: *mut u8, times: usize) -> *mut u8 { match self {
        Ok (c) => c.copy_to_ptr_times (dst, times),
        Err (c) => c.copy_to_ptr_times (dst, times)
    } }
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
