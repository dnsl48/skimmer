use crate::marker::Marker;
use std::fmt::Debug;

pub trait Datum: Clone + Debug {
    fn len(&self) -> usize;

    fn as_slice(&self) -> &[u8];
}

pub struct Data<Datum> {
    data: Vec<Datum>,
}

impl<D> Data<D>
where
    D: Datum,
{
    pub fn with_capacity(size: usize) -> Data<D> {
        Data {
            data: Vec::with_capacity(size),
        }
    }

    pub fn clear(&mut self) {
        self.data.clear()
    }

    pub fn amount(&self) -> usize {
        self.data.len()
    }

    pub fn push(&mut self, datum: D) {
        self.data.push(datum)
    }

    pub fn marker_len(&self, marker: &Marker) -> usize {
        if marker.pos1.0 == marker.pos2.0 {
            marker.pos2.1 - marker.pos1.1
        } else {
            let mut tot_len = 0;

            tot_len += self.data[marker.pos1.0].len() - marker.pos1.1;
            tot_len += marker.pos2.1;

            for i in marker.pos1.0 + 1..marker.pos2.0 {
                tot_len += self.data[i].len();
            }

            tot_len
        }
    }

    pub fn resize(&self, marker: Marker, newlen: usize) -> Marker {
        if marker.pos1.0 == marker.pos2.0 {
            Marker::new(
                (marker.pos1.0, marker.pos1.1),
                (marker.pos2.0, marker.pos1.1 + newlen),
            )
        } else {
            let ref datum = self.data[marker.pos1.0];

            if datum.len() - marker.pos1.1 >= newlen {
                Marker::new(
                    (marker.pos1.0, marker.pos1.1),
                    (marker.pos1.0, marker.pos1.1 + newlen),
                )
            } else {
                let pos1 = marker.pos1;
                let mut pos2 = marker.pos1;
                let mut tot_len = newlen - (datum.len() - marker.pos1.1);

                loop {
                    pos2.0 += 1;
                    let ref datum = self.data[pos2.0];
                    if datum.len() >= tot_len {
                        pos2.1 = tot_len;
                        break;
                    } else {
                        tot_len -= datum.len();
                    }
                }

                Marker::new(pos1, pos2)
            }
        }
    }

    pub fn chunk<'a, 'b>(&'a self, marker: &'b Marker) -> Chunk<'a> {
        if marker.pos1.0 == marker.pos2.0 {
            self._chunk_slice(marker.pos1.0, marker.pos1.1, marker.pos2.1)
        } else {
            self._chunk_vec(marker)
        }
    }

    fn _chunk_slice<'a>(&'a self, datum_idx: usize, start: usize, end: usize) -> Chunk<'a> {
        Chunk::Slice(&self.data[datum_idx].as_slice()[start..end])
    }

    fn _chunk_vec<'a, 'b>(&'a self, marker: &'b Marker) -> Chunk<'a> {
        let mut vec: Vec<u8>;

        let mut len = 0;

        len += self.data[marker.pos1.0].len() - marker.pos1.1;
        len += marker.pos2.1;

        if marker.pos2.0 - marker.pos1.0 > 1 {
            for i in marker.pos1.0 + 1..marker.pos2.0 {
                len += self.data[i].len();
            }
        }

        vec = Vec::with_capacity(len);

        vec.extend(&self.data[marker.pos1.0].as_slice()[marker.pos1.1..]);

        if marker.pos2.0 - marker.pos1.0 > 1 {
            for i in marker.pos1.0 + 1..marker.pos2.0 {
                vec.extend(self.data[i].as_slice());
            }
        }

        vec.extend(&self.data[marker.pos2.0].as_slice()[..marker.pos2.1]);

        Chunk::Vec(vec)
    }
}

pub enum Chunk<'a> {
    Slice(&'a [u8]),
    Vec(Vec<u8>),
}

impl<'a> Chunk<'a> {
    pub fn as_slice(&self) -> &[u8] {
        self.into()
    }
}

impl<'a, 'b> Into<&'b [u8]> for &'b Chunk<'a> {
    fn into(self) -> &'b [u8] {
        match *self {
            Chunk::Slice(slice) => slice,
            Chunk::Vec(ref v) => v.as_slice(),
        }
    }
}

impl<'a> From<Vec<u8>> for Chunk<'a> {
    fn from(v: Vec<u8>) -> Chunk<'a> {
        Chunk::Vec(v)
    }
}
