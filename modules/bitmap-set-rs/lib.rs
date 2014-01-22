#[crate_id = "github.com/csherratt/bitmap-set#bitmap-set:0.1"];
#[license = "ASL2"];
#[crate_type = "lib"];
#[comment = "A bitmap-set in rust"];

use std::vec;

pub struct BitMapSet {
    cells: ~[u64]
}

impl BitMapSet {
    pub fn new(size: uint) -> BitMapSet
    {
        let bsize = size / 64 + if size % 64 != 0 {1} else {0};

        let mut out: ~[u64] = vec::with_capacity(bsize);

        for _ in range(0, bsize) {
            out.push(0);
        }

        BitMapSet {
            cells: out
        }
    }

    pub fn set(&mut self, idx: uint)
    {
        let bit = idx & 0x3F;
        let word = idx >> 6;
        self.cells[word] |= 1 << bit;
    }

    pub fn check(&self, idx: uint) -> bool
    {
        let bit = idx & 0x3F;
        let word = idx >> 6;
        (self.cells[word] & (1 << bit)) != 0
    }
}