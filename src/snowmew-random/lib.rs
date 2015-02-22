//   Copyright 2014-2015 Colin Sherratt
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

extern crate "snowmew-core" as core;
extern crate rand;

use rand::{Rng, ChaChaRng};

#[derive(Copy)]
pub struct RandomData {
    nonce: u64,
    frame: u32,
    rng: ChaChaRng
}

impl Clone for RandomData {
    fn clone(&self) -> RandomData {
        RandomData {
            nonce: self.nonce,
            frame: self.frame,
            rng: self.rng
        }
    }
}

impl RandomData {
    pub fn new() -> RandomData {
        RandomData {
            nonce: 0,
            frame: 0,
            rng: ChaChaRng::new_unseeded()
        }
    }

    fn reseed(&mut self) {
        let nonce = self.nonce;
        let frame = self.frame as u64;
        self.rng.set_counter(frame << 32, nonce);
    }
}

pub trait Random {
    fn rng(&mut self) -> &mut RandomData;

    fn set_nonce(&mut self, nonce: u64) {
        self.rng().nonce = nonce;
        self.rng().reseed();

    }

    fn set_frame(&mut self, frame: u32) {
        self.rng().frame = frame;
        self.rng().reseed();
    }
}

impl Rng for RandomData {
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }
}
