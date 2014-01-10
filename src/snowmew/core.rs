use cow::btree::BTree;

use geometry::{Geometry, VertexBuffer};
use shader::Shader;

use cgmath::vector::*;
use cgmath::transform::*;

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

#[deriving(Clone)]
pub struct Database {
    priv last_key: i32,

    // raw data
    priv objects: BTree<object_key, Object>,
    priv location: BTree<object_key, Transform3D<f32>>,
    priv draw: BTree<object_key, Drawable>,
    
    priv geometry: BTree<object_key, Geometry>,
    priv vertex: BTree<object_key, VertexBuffer>,
    priv shader: BTree<object_key, Shader>,

    // --- indexes ---
    // map all children to a parent
    priv index_parent_child: BTree<i32, BTree<object_key, ()>>,
}

impl Database {
    pub fn new() -> Database
    {
        Database {
            last_key: 1,
            objects: BTree::new(),
            location: BTree::new(),
            draw: BTree::new(),
            
            geometry: BTree::new(),
            vertex: BTree::new(),
            shader: BTree::new(),

            // --- indexes ---
            // map all children to a parent
            index_parent_child: BTree::new(),
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
                child_list.insert(child, ());
                None
            },
            None => {
                let mut child_list = BTree::new();
                child_list.insert(child, ());
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
        self.location.insert(key, location);
    }

    pub fn location<'a>(&'a self, key: object_key) -> Option<&'a Transform3D<f32>>
    {
        self.location.find(&key)
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

        for (key, _) in child.iter() {
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


        for (key, _) in child.iter() {
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
}
