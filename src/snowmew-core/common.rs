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

use cow::btree::{BTreeMap, BTreeSet, BTreeSetIterator};
use serialize::Encodable;
use io::IoState;
use input;

#[deriving(Clone, Default)]
pub struct FrameInfo {
    count: uint,  /* unique frame identifier */
    time: f64,    /* current time in seconds */
    delta: f64,   /* time from last frame */
}


#[deriving(Clone, Default, Encodable, Decodable)]
pub struct Object {
    pub parent: ObjectKey,
}

pub type ObjectKey = u32;
pub type StringKey = u32;

#[deriving(Clone, Encodable, Decodable)]
pub struct CommonData {
    last_oid:       ObjectKey,
    objects:        BTreeMap<ObjectKey, Object>,
    parent_child:   BTreeMap<ObjectKey, BTreeSet<ObjectKey>>,
    scene_children: BTreeMap<ObjectKey, BTreeSet<ObjectKey>>,
    io: IoState
}

impl CommonData {
    pub fn new() -> CommonData {
        CommonData {
            last_oid: 1,
            objects: BTreeMap::new(),
            parent_child: BTreeMap::new(),
            scene_children: BTreeMap::new(),
            io: IoState::new()
        }
    }

    fn new_key(&mut self) -> ObjectKey {
        let new_key = self.last_oid;
        self.last_oid += 1;
        new_key
    }

    fn update_parent_child(&mut self, parent: ObjectKey, child: ObjectKey) {
        let new = match self.parent_child.find_mut(&parent) {
            Some(child_list) => {
                child_list.insert(child);
                None
            },
            None => {
                let mut child_list = BTreeSet::new();
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


pub trait Common {
    fn get_common<'a>(&'a self) -> &'a CommonData;
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData;

    fn add_dir(&mut self, parent: Option<ObjectKey>) -> ObjectKey {
        self.new_object(parent)
    }

    fn new_scene(&mut self) -> ObjectKey {
        let oid = self.new_object(None);
        self.get_common_mut().scene_children.insert(oid, BTreeSet::new());
        oid
    }

    fn new_object(&mut self, parent: Option<ObjectKey>) -> ObjectKey {
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
            match self.get_common().scene_children.find(&parent) {
                None => {
                    parent = self.get_common().objects.find(&parent).unwrap().parent;
                }
                Some(_) => {
                    scene_id = Some(parent);
                    parent = 0;
                }
            }
        }

        match scene_id {
            Some(id) => {
                let sc = self.get_common_mut().scene_children.find_mut(&id).unwrap();
                sc.insert(new_key);
            }
            None => ()
        }

        new_key
    }

    fn scene_iter<'a>(&'a self, oid: ObjectKey) -> BTreeSetIterator<'a, u32> {
        let sc = self.get_common().scene_children.find(&oid)
            .expect("Failed to find scene");
        sc.iter()
    }

    fn object<'a>(&'a self, oid: ObjectKey) -> Option<&'a Object> {
        self.get_common().objects.find(&oid)
    }

    fn walk_dir<'a>(&'a self, oid: ObjectKey) -> DirIter<'a> {
        let dir = self.get_common().parent_child.find(&oid).unwrap();
        dir.iter()
    }

    fn name(&self, key: ObjectKey) -> String {
        self.get_common().name(key)
    }

    fn window_action(&mut self, evt: input::WindowEvent) {
        self.get_common_mut().io.window_action(evt);
    }

    fn io_state(&self) -> &IoState { &self.get_common().io }
}

pub trait Duplicate {
    fn duplicate(&mut self, src: ObjectKey, dst: ObjectKey);
}

pub trait Delete {
    fn delete(&mut self, oid: ObjectKey) -> bool;
}

impl Delete for CommonData {
    fn delete(&mut self, oid: ObjectKey) -> bool {
        let o = self.objects.find(&oid).map(|x| *x);
        match o {
            Some(o) => {
                self.objects.remove(&oid)                      |
                self.parent_child.remove(&oid)                 |
                self.scene_children.remove(&oid)               |
                (self.parent_child.find_mut(&o.parent)
                    .map(|x| { x.remove(&oid) }) == Some(true)) |
                (self.scene_children.find_mut(&o.parent)
                    .map(|x| { x.remove(&oid) }) == Some(true))
            }
            None => false
        }
    }
}

impl Common for CommonData {
    fn get_common<'a>(&'a self) -> &'a CommonData {self}
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData {self}
}

pub type DirIter<'a> = BTreeSetIterator<'a, ObjectKey>;