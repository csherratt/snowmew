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

use image::{self, GenericImage};
use graphics::Texture;

pub fn load_texture(path: &Path) -> Texture {
    let img = image::open(path).ok().expect("Failed to load image.")
                               .to_rgba();
    let (w, h) = img.dimensions();
    let data = img.into_vec();
    let mut out = Texture::new(w, h, 4, data);
    out
}