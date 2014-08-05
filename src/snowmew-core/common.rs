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

use cow::btree::{BTreeMap, BTreeMapIterator, BTreeSet, BTreeSetIterator};

#[deriving(Clone, Default)]
pub struct FrameInfo {
    count: uint,  /* unique frame identifier */
    time: f64,    /* current time in seconds */
    delta: f64,   /* time from last frame */
}


#[deriving(Clone, Default)]
pub struct Object {
    pub parent: ObjectKey,
    pub name: ObjectKey,
}

pub type ObjectKey = u32;
pub type StringKey = u32;

#[deriving(Clone)]
pub struct CommonData {
    last_sid:       StringKey,
    strings:        BTreeMap<StringKey, String>,
    string_to_key:  BTreeMap<String, StringKey>,

    last_oid:       ObjectKey,
    objects:        BTreeMap<ObjectKey, Object>,
    parent_child:   BTreeMap<ObjectKey, BTreeMap<StringKey, ObjectKey>>,

    scene_children: BTreeMap<ObjectKey, BTreeSet<ObjectKey>>
}

impl CommonData {
    pub fn new() -> CommonData {
        CommonData {
            last_sid:           1,
            strings:            BTreeMap::new(),
            string_to_key:      BTreeMap::new(),

            last_oid:           1,
            objects:            BTreeMap::new(),
            parent_child:       BTreeMap::new(),

            scene_children:     BTreeMap::new()
        }   
    }

    fn ifind(&self, node: Option<ObjectKey>, str_key: &str) -> Option<ObjectKey> {
        let node = match node {
            Some(key) => key,
            None => 0
        };

        let child = match self.parent_child.find(&node) {
            Some(children) => children,
            None => return None,
        };

        let str_key = match self.string_to_key.find(&str_key.to_string()) {
            Some(key) => key,
            None => return None
        };

        match child.find(str_key) {
            Some(a) => Some(*a),
            None => None
        }
    }

    fn name(&self, key: ObjectKey) -> String {
        match self.objects.find(&key) {
            Some(node) => {
                format!("{:s}/{:s}", self.name(node.parent), *self.strings.find(&node.name).unwrap())
            },
            None => "base".to_string()
        }
    }


    fn new_key(&mut self) -> ObjectKey {
        let new_key = self.last_oid;
        self.last_oid += 1;
        new_key        
    }

    fn update_parent_child(&mut self, parent: ObjectKey, child_name: StringKey, child: ObjectKey) {
        let new = match self.parent_child.find_mut(&parent) {
            Some(child_list) => {
                child_list.insert(child_name, child);
                None
            },
            None => {
                let mut child_list = BTreeMap::new();
                child_list.insert(child_name, child);
                Some(child_list)
            }
        };

        match new {
            Some(child_list) => {self.parent_child.insert(parent, child_list);},
            None => (),
        }
    }

    fn new_string(&mut self, s: &str) -> StringKey {
        let (update, name) = match self.string_to_key.find(&s.to_string()) {
            None => {
                (true, 0)
            }
            Some(key) => {
                (false, key.clone())
            }
        };

        if update {
            let name = self.last_sid;
            self.last_sid += 1;
            self.strings.insert(name, s.to_string());
            self.string_to_key.insert(s.to_string(), name);
            name
        } else {
            name
        }
    }
}

pub trait Common {
    fn get_common<'a>(&'a self) -> &'a CommonData;
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData;

    fn add_dir(&mut self, parent: Option<ObjectKey>, name: &str) -> ObjectKey {
        self.new_object(parent, name)
    }

    fn new_scene(&mut self, name: &str) -> ObjectKey {
        let oid = self.new_object(None, name);
        self.get_common_mut().scene_children.insert(oid, BTreeSet::new());
        oid
    }

    fn new_object(&mut self, parent: Option<ObjectKey>, name: &str) -> ObjectKey {
        let new_key = self.get_common_mut().new_key();
        let mut parent = match parent {
            Some(key) => key,
            None => 0
        };

        let object = Object {
            name: self.get_common_mut().new_string(name),
            parent: parent
        };

        self.get_common_mut().objects.insert(new_key, object);
        self.get_common_mut().update_parent_child(parent, object.name, new_key);

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

    fn find(&self, str_key: &str) -> Option<ObjectKey> {
        let mut node = None;
        for s in str_key.split('/') {
            let next = self.get_common().ifind(node, s);
            if next == None {
                return None
            }
            node = next;
        }
        node
    }

    fn walk_dir<'a>(&'a self, oid: ObjectKey) -> DirIter<'a> {
        let dir = self.get_common().parent_child.find(&oid).unwrap();
        DirIter {
            common: self.get_common(),
            iter: dir.iter()
        }
    }

    fn name(&self, key: ObjectKey) -> String {
        self.get_common().name(key)
    }
}

impl Common for CommonData {
    fn get_common<'a>(&'a self) -> &'a CommonData {self}
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData {self}
}

pub struct DirIter<'a> {
    common: &'a CommonData,
    iter: BTreeMapIterator<'a, StringKey, ObjectKey>,
}

impl<'a> Iterator<(&'a str, ObjectKey)> for DirIter<'a> {
    fn next(&mut self) -> Option<(&'a str, ObjectKey)> {
        match self.iter.next() {
            Some((sid, oid)) => {
                Some((self.common.strings
                        .find(sid).expect("Found StringKey w/o Key").as_slice(),
                    *oid)
                )
            }
            None => None
        }
    }
}