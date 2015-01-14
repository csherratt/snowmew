//   Copyright 2014 Colin Sherratt
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

use std::default;

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Texture {
    width: u32,
    height: u32,
    depth: u32,
    data: Vec<u8>
}

fn offset(width: usize, depth: usize,
          row: usize, column: usize, component: usize) -> usize {
    width * depth * row +
    depth * column +
    component
}

fn flip(dat: &mut Vec<u8>, height: usize, width: usize, depth: usize) {
    for row in range(0, height/2) {
        let swap_row = height - row - 1;
        for column in range(0, width) {
            for d in range(0, depth) {
                let a_addr = offset(width, depth, row, column, d);
                let b_addr = offset(width, depth, swap_row, column, d);
                let temp = (*dat)[a_addr];
                dat[a_addr] = (*dat)[b_addr];
                dat[b_addr] = temp;
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
    pub fn new(width: u32, height: u32, depth: u32, data: Vec<u8>) -> Texture {
        Texture {
            width: width,
            height: height,
            depth: depth,
            data: data
        }
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn depth(&self) -> u32 { self.depth }
    pub fn data<'a>(&'a self) -> &'a [u8] { &self.data[] }
    pub fn flip(&mut self) {
        flip(&mut self.data, self.height as usize, self.width as usize, self.depth as usize);
    }
}