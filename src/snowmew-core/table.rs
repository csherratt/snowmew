
use std::collections::VecMap;
use std::collections::vec_map::Iter;
use std::sync::Arc;
use std::default::Default;


use cow::btree::{BTreeMap, BTreeMapIterator};
use cow::btree::{BTreeSet, BTreeSetIterator};
use collect::iter::{OrderedMapIterator, OrderedSetIterator};


use Entity;


/// a Static table should be used for infrequently updated data
#[derive(RustcEncodable, RustcDecodable)]
pub struct Static<T: Send+Sync+Clone+Default>(BTreeMap<Entity, T>);

impl<T: Send+Clone+Sync+Default> Clone for Static<T> {
    fn clone(&self) -> Static<T> {
        Static(match self { &Static(ref t) => t.clone() })
    }
}

impl<T: Send+Clone+Sync+Default> Static<T> {
    pub fn new() -> Static<T> {
        Static(BTreeMap::new())
    }

    pub fn insert(&mut self, key: Entity, value: T) -> bool {
        match self { &mut Static(ref mut t) => t.insert(key, value) }
    }

    pub fn get(&self, key: Entity) -> Option<&T> {
        match self { &Static(ref t) => t.get(&key) }
    }

    pub fn get_mut(&mut self, key: Entity) -> Option<&mut T> {
        match self { &mut Static(ref mut t) => t.get_mut(&key) }
    }

    pub fn remove(&mut self, key: Entity) -> bool {
        match self { &mut Static(ref mut t) => t.remove(&key) }
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
    iter: BTreeMapIterator<'a, Entity, T>
}

impl<'a, T: Send+Sync> Iterator for StaticIterator<'a, T> {
    type Item = (Entity, &'a T);

    fn next(&mut self) -> Option<(Entity, &'a T)> {
        match self.iter.next() {
            None => None,
            Some((&key, value)) => Some((key, value))
        }
    }
}

#[derive(Clone, RustcEncodable, RustcDecodable, Default)]
pub struct StaticSet(BTreeSet<Entity>);

impl StaticSet {
    pub fn new() -> StaticSet {
        StaticSet(BTreeSet::new())
    }

    pub fn insert(&mut self, key: Entity) -> bool {
        match self { &mut StaticSet(ref mut t) => t.insert(key) }
    }

    pub fn remove(&mut self, key: Entity) -> bool {
        match self { &mut StaticSet(ref mut t) => t.remove(&key) }
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
    iter: BTreeSetIterator<'a, Entity>
}

impl<'a> Iterator for StaticSetIterator<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Entity> {
        match self.iter.next() {
            None => None,
            Some(&key) => Some(key)
        }
    }
}

#[derive(Default, RustcEncodable, RustcDecodable)]
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

    pub fn get(&self, key: Entity) -> Option<&T> {
        match self { &Dynamic(ref t) => t.get(&(key as uint)) }
    }

    pub fn get_mut(&mut self, key: Entity) -> Option<&mut T> {
        match self { &mut Dynamic(ref mut t) => t.make_unique().get_mut(&(key as uint)) }
    }

    pub fn insert(&mut self, key: Entity, value: T) -> bool {
        match self { &mut Dynamic(ref mut t) => t.make_unique().insert(key as uint, value) }.is_some()
    }

    pub fn remove(&mut self, key: Entity) -> bool {
        match self { &mut Dynamic(ref mut t) => t.make_unique().remove(&(key as uint)) }.is_some()
    }

    pub fn iter(&self) -> DynamicIterator<T> {
        DynamicIterator {
            iter: match self { &Dynamic(ref t) => t.iter() }
        }
    }

    pub fn len(&self) -> uint {
        match self { &Dynamic(ref t) => t.len() }
    }

    pub fn highest_entity(&self) -> Entity {
        match self {
            &Dynamic(ref t) => {
                t.iter()
                 .next_back()
                 .map(|(k, _)| k as u32)
                 .unwrap_or(0u32)
            }
        }
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

