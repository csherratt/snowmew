use std::default;

#[deriving(Clone)]
pub struct Texture {
    width: uint,
    height: uint,
    depth: uint,
    data: Vec<u8>
}

fn offset(width: uint, depth: uint,
          row: uint, column: uint, component: uint) -> uint {
    width * depth * row +
    depth * column +
    component
}

fn flip(dat: &mut Vec<u8>, height: uint, width: uint, depth: uint) {
    for row in range(0, height/2) {
        let swap_row = height - row - 1;
        for column in range(0, width) {
            for d in range(0, depth) {
                let a_addr = offset(width, depth, row, column, d);
                let b_addr = offset(width, depth, swap_row, column, d);
                let temp = *dat.get(a_addr);
                *dat.get_mut(a_addr) = *dat.get(b_addr);
                *dat.get_mut(b_addr) = temp;
            }
        }
    }
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
    pub fn flip(&mut self) {
        flip(&mut self.data, self.height, self.width, self.depth);
    }
}