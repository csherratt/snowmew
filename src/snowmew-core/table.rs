
use std::collections::VecMap;
use std::collections::vec_map::Iter;
use std::sync::Arc;

use rustc_serialize::{Encodable, Decodable, Encoder, Decoder};

use collect::{cow_trie_map, cow_trie_set};
use collect::{CowTrieMap, CowTrieSet};
use collect::iter::{OrderedMapIterator, OrderedSetIterator};


use Entity;


/// a Static table should be used for infrequently updated data
pub struct Static<T: Send+Sync+Clone>(Arc<CowTrieMap<T>>);

impl<T: Send+Clone+Sync> Clone for Static<T> {
    fn clone(&self) -> Static<T> {
        Static(match self { &Static(ref t) => t.clone() })
    }
}

impl<T: Send+Clone+Sync> Static<T> {
    pub fn new() -> Static<T> {
        Static(Arc::new(CowTrieMap::new()))
    }

    pub fn insert(&mut self, key: Entity, value: T) -> bool {
        match self { &mut Static(ref mut t) => t.make_unique().insert(key as usize, value).is_some() }
    }

    pub fn get(&self, key: Entity) -> Option<&T> {
        let key = key as usize;
        match self { &Static(ref t) => t.get(&key) }
    }

    pub fn get_mut(&mut self, key: Entity) -> Option<&mut T> {
        let key = key as usize;
        match self { &mut Static(ref mut t) => t.make_unique().get_mut(&key) }
    }

    pub fn remove(&mut self, key: Entity) -> bool {
        let key = key as usize;
        match self { &mut Static(ref mut t) => t.make_unique().remove(&key).is_some() }
    }

    pub fn iter(&self) -> StaticIterator<T> {
        StaticIterator {
            iter: match self { &Static(ref t) => t.iter() }
        }
    }

    pub fn len(&self) -> usize {
        match self { &Static(ref t) => t.len() }
    }
}

pub struct StaticIterator<'a, T:'a> {
    iter: cow_trie_map::Iter<'a, T>
}

impl<'a, T: Send+Sync> Iterator for StaticIterator<'a, T> {
    type Item = (Entity, &'a T);

    fn next(&mut self) -> Option<(Entity, &'a T)> {
        match self.iter.next() {
            None => None,
            Some((key, value)) => Some((key as Entity, value))
        }
    }
}

impl<T:Send+Sync+Clone+Encodable> Encodable for Static<T> {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        e.emit_map(self.len(), |e| {
            let mut i = 0;
            for (key, val) in self.iter() {
                try!(e.emit_map_elt_key(i, |e| key.encode(e)));
                try!(e.emit_map_elt_val(i, |e| val.encode(e)));
                i += 1;
            }
            Ok(())
        })
    }
}

impl<T:Send+Sync+Clone+Decodable> Decodable for Static<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Static<T>, D::Error> {
        d.read_map(|d, len| {
            let mut map = Static::new();
            for i in (0us..len) {
                let key = try!(d.read_map_elt_key(i, |d| Decodable::decode(d)));
                let val = try!(d.read_map_elt_val(i, |d| Decodable::decode(d)));
                map.insert(key, val);
            }
            Ok(map)
        })
    }
}


#[derive(Clone, Default)]
pub struct StaticSet(CowTrieSet);

impl StaticSet {
    pub fn new() -> StaticSet {
        StaticSet(CowTrieSet::new())
    }

    pub fn insert(&mut self, key: Entity) -> bool {
        match self { &mut StaticSet(ref mut t) => t.insert(key as usize) }
    }

    pub fn contains(&self, key: Entity) -> bool {
        match self { &StaticSet(ref t) => t.contains(&(key as usize)) }
    }

    pub fn remove(&mut self, key: Entity) -> bool {
        let key = key as usize;
        match self { &mut StaticSet(ref mut t) => t.remove(&key) }
    }

    pub fn iter(&self) -> StaticSetIterator {
        StaticSetIterator {
            iter: match self { &StaticSet(ref t) => t.iter() }
        }
    }

    pub fn len(&self) -> usize {
        match self { &StaticSet(ref t) => t.len() }
    }
}

pub struct StaticSetIterator<'a> {
    iter: cow_trie_set::Iter<'a>
}

impl<'a> Iterator for StaticSetIterator<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Entity> {
        match self.iter.next() {
            None => None,
            Some(key) => Some(key as Entity)
        }
    }
}

impl Encodable for StaticSet {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(self.len(), |s| {
            let mut i = 0;
            for e in self.iter() {
                try!(s.emit_seq_elt(i, |s| e.encode(s)));
                i += 1;
            }
            Ok(())
        })
    }
}

impl Decodable for StaticSet {
    fn decode<D: Decoder>(d: &mut D) -> Result<StaticSet, D::Error> {
        d.read_seq(|d, len| {
            let mut set = StaticSet::new();
            for i in (0us..len) {
                set.insert(try!(d.read_seq_elt(i, |d| Decodable::decode(d))));
            }
            Ok(set)
        })
    }
}

#[derive(Default, RustcEncodable, RustcDecodable)]
pub struct Dynamic<T: Send+Sync+Clone>(Arc<VecMap<T>>);

impl<T: Send+Clone+Sync> Clone for Dynamic<T> {
    fn clone(&self) -> Dynamic<T> {
        Dynamic(match self { &Dynamic(ref t) => t.clone() })
    }
}

impl<T: Send+Sync+Clone> Dynamic<T> {
    pub fn new() -> Dynamic<T> {
        Dynamic(Arc::new(VecMap::new()))
    }

    pub fn get(&self, key: Entity) -> Option<&T> {
        match self { &Dynamic(ref t) => t.get(&(key as usize)) }
    }

    pub fn get_mut(&mut self, key: Entity) -> Option<&mut T> {
        match self { &mut Dynamic(ref mut t) => t.make_unique().get_mut(&(key as usize)) }
    }

    pub fn insert(&mut self, key: Entity, value: T) -> bool {
        match self { &mut Dynamic(ref mut t) => t.make_unique().insert(key as usize, value) }.is_some()
    }

    pub fn remove(&mut self, key: Entity) -> bool {
        match self { &mut Dynamic(ref mut t) => t.make_unique().remove(&(key as usize)) }.is_some()
    }

    pub fn iter(&self) -> DynamicIterator<T> {
        DynamicIterator {
            iter: match self { &Dynamic(ref t) => t.iter() }
        }
    }

    pub fn len(&self) -> usize {
        match self { &Dynamic(ref t) => t.len() }
    }
}

pub struct DynamicIterator<'a, T:'a> {
    iter: Iter<'a, T>
}

impl<'a, T: Send+Sync> Iterator for DynamicIterator<'a, T> {
    type Item = (Entity, &'a T);

    fn next(&mut self) -> Option<(Entity, &'a T)> {
        match self.iter.next() {
            None => None,
            Some((key, value)) => Some((key as Entity, value))
        }
    }
}

impl<'a, T: Send+Sync> OrderedMapIterator<Entity, &'a T> for StaticIterator<'a, T> {}
impl<'a> OrderedSetIterator<Entity> for StaticSetIterator<'a> {}
impl<'a, T: Send+Sync> OrderedMapIterator<Entity, &'a T> for DynamicIterator<'a, T> {}

