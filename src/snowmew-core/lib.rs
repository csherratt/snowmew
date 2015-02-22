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

#![crate_name = "snowmew-core"]
#![crate_type = "lib"]
#![feature(alloc, collections)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate collections;
extern crate collect;
extern crate cow;

pub use common::Entity;

/// contains the common data for the Entity manager
pub mod common;
/// contains a few different formats that can be used
/// to represent a table in snowmew
pub mod table;
/// contains the `Game` trait
pub mod game;

pub trait Get<K> {
    type Value;

    fn get(&self, key: K) -> Option<Self::Value>;
}

pub trait GetRef<K> {
    type Value;

    fn get_ref(&self, key: K) -> Option<&Self::Value>;
}

pub trait GetRefMut<K> {
    type Value;

    fn get_mut(&mut self, key: K) -> Option<&mut Self::Value>;
}

pub trait Set<K, V> {
    fn set(&mut self, key: K, Value: V);
}

pub trait ToEntity {
    fn to_entity(self) -> Entity;
}

impl ToEntity for Entity {
    fn to_entity(self) -> Entity { self }
}