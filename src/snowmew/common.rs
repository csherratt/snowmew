use cow::btree::{BTreeMap, BTreeMapIterator};

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
    last_sid:      StringKey,
    strings:       BTreeMap<StringKey, ~str>,
    string_to_key: BTreeMap<~str, StringKey>,

    last_oid:      ObjectKey,
    objects:       BTreeMap<ObjectKey, Object>,
    parent_child:  BTreeMap<ObjectKey, BTreeMap<StringKey, ObjectKey>>,
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

        let str_key = match self.string_to_key.find(&str_key.to_owned()) {
            Some(key) => key,
            None => return None
        };

        match child.find(str_key) {
            Some(a) => Some(*a),
            None => None
        }
    }

    fn name(&self, key: ObjectKey) -> ~str {
        match self.objects.find(&key) {
            Some(node) => {
                format!("{:s}/{:s}", self.name(node.parent), *self.strings.find(&node.name).unwrap())
            },
            None => "base".to_owned()
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
        self.get_common_mut().update_parent_child(parent, object.name, new_key);

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

    fn walk_dir<'a>(&'a self, oid: ObjectKey) -> DirIter<'a> {
        let dir = self.get_common().parent_child.find(&oid).unwrap();
        DirIter {
            common: self.get_common(),
            iter: dir.iter()
        }
    }


    fn name(&self, key: ObjectKey) -> ~str {
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