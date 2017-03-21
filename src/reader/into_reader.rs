use reader::{ Read, BytesReader, SliceReader };



pub trait IntoReader where Self::Reader: Read {
    type Reader;

    fn into_reader (self) -> Self::Reader;
}



impl IntoReader for &'static str {
    type Reader = SliceReader;

    fn into_reader (self) -> SliceReader { SliceReader::new (self.as_bytes ()) }
}



impl IntoReader for String {
    type Reader = BytesReader;

    fn into_reader (self) -> BytesReader { BytesReader::new (self.into_bytes ()) }
}



impl IntoReader for Vec<u8> {
    type Reader = BytesReader;

    fn into_reader (self) -> BytesReader { BytesReader::new (self) }
}



/*
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
*/
