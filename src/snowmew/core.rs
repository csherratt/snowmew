use geometry::Geometry;
use shader::Shader;
use coregl::{Uniforms, Texture};
use render::Context;
use cow::btree::BTree;

use cgmath::vector::*;

pub trait DrawSize {
    fn size(&self) -> (uint, uint);
}

pub trait DrawTarget: DrawSize {
    fn draw(&self, ctx: &mut Context, &Shader, &Geometry, &[(i32, &Uniforms)], &[&Texture]);
}

pub trait FrameBuffer: DrawSize {
    fn viewport(&self, ctx: &mut Context, offset :(uint, uint), size :(uint, uint), f: |&mut DrawTarget, ctx: &mut Context|);
}

pub struct FrameInfo {
    count: uint,  /* unique frame identifier */
    time: f64,    /* current time in seconds */
    delta: f64,   /* time from last frame */
}

#[deriving(Clone, Default)]
pub struct Location
{
    position: Vec3<f32>,
    rotation: Vec3<f32>,
    scale: f32,
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
    shader: shader_key,
    geometry: geometry_key,
    textures: ~[texture_key],
}

pub type object_key = i32;
pub type shader_key = i32;
pub type geometry_key = i32;
pub type texture_key = i32;

pub struct Database {
    priv last_key: i32,

    // raw data
    priv objects: BTree<object_key, Object>,
    priv location: BTree<object_key, Location>,
    priv draw: BTree<object_key, Drawable>,
    
    priv geometry: BTree<geometry_key, Geometry>,
    priv shaders: BTree<shader_key, Shader>,

    // --- indexes ---
    // map all children to a parent
    priv index_parent_child: BTree<i32, BTree<i32, ()>>,
}

impl Database {
    pub fn new() -> Database
    {
        Database {
            last_key: 1,
            objects: BTree::new(),
            location:  BTree::new(),
            draw: BTree::new(),
            
            geometry:  BTree::new(),
            shaders: BTree::new(),

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

    pub fn update_location(&mut self, key: object_key, location: Location)
    {
        self.location.insert(key, location);
    }

    fn ifind(&self, node: Option<object_key>, str_key: &str) -> Option<object_key>
    {
        let node = match node {
            Some(key) => key,
            None => 0
        };

        let children = match self.index_parent_child.find(&node) {
            Some(children) => children,
            None => return None,
        };

        for (key, _) in children.iter() {
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
}
