
use cow::btree::{BTreeMap, BTreeSet, BTreeSetIterator, BTreeMapIterator};
use cow::join::{join_set_to_map, JoinMapSetIterator};

use octtree;
use octtree::sparse::Sparse;
use octtree::{Cube};

use geometry::{Geometry, VertexBuffer};
use shader::Shader;

use cgmath::transform::*;
use cgmath::matrix::*;
use cgmath::quaternion::*;
use cgmath::vector::*;

use bitmap::BitMapSet;

use default::load_default;

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

#[deriving(Clone, Default, Eq)]
pub struct Drawable
{
    shader: object_key,
    geometry: object_key,
    textures: ~[object_key],
}

pub type object_key = uint;

pub struct Location {
    priv trans: Transform3D<f32>
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
    priv last_key: object_key,

    // raw data
    priv objects: BTreeMap<object_key, Object>,
    priv location: BTreeMap<object_key, Location>,
    priv draw: BTreeMap<object_key, Drawable>,
    
    priv geometry: BTreeMap<object_key, Geometry>,
    priv vertex: BTreeMap<object_key, VertexBuffer>,
    priv shader: BTreeMap<object_key, Shader>,

    // --- indexes ---
    // map all children to a parent
    priv index_parent_child: BTreeMap<object_key, BTreeSet<object_key>>,
    priv position: octtree::sparse::Sparse<f32, Cube<f32>, object_key>
}

impl Database {
    pub fn new() -> Database
    {
        let mut new = Database::empty();
        load_default(&mut new);
        new
        
    }

    pub fn empty() -> Database
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
            position: octtree::sparse::Sparse::new(1000f32, 6)
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

    pub fn object<'a>(&'a self, oid: object_key) -> Option<&'a Object>
    {
        self.objects.find(&oid)
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

    pub fn add_dir(&mut self, parent: Option<object_key>, name: ~str) -> object_key
    {
        self.new_object(parent, name)
    }

    pub fn position(&self, oid: object_key) -> Mat4<f32>
    {
        let obj = self.objects.find(&oid);
        let p_mat = match obj {
            Some(obj) => self.position(obj.parent),
            None => Mat4::identity()
        };

        let loc = match self.location(oid) {
            Some(t) => {t.get().to_mat4()},
            None => Mat4::identity()
        };
        p_mat.mul_m(&loc)
    }

    fn set_position(&mut self, oid: object_key)
    {
        let sphere = Cube::from_mat4(&self.position(oid));
        self.position.insert(sphere, oid);
    }

    fn update_position(&mut self, oid: object_key, old: &Mat4<f32>)
    {
        let old = Cube::from_mat4(old);
        self.position.remove(old);
        self.set_position(oid);
    }

    pub fn update_location(&mut self, key: object_key, location: Transform3D<f32>)
    {
        let old = self.position(key);
        if self.location.insert(key, Location{trans: location}) {
            self.update_position(key, &old);
        } else {
            self.set_position(key);
        }
    }

    pub fn location<'a>(&'a self, key: object_key) -> Option<&'a Transform3D<f32>>
    {
        match self.location.find(&key) {
            Some(loc) => Some(&loc.trans),
            None => None
        }
    }

    pub fn update_drawable(&mut self, key: object_key, draw: Drawable)
    {
        self.draw.insert(key, draw);
    }

    pub fn drawable<'a>(&'a self, key: object_key) -> Option<&'a Drawable>
    {
        self.draw.find(&key)
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

    pub fn geometry<'a>(&'a self, oid: object_key) -> Option<&'a Geometry>
    {
        self.geometry.find(&oid)
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

    pub fn walk_drawables<'a>(&'a self) -> UnwrapKey<BTreeMapIterator<'a, object_key, Drawable>>
    {
        UnwrapKey::new(self.draw.iter())
    } 

    pub fn walk_in_camera<'a>(&'a self, oid: object_key, camera: &Mat4<f32>) -> IterObjs<'a>
    {
        let mut set = BitMapSet::new(1024*1024);
        self.position.quary(camera, |_, val| {set.set(*val);});

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
            view_set: set,
            stack: stack
        }
    }

    pub fn walk_vertex_buffers<'a>(&'a self) -> BTreeMapIterator<'a, object_key, VertexBuffer>
    {
        self.vertex.iter()
    }

    pub fn walk_shaders<'a>(&'a self) -> BTreeMapIterator<'a, object_key, Shader>
    {
        self.shader.iter()
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
    priv view_set: BitMapSet,
    priv stack: ~[IterObjsLayer<'a>]
}

impl<'a> Iterator<(object_key, Mat4<f32>)> for IterObjs<'a>
{
    #[inline(always)]
    fn next(&mut self) -> Option<(object_key, Mat4<f32>)>
    {
        loop {
            let len = self.stack.len();
            if len == 0 {
                return None;
            }

            match self.stack[len-1].child_iter.next() {
                Some((object_key, loc)) => {
                    if self.view_set.check(*object_key) {
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

                        return Some((*object_key, mat))
                    }
                },
                None => { self.stack.pop(); }
            }
        }
    }
}

struct UnwrapKey<IN>
{
    input: IN
}

impl<IN> UnwrapKey<IN> {
    fn new(input: IN) -> UnwrapKey<IN>
    {
        UnwrapKey {
            input: input
        }
    }
}

impl<'a, K: Clone, V, IN: Iterator<(&'a K, &'a V)>> Iterator<(K, &'a V)> for UnwrapKey<IN>
{
    #[inline(always)]
    fn next(&mut self) -> Option<(K, &'a V)>
    {
        match self.input.next() {
            Some((k, v)) => Some((k.clone(), v)),
            None => None
        }
    }
}