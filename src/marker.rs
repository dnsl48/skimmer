#[derive (Clone, Debug)]
pub struct Marker {
    pub pos1: (usize, usize),
    pub pos2: (usize, usize)
}



impl Marker {
    pub fn new (pos1: (usize, usize), pos2: (usize, usize)) -> Marker {
        Marker { pos1: pos1, pos2: pos2 }
    }
}
