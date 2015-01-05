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

extern crate snowmew;

use snowmew::random::{Random, RandomData};
use std::rand::Rng;

struct Foo {
    random: RandomData
}

impl Random for Foo {
    fn get_random_mut(&mut self) -> &mut RandomData {
        &mut self.random
    }
}

#[test]
fn check_seed() {
    let mut foo = Foo { random: RandomData::new() };

    foo.set_nonce(1234);

    foo.set_frame(10);
    let f10: Vec<u32> = range(0, 8).map(|_| foo.rng().next_u32()).collect();

    foo.set_frame(11);
    let f11: Vec<u32> = range(0, 8).map(|_| foo.rng().next_u32()).collect();

    foo.set_frame(10);
    let f10_2: Vec<u32> = range(0, 8).map(|_| foo.rng().next_u32()).collect();

    assert_eq!(f10, f10_2);
    assert!(f10 != f11);
}