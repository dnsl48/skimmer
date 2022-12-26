use crate::reader::{BytesReader, IoreadeofReader, Read, SliceReader};

use std::fs;
use std::io;
use std::net;

#[cfg(unix)]
use std::os::unix::net::UnixStream;

pub trait IntoReader
where
    Self::Reader: Read,
{
    type Reader;

    fn into_reader(self) -> Self::Reader;
}

impl IntoReader for &'static str {
    type Reader = SliceReader;

    fn into_reader(self) -> SliceReader {
        SliceReader::new(self.as_bytes())
    }
}

impl IntoReader for String {
    type Reader = BytesReader;

    fn into_reader(self) -> BytesReader {
        BytesReader::new(self.into_bytes())
    }
}

impl IntoReader for Vec<u8> {
    type Reader = BytesReader;

    fn into_reader(self) -> BytesReader {
        let s = String::from_utf8(self);

        let bytes = match s {
            Ok(s) => s.into_bytes(),
            Err(_) => Vec::new(),
        };

        BytesReader::new(bytes)
    }
}

impl IntoReader for fs::File {
    type Reader = IoreadeofReader;

    fn into_reader(self) -> IoreadeofReader {
        IoreadeofReader::new(self)
    }
}

impl<'a> IntoReader for &'a fs::File {
    type Reader = IoreadeofReader;

    fn into_reader(self) -> IoreadeofReader {
        IoreadeofReader::new(self)
    }
}

impl IntoReader for net::TcpStream {
    type Reader = IoreadeofReader;

    fn into_reader(self) -> IoreadeofReader {
        IoreadeofReader::new(self)
    }
}

impl<'a> IntoReader for &'a net::TcpStream {
    type Reader = IoreadeofReader;

    fn into_reader(self) -> IoreadeofReader {
        IoreadeofReader::new(self)
    }
}

#[cfg(unix)]
impl IntoReader for UnixStream {
    type Reader = IoreadeofReader;

    fn into_reader(self) -> IoreadeofReader {
        IoreadeofReader::new(self)
    }
}

#[cfg(unix)]
impl<'a> IntoReader for &'a UnixStream {
    type Reader = IoreadeofReader;

    fn into_reader(self) -> IoreadeofReader {
        IoreadeofReader::new(self)
    }
}

impl<R> IntoReader for io::BufReader<R>
where
    R: io::Read,
{
    type Reader = IoreadeofReader;

    fn into_reader(self) -> IoreadeofReader {
        IoreadeofReader::new(self)
    }
}

impl<T> IntoReader for io::Cursor<T>
where
    T: AsRef<[u8]>,
{
    type Reader = IoreadeofReader;

    fn into_reader(self) -> IoreadeofReader {
        IoreadeofReader::new(self)
    }
}

impl<'a, R> IntoReader for &'a mut R
where
    R: io::Read + ?Sized,
{
    type Reader = IoreadeofReader;

    fn into_reader(self) -> IoreadeofReader {
        IoreadeofReader::new(self)
    }
}

impl<R> IntoReader for Box<R>
where
    R: io::Read + ?Sized,
{
    type Reader = IoreadeofReader;

    fn into_reader(self) -> IoreadeofReader {
        IoreadeofReader::new(self)
    }
}

impl<'a> IntoReader for &'a [u8] {
    type Reader = IoreadeofReader;

    fn into_reader(self) -> IoreadeofReader {
        IoreadeofReader::new(self)
    }
}
