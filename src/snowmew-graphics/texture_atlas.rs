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

use snowmew::common::Entity;
use snowmew::table::{Static, StaticIterator};

use Texture;

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Atlas {
    width: uint,
    height: uint,
    depth: uint,
    max_layers: uint,
    layers: Static<uint>,
    free_layers: Vec<uint>
}

impl Atlas {
    pub fn new(width: uint, height: uint, depth: uint) -> Atlas {
        let layers = 100_000_000 / (width * height * depth);

        let mut free_layers = Vec::new();
        for l in range(0, layers) {
            free_layers.push(l);
        }

        Atlas {
            width: width,
            height: height,
            depth: depth,
            max_layers: layers,
            layers: Static::new(),
            free_layers: free_layers
        }
    }

    pub fn check_texture(&self, text: &Texture) -> bool {
        self.width == text.width()   &&
        self.height == text.height() &&
        self.depth == text.depth()   &&
        (!self.free_layers.is_empty())
    }

    pub fn add_texture(&mut self, id: Entity, text: &Texture) -> uint {
        assert!(self.check_texture(text));

        let layer = self.free_layers.pop().expect("Failed to get free layer");
        self.layers.insert(id, layer);
        layer
    }

    pub fn texture_iter<'a>(&'a self) -> StaticIterator<'a, uint> {
        self.layers.iter()
    }

    pub fn max_layers(&self) -> uint {self.max_layers}
}