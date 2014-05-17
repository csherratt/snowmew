use std::default;

#[deriving(Clone)]
pub struct Texture {
    width: uint,
    height: uint,
    depth: uint,
    data: Vec<u8>
}

impl default::Default for Texture {
    fn default() -> Texture {
        Texture::new(0, 0, 0, Vec::new())
    }
}

impl Texture {
    pub fn new(width: uint, height: uint, depth: uint, data: Vec<u8>) -> Texture {
        Texture {
            width: width,
            height: height,
            depth: depth,
            data: data
        }
    }

    pub fn width(&self) -> uint { self.width }
    pub fn height(&self) -> uint { self.height }
    pub fn depth(&self) -> uint { self.depth }
    pub fn data<'a>(&'a self) -> &'a [u8] { self.data.as_slice() }
}