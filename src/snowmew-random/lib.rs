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

#![feature(associated_types)]
extern crate "snowmew-core" as core;

use std::rand::{Rng, ChaChaRng};

#[derive(Copy)]
pub struct RandomData {
    nonce: u64,
    frame: u32,
    rng: ChaChaRng
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
        let frame = self.frame;
        self.rng.set_counter((frame as u64) << 32, nonce);
    }
}

pub trait Random {
    fn get_random_mut(&mut self) -> &mut RandomData;

    fn rng(&mut self) -> &mut ChaChaRng {
        &mut self.get_random_mut().rng
    }

    fn set_nonce(&mut self, nonce: u64) {
        self.get_random_mut().nonce = nonce;
        self.get_random_mut().reseed();

    }

    fn set_frame(&mut self, frame: u32) {
        self.get_random_mut().frame = frame;
        self.get_random_mut().reseed();
    }
}
