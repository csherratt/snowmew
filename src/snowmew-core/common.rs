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

use serialize::Encodable;
use std::sync::Arc;
use io::IoState;
use input;
use table::{Static, StaticSet, StaticSetIterator};

/// A common set of data owned by an `Entity`
#[deriving(Clone, Default, Encodable, Decodable)]
pub struct Object {
    /// Who is the parent of this object
    pub parent: Entity,
}

/// A key to connect Entities across Systems
pub type Entity = u32;

/// CommonData is a container that contains all the information needed
/// to implement the Common root of `snowmews`'s entity systems
#[deriving(Clone, Encodable, Decodable)]
pub struct CommonData {
    last_oid:       Entity,
    objects:        Static<Object>,
    parent_child:   Static<StaticSet>,
    scene_children: Static<StaticSet>,
    freelist: Arc<Vec<Entity>>,
    io: IoState
}

impl CommonData {
    /// Create CommonData for use with the `Common` trait
    pub fn new() -> CommonData {
        CommonData {
            last_oid: 1,
            objects: Static::new(),
            parent_child: Static::new(),
            scene_children: Static::new(),
            freelist: Arc::new(Vec::new()),
            io: IoState::new()
        }
    }

    fn new_key(&mut self) -> Entity {
        if !self.freelist.is_empty() {
            return self.freelist.make_unique().pop().expect("missing entry...");
        }
        let new_key = self.last_oid;
        self.last_oid += 1;
        new_key
    }

    fn update_parent_child(&mut self, parent: Entity, child: Entity) {
        let new = match self.parent_child.get_mut(parent) {
            Some(child_list) => {
                child_list.insert(child);
                None
            },
            None => {
                let mut child_list = StaticSet::new();
                child_list.insert(child);
                Some(child_list)
            }
        };

        match new {
            Some(child_list) => {self.parent_child.insert(parent, child_list);},
            None => (),
        }
    }
}

/// Common is a trait that your `GameData` needs to implement as the
/// root system for the entity manager.
pub trait Common {
    /// get a non-mutable pointer to the `CommonData` of `GameData`
    fn get_common<'a>(&'a self) -> &'a CommonData;
    /// get a mutable pointer to the `CommonData` from the `GameData`
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData;

    /// Create a new scene.
    fn new_scene(&mut self) -> Entity {
        let oid = self.new_object(None);
        self.get_common_mut().scene_children.insert(oid, StaticSet::new());
        oid
    }

    /// Create a new object, if a parent is supplied the object
    /// is owned by the parent. This can be used to create parent-child
    /// bonding between objects
    fn new_object(&mut self, parent: Option<Entity>) -> Entity {
        let new_key = self.get_common_mut().new_key();
        let mut parent = match parent {
            Some(key) => key,
            None => 0
        };

        let object = Object {
            parent: parent
        };

        self.get_common_mut().objects.insert(new_key, object);
        self.get_common_mut().update_parent_child(parent, new_key);

        let mut scene_id = None;
        while parent != 0 {
            match self.get_common().scene_children.get(parent) {
                None => {
                    parent = self.get_common().objects.get(parent).unwrap().parent;
                }
                Some(_) => {
                    scene_id = Some(parent);
                    parent = 0;
                }
            }
        }

        match scene_id {
            Some(id) => {
                let sc = self.get_common_mut().scene_children.get_mut(id).unwrap();
                sc.insert(new_key);
            }
            None => ()
        }

        new_key
    }

    /// Create an Iterator that iterators over the scene supplied.
    fn scene_iter<'a>(&'a self, oid: Entity) -> StaticSetIterator<'a> {
        let sc = self.get_common().scene_children.get(oid)
            .expect("Failed to get scene");
        sc.iter()
    }

    /// Get the object metadata for Entity
    fn object<'a>(&'a self, oid: Entity) -> Option<&'a Object> {
        self.get_common().objects.get(oid)
    }

    /// Apply an `WindowEvent` to the system, this will update
    /// the io metadata (io_state)
    fn window_action(&mut self, evt: input::WindowEvent) {
        self.get_common_mut().io.window_action(evt);
    }

    /// Read the io metadata
    fn io_state(&self) -> &IoState { &self.get_common().io }
}

/// Duplicate all components owned by `src` into `dst`
pub trait Duplicate {
    fn duplicate(&mut self, src: Entity, dst: Entity);
}

/// Delete all components owned by the `Entity`
pub trait Delete {
    fn delete(&mut self, oid: Entity) -> bool;
}

impl Delete for CommonData {
    fn delete(&mut self, oid: Entity) -> bool {
        let o = self.objects.get(oid).map(|x| *x);
        match o {
            Some(o) => {
                self.freelist.make_unique().push(oid);
                self.objects.remove(oid)                      |
                self.parent_child.remove(oid)                 |
                self.scene_children.remove(oid)               |
                (self.parent_child.get_mut(o.parent)
                    .map(|x| { x.remove(oid) }) == Some(true)) |
                (self.scene_children.get_mut(o.parent)
                    .map(|x| { x.remove(oid) }) == Some(true))
            }
            None => false
        }
    }
}

impl Common for CommonData {
    fn get_common<'a>(&'a self) -> &'a CommonData {self}
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData {self}
}
