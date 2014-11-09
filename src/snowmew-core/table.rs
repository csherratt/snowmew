
use std::collections::VecMap;
use std::collections::vec_map::Entries;
use std::sync::Arc;

use ObjectKey;
use cow::btree::{BTreeMap, BTreeMapIterator};
use cow::btree::{BTreeSet, BTreeSetIterator};
use std::default::Default;
use serialize::Encodable;


/// a Static table should be used for infrequently updated data
#[deriving(Encodable, Decodable)]
pub struct Static<T: Send+Sync+Clone+Default>(BTreeMap<ObjectKey, T>);

impl<T: Send+Clone+Sync+Default> Clone for Static<T> {
    fn clone(&self) -> Static<T> {
        Static(match self { &Static(ref t) => t.clone() })
    }
}

impl<T: Send+Clone+Sync+Default> Static<T> {
    pub fn new() -> Static<T> {
        Static(BTreeMap::new())
    }

    pub fn insert(&mut self, key: ObjectKey, value: T) -> bool {
        match self { &Static(ref mut t) => t.insert(key, value) }
    }

    pub fn get(&self, key: ObjectKey) -> Option<&T> {
        match self { &Static(ref t) => t.find(&key) }
    }

    pub fn get_mut(&mut self, key: ObjectKey) -> Option<&mut T> {
        match self { &Static(ref mut t) => t.find_mut(&key) }
    }

    pub fn remove(&mut self, key: ObjectKey) -> bool {
        match self { &Static(ref mut t) => t.remove(&key) }
    }

    pub fn iter(&self) -> StaticIterator<T> {
        StaticIterator {
            iter: match self { &Static(ref t) => t.iter() }
        }
    }

    pub fn len(&self) -> uint {
        match self { &Static(ref t) => t.len() }
    }
}

pub struct StaticIterator<'a, T:'a> {
    iter: BTreeMapIterator<'a, ObjectKey, T>
}

impl<'a, T: Send+Sync> Iterator<(ObjectKey, &'a T)> for StaticIterator<'a, T> {
    fn next(&mut self) -> Option<(ObjectKey, &'a T)> {
        match self.iter.next() {
            None => None,
            Some((&key, value)) => Some((key, value))
        }
    }
}

#[deriving(Clone, Encodable, Decodable, Default)]
pub struct StaticSet(BTreeSet<ObjectKey>);

impl StaticSet {
    pub fn new() -> StaticSet {
        StaticSet(BTreeSet::new())
    }

    pub fn insert(&mut self, key: ObjectKey) -> bool {
        match self { &StaticSet(ref mut t) => t.insert(key) }
    }

    pub fn remove(&mut self, key: ObjectKey) -> bool {
        match self { &StaticSet(ref mut t) => t.remove(&key) }
    }

    pub fn iter(&self) -> StaticSetIterator {
        StaticSetIterator {
            iter: match self { &StaticSet(ref t) => t.iter() }
        }
    }

    pub fn len(&self) -> uint {
        match self { &StaticSet(ref t) => t.len() }
    }
}

pub struct StaticSetIterator<'a> {
    iter: BTreeSetIterator<'a, ObjectKey>
}

impl<'a> Iterator<ObjectKey> for StaticSetIterator<'a> {
    fn next(&mut self) -> Option<ObjectKey> {
        match self.iter.next() {
            None => None,
            Some(&key) => Some(key)
        }
    }
}

#[deriving(Default)]
pub struct Dynamic<T: Send+Sync+Clone>(Arc<VecMap<T>>);

impl<T: Send+Clone+Sync+Default> Clone for Dynamic<T> {
    fn clone(&self) -> Dynamic<T> {
        Dynamic(match self { &Dynamic(ref t) => t.clone() })
    }
}

impl<T: Send+Sync+Clone> Dynamic<T> {
    pub fn new() -> Dynamic<T> {
        Dynamic(Arc::new(VecMap::new()))
    }

    pub fn get(&self, key: ObjectKey) -> Option<&T> {
        match self { &Dynamic(ref t) => t.get(&(key as uint)) }
    }

    pub fn get_mut(&mut self, key: ObjectKey) -> Option<&mut T> {
        match self { &Dynamic(ref mut t) => t.make_unique().get_mut(&(key as uint)) }
    }

    pub fn insert(&mut self, key: ObjectKey, value: T) -> bool {
        match self { &Dynamic(ref mut t) => t.make_unique().insert(key as uint, value) }.is_some()
    }

    pub fn remove(&mut self, key: ObjectKey) -> bool {
        match self { &Dynamic(ref mut t) => t.make_unique().remove(&(key as uint)) }.is_some()
    }

    pub fn iter(&self) -> DynamicIterator<T> {
        DynamicIterator {
            iter: match self { &Dynamic(ref t) => t.iter() }
        }
    }

    pub fn len(&self) -> uint {
        match self { &Dynamic(ref t) => t.len() }
    }
}

pub struct DynamicIterator<'a, T:'a> {
    iter: Entries<'a, T>
}

impl<'a, T: Send+Sync> Iterator<(ObjectKey, &'a T)> for DynamicIterator<'a, T> {
    fn next(&mut self) -> Option<(ObjectKey, &'a T)> {
        match self.iter.next() {
            None => None,
            Some((key, value)) => Some((key as ObjectKey, value))
        }
    }
}