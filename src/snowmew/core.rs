
use std::default::Default;

use cow::btree::{BTreeMap, BTreeSet, BTreeSetIterator};

use cgmath::transform::*;
use cgmath::quaternion::*;
use cgmath::vector::*;

use default::load_default;
use position;
use position::Positions;

use graphics;
use graphics::{Graphics};


#[deriving(Clone, Default)]
pub struct FrameInfo {
    count: uint,  /* unique frame identifier */
    time: f64,    /* current time in seconds */
    delta: f64,   /* time from last frame */
}


#[deriving(Clone, Default)]
pub struct Object
{
    pub parent: ObjectKey,
    pub name: ObjectKey,
}

pub type ObjectKey = u32;
pub type StringKey = u32;

pub struct Location {
    trans: Transform3D<f32>
}

impl Default for Location
{
    fn default() -> Location
    {
        Location {
            trans: Transform3D::new(1f32, Quat::zero(), Vec3::zero())
        }
    }
}

impl Clone for Location
{
    fn clone(&self) -> Location
    {
        let tras = self.trans.get();
        Location {
            trans: Transform3D::new(tras.scale.clone(),
                                    tras.rot.clone(),
                                    tras.disp.clone())
        }
    }
}

#[deriving(Clone)]
pub struct Database {
    common:   CommonData,
    position: position::PositionData,
    graphics: graphics::GraphicsData
}

#[deriving(Clone)]
pub struct CommonData {
    last_sid:      StringKey,
    strings:       BTreeMap<StringKey, ~str>,
    string_to_key: BTreeMap<~str, StringKey>,

    last_oid:      ObjectKey,
    objects:       BTreeMap<ObjectKey, Object>,
    index_parent_child: BTreeMap<ObjectKey, BTreeSet<ObjectKey>>,
}

impl CommonData {
    pub fn new() -> CommonData {
        CommonData {
            last_sid:           1,
            strings:            BTreeMap::new(),
            string_to_key:      BTreeMap::new(),

            last_oid:           1,
            objects:            BTreeMap::new(),
            index_parent_child: BTreeMap::new(),
        }   
    }

    fn ifind(&self, node: Option<ObjectKey>, str_key: &str) -> Option<ObjectKey> {
        let node = match node {
            Some(key) => key,
            None => 0
        };

        let child = match self.index_parent_child.find(&node) {
            Some(children) => children,
            None => return None,
        };

        for key in child.iter() {
            match self.objects.find(key) {
                Some(obj) => {
                    if self.strings.find(&obj.name)
                        .expect("Could not find str key").as_slice() == str_key {
                        return Some(*key);
                    }
                },
                _ => ()
            }
        }

        None
    }

    fn name(&self, key: ObjectKey) -> ~str {
        match self.objects.find(&key) {
            Some(node) => {
                format!("{:s}/{:s}", self.name(node.parent), *self.strings.find(&node.name).unwrap())
            },
            None => ~"base"
        }
    }


    fn new_key(&mut self) -> ObjectKey {
        let new_key = self.last_oid;
        self.last_oid += 1;
        new_key        
    }

    fn update_parent_child(&mut self, parent: ObjectKey, child: ObjectKey) {
        let new = match self.index_parent_child.find_mut(&parent) {
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
            Some(child_list) => {self.index_parent_child.insert(parent, child_list);},
            None => (),
        }
    }

    fn new_string(&mut self, s: &str) -> StringKey {
        let (update, name) = match self.string_to_key.find(&s.to_owned()) {
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
            self.strings.insert(name, s.to_owned());
            self.string_to_key.insert(s.to_owned(), name);
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

    fn new_object(&mut self, parent: Option<ObjectKey>, name: &str) -> ObjectKey {
        let new_key = self.get_common_mut().new_key();
        let parent = match parent {
            Some(key) => key,
            None => 0
        };

        let object = Object {
            name: self.get_common_mut().new_string(name),
            parent: parent
        };

        self.get_common_mut().objects.insert(new_key, object);
        self.get_common_mut().update_parent_child(parent, new_key);

        new_key
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

    fn walk_dir<'a>(&'a self, oid: ObjectKey) -> BTreeSetIterator<'a, ObjectKey> {
        let dir = self.get_common().index_parent_child.find(&oid).unwrap();
        dir.iter()
    }

    fn name(&self, key: ObjectKey) -> ~str {
        self.get_common().name(key)
    }
}

impl Common for Database {
    fn get_common<'a>(&'a self) -> &'a CommonData {
        &self.common
    }

    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData {
        &mut self.common
    }
}

impl Database {
    pub fn new() -> Database {
        let mut new = Database::empty();
        load_default(&mut new);
        new
        
    }

    pub fn empty() -> Database {
        Database {
            common:   CommonData::new(),
            position: position::PositionData::new(),
            graphics: graphics::GraphicsData::new(),
        }
    }
}

impl Positions for Database {
    fn get_position<'a>(&'a self) -> &'a position::PositionData {
        &self.position
    }

    fn get_position_mut<'a>(&'a mut self) -> &'a mut position::PositionData {
        &mut self.position
    }
}

impl Graphics for Database {
    fn get_graphics<'a>(&'a self) -> &'a graphics::GraphicsData {
        &self.graphics
    }

    fn get_graphics_mut<'a>(&'a mut self) -> &'a mut graphics::GraphicsData {
        &mut self.graphics
    }
}

/*struct IterObjsLayer<'a>
{
    child_iter: JoinMapSetIterator<
                    BTreeSetIterator<'a, ObjectKey>,
                    BTreeMapIterator<'a, ObjectKey, position::Id>>
}

pub struct IterObjs<'a>
{
    db: &'a Common,
    stack: ~[IterObjsLayer<'a>]
}

impl<'a> Iterator<(ObjectKey, uint)> for IterObjs<'a>
{
    #[inline(always)]
    fn next(&mut self) -> Option<(ObjectKey, uint)>
    {
        loop {
            let len = self.stack.len();
            if len == 0 {
                return None;
            }

            match self.stack[len-1].child_iter.next() {
                Some((oid, loc)) => {
                    match self.db.get().index_parent_child.find(oid) {
                        Some(set) => {
                            self.stack.push(IterObjsLayer {
                                child_iter: join_set_to_map(set.iter(), self.db.location.iter())
                            });
                        },
                        None => ()
                    }

                    return Some((*oid, self.db.position.deref().get_loc(*loc)))
                },
                None => { self.stack.pop(); }
            }
        }
    }
}*/
