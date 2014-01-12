use cow::btree::{BTreeMap, BTreeSet, BTreeSetIterator, BTreeMapIterator};
use cow::join::{join_set_to_map, JoinMapSetIterator};

use geometry::{Geometry, VertexBuffer};
use shader::Shader;

use cgmath::vector::*;
use cgmath::transform::*;
use cgmath::matrix::*;

#[deriving(Clone, Default)]
pub struct FrameInfo {
    count: uint,  /* unique frame identifier */
    time: f64,    /* current time in seconds */
    delta: f64,   /* time from last frame */
}

#[deriving(Clone, Default)]
pub struct Object
{
    parent: object_key,
    name: ~str,
}

#[deriving(Clone, Default)]
pub struct Drawable
{
    shader: object_key,
    geometry: object_key,
    textures: ~[object_key],
}

pub type object_key = i32;

#[deriving(Clone, Default)]
pub struct Location {
    priv trans: Transform3D<f32>
}

//#[deriving(Clone)]
pub struct Database {
    priv last_key: i32,

    // raw data
    priv objects: BTreeMap<object_key, Object>,
    priv location: BTreeMap<object_key, Location>,
    priv draw: BTreeMap<object_key, Drawable>,
    
    priv geometry: BTreeMap<object_key, Geometry>,
    priv vertex: BTreeMap<object_key, VertexBuffer>,
    priv shader: BTreeMap<object_key, Shader>,

    // --- indexes ---
    // map all children to a parent
    priv index_parent_child: BTreeMap<i32, BTreeSet<object_key>>,
}

impl Database {
    pub fn new() -> Database
    {
        Database {
            last_key: 1,
            objects: BTreeMap::new(),
            location: BTreeMap::new(),
            draw: BTreeMap::new(),
            
            geometry: BTreeMap::new(),
            vertex: BTreeMap::new(),
            shader: BTreeMap::new(),

            // --- indexes ---
            // map all children to a parent
            index_parent_child: BTreeMap::new(),
        }
    }

    fn new_key(&mut self) -> object_key
    {
        let new_key = self.last_key;
        self.last_key += 1;
        new_key        
    }

    fn update_parent_child(&mut self, parent: object_key, child: object_key)
    {
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

    pub fn new_object(&mut self, parent: Option<object_key>, name: ~str) -> object_key
    {
        let new_key = self.new_key();
        let parent = match parent {
            Some(key) => key,
            None => 0
        };

        let object = Object {
            name: name,
            parent: parent
        };

        self.objects.insert(new_key, object);
        self.update_parent_child(parent, new_key);

        new_key
    }

    pub fn update_location(&mut self, key: object_key, location: Transform3D<f32>)
    {
        self.location.insert(key, Location{trans: location});
    }

    pub fn location<'a>(&'a self, key: object_key) -> Option<&'a Transform3D<f32>>
    {
        match self.location.find(&key) {
            Some(loc) => Some(&loc.trans),
            None => None
        }
    }

    fn ifind(&self, node: Option<object_key>, str_key: &str) -> Option<object_key>
    {
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
                    if obj.name.as_slice() == str_key {
                        return Some(*key);
                    }
                },
                _ => ()
            }
        }

        None
    }

    pub fn name(&self, key: object_key) -> ~str
    {
        match self.objects.find(&key) {
            Some(node) => {
                format!("{}/{}", self.name(node.parent), node.name)
            },
            None => ~""
        }
    }

    pub fn find(&self, str_key: &str) -> Option<object_key>
    {
        let mut node = None;
        for s in str_key.split('/') {
            let next = self.ifind(node, s);
            if next == None {
                return None
            }
            node = next;
        }
        node
    }

    fn idump(&self, depth: int, node: object_key)
    {
        let child = match self.index_parent_child.find(&node) {
            Some(children) => children,
            None => return,
        };


        for key in child.iter() {
            println!("{:5}: {:s}", *key, self.name(*key));
            self.idump(depth+1, *key);
        }
    }

    pub fn dump(&self) {self.idump(0, 0);}

    pub fn add_vertex_buffer(&mut self, parent: object_key, name: ~str, vb: VertexBuffer) -> object_key
    {
        let oid = self.new_object(Some(parent), name);
        self.vertex.insert(oid, vb);
        oid
    }

    pub fn add_geometry(&mut self, parent: object_key, name: ~str, geo: Geometry) -> object_key
    {
        let oid = self.new_object(Some(parent), name);
        self.geometry.insert(oid, geo);
        oid
    }

    pub fn add_shader(&mut self, parent: object_key, name: ~str, shader: Shader) -> object_key
    {
        let oid = self.new_object(Some(parent), name);
        self.shader.insert(oid, shader);
        oid
    }

    pub fn set_draw(&mut self, oid: object_key, geo: object_key, shader: object_key)
    {
        self.draw.insert(oid,
            Drawable {
                geometry: geo,
                shader: shader,
                textures: ~[]
            }
        );
    }

    pub fn walk_drawables<'a>(&'a self, oid: object_key) -> IterObjs<'a>
    {
        let mat = match self.location.find(&oid) {
            Some(loc) => loc.trans.get().to_mat4(),
            None => Mat4::identity()
        };

        let stack = match self.index_parent_child.find(&oid) {
            Some(set) => {
                ~[IterObjsLayer {
                    child_iter: join_set_to_map(
                                    set.iter(),
                                    self.location.iter()),
                    mat: mat,
                }]
            },
            None => ~[]
        };

        IterObjs {
            db: self,
            stack: stack
        }
    }
}

struct IterObjsLayer<'a>
{
    child_iter: JoinMapSetIterator<
                    BTreeSetIterator<'a, object_key>,
                    BTreeMapIterator<'a, object_key, Location>>,
    mat: Mat4<f32>
}

pub struct IterObjs<'a>
{
    priv db: &'a Database,
    priv stack: ~[IterObjsLayer<'a>]
}

impl<'a> Iterator<(object_key, Mat4<f32>, &'a Drawable)> for IterObjs<'a>
{
    fn next(&mut self) -> Option<(object_key, Mat4<f32>, &'a Drawable)>
    {
        loop {
            let len = self.stack.len();
            if len == 0 {
                return None;
            }

            match self.stack[len-1].child_iter.next() {
                Some((object_key, loc)) => {

                    let mat = self.stack[len-1].mat.mul_m(&loc.trans.get().to_mat4());

                    match self.db.index_parent_child.find(object_key) {
                        Some(set) => {
                            self.stack.push(IterObjsLayer {
                                mat: mat,
                                child_iter: join_set_to_map(set.iter(), self.db.location.iter())
                            });
                        },
                        None => ()
                    }

                    match self.db.draw.find(object_key) {
                        Some(draw) => return Some((*object_key, mat, draw)),
                        None => ()
                    }
                },
                None => { self.stack.pop(); }
            }
        }
    }
}