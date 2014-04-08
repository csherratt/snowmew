
use std::default::Default;

use sync::Arc;

use cow::btree::{BTreeMap, BTreeSet, BTreeSetIterator, BTreeMapIterator};
use cow::join::{join_set_to_map, join_maps, JoinMapSetIterator, JoinMapIterator};

use geometry::{Geometry, VertexBuffer};
use material::Material;
use timing::Timing;

use cgmath::transform::*;
use cgmath::matrix::*;
use cgmath::quaternion::*;
use cgmath::vector::*;

use default::load_default;
use position;

#[deriving(Clone, Default)]
pub struct FrameInfo {
    count: uint,  /* unique frame identifier */
    time: f64,    /* current time in seconds */
    delta: f64,   /* time from last frame */
}

#[deriving(Clone)]
pub enum ObjectType {
    Invalid,
    Directory,
    Draw,
    VertexBuffer
}

impl Default for ObjectType
{
    fn default() -> ObjectType
    {
        Invalid
    }
}

#[deriving(Clone)]
pub struct Light
{
    pub color: Vec3<f32>,
    pub intensity: f32
}

impl Default for Light
{
    fn default() -> Light
    {
        Light {
            color: Vec3::new(0f32, 0f32, 0f32),
            intensity: 0.
        }
    }
}

#[deriving(Clone, Default)]
pub struct Object
{
    pub obj_type: ObjectType,
    pub parent: ObjectKey,
    pub name: ObjectKey,
}

#[deriving(Clone, Default, Eq)]
pub struct Drawable
{
    pub geometry: ObjectKey,
    pub material: ObjectKey
}

impl Ord for Drawable
{
    fn lt(&self, other: &Drawable) -> bool
    {
        let order = self.geometry.cmp(&other.geometry);
        match order {
            Equal => self.material.cmp(&other.material) == Less,
            Greater => false,
            Less => true
        }        
    }
}

impl TotalEq for Drawable {}

impl TotalOrd for Drawable {
    fn cmp(&self, other: &Drawable) -> Ordering
    {
        let order = self.geometry.cmp(&other.geometry);
        match order {
            Equal => self.material.cmp(&other.material),
            Greater => Greater,
            Less => Less
        }
    }
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
    last_sid:      StringKey,
    strings:       BTreeMap<StringKey, ~str>,
    string_to_key: BTreeMap<~str, StringKey>,

    // raw data
    last_oid:      ObjectKey,
    objects:       BTreeMap<ObjectKey, Object>,
    location:      BTreeMap<ObjectKey, position::Id>,
    draw:          BTreeMap<ObjectKey, Drawable>,
    geometry:      BTreeMap<ObjectKey, Geometry>,
    vertex:        BTreeMap<ObjectKey, VertexBuffer>,
    material:      BTreeMap<ObjectKey, Material>,
    light:         BTreeMap<ObjectKey, Light>,
    pub position:  Arc<position::Deltas>,

    // --- indexes ---
    // map all children to a parent
    index_parent_child: BTreeMap<ObjectKey, BTreeSet<ObjectKey>>,
    pub draw_bins: BTreeMap<ObjectKey, BTreeSet<position::Id>>,

    // other
    timing: Timing
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
            last_sid:           1,
            strings:            BTreeMap::new(),
            string_to_key:      BTreeMap::new(),

            last_oid:           1,
            objects:            BTreeMap::new(),
            location:           BTreeMap::new(),
            draw:               BTreeMap::new(),
            geometry:           BTreeMap::new(),
            vertex:             BTreeMap::new(),
            material:           BTreeMap::new(),
            light:              BTreeMap::new(),
            position:           Arc::new(position::Deltas::new()),

            // --- indexes ---
            // map all children to a parent
            index_parent_child: BTreeMap::new(),
            draw_bins: BTreeMap::new(),

            timing: Timing::new()
        }
    }

    fn new_string(&mut self, s: &str) -> StringKey
    {
        let (update, name) = match self.string_to_key.find(&s.to_owned()) {
            None => {
                let id = self.last_sid;
                self.last_sid += 1;

                self.strings.insert(id, s.to_owned());

                (true, id)
            }
            Some(key) => {
                (false, key.clone())
            }
        };

        if update {
            self.string_to_key.insert(s.to_owned(), name);
        }
        name
    }

    fn new_key(&mut self) -> ObjectKey
    {
        let new_key = self.last_oid;
        self.last_oid += 1;
        new_key        
    }

    fn update_parent_child(&mut self, parent: ObjectKey, child: ObjectKey)
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

    pub fn object<'a>(&'a self, oid: ObjectKey) -> Option<&'a Object>
    {
        self.objects.find(&oid)
    }

    pub fn new_object(&mut self, parent: Option<ObjectKey>, name: &str) -> ObjectKey
    {
        let new_key = self.new_key();
        let parent = match parent {
            Some(key) => key,
            None => 0
        };

        let object = Object {
            obj_type: Invalid,
            name: self.new_string(name),
            parent: parent
        };

        self.objects.insert(new_key, object);
        self.update_parent_child(parent, new_key);

        new_key
    }

    pub fn add_dir(&mut self, parent: Option<ObjectKey>, name: &str) -> ObjectKey
    {
        self.new_object(parent, name)
    }

    pub fn position(&self, oid: ObjectKey) -> Mat4<f32>
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

    fn get_position_id(&mut self, key: ObjectKey) -> position::Id
    {
        if key == 0 {
            position::Deltas::root()
        } else {
            match self.location.find(&key) {
                Some(id) =>  return *id,
                None => ()
            }

            let poid = self.objects.find(&key).unwrap().parent;
            let pid = self.get_position_id(poid);
            let id = self.position.make_unique().insert(pid, Transform3D::new(1f32, Quat::identity(), Vec3::new(0f32, 0f32, 0f32)));
            self.location.insert(key, id);
            id
        }
    }

    pub fn update_location(&mut self, key: ObjectKey, location: Transform3D<f32>)
    {
        let id = self.get_position_id(key);
        self.position.make_unique().update(id, location);
    }

    pub fn location(&self, key: ObjectKey) -> Option<Transform3D<f32>>
    {
        match self.location.find(&key) {
            Some(id) => Some(self.position.deref().get_delta(*id)),
            None => None
        }
    }

    pub fn update_drawable(&mut self, key: ObjectKey, draw: Drawable)
    {
        self.draw.insert(key, draw.clone());

    }

    pub fn drawable<'a>(&'a self, key: ObjectKey) -> Option<&'a Drawable>
    {
        self.draw.find(&key)
    }

    fn ifind(&self, node: Option<ObjectKey>, str_key: &str) -> Option<ObjectKey>
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
                    if self.strings.find(&obj.name).unwrap().as_slice() == str_key {
                        return Some(*key);
                    }
                },
                _ => ()
            }
        }

        None
    }

    pub fn last_name(&self, key: ObjectKey) -> ~str
    {
        match self.objects.find(&key) {
            Some(node) => {
                self.strings.find(&node.name).unwrap().to_owned()
            },
            None => ~"base"
        }
    }

    pub fn name(&self, key: ObjectKey) -> ~str
    {
        match self.objects.find(&key) {
            Some(node) => {
                format!("{:s}/{:s}", self.name(node.parent), *self.strings.find(&node.name).unwrap())
            },
            None => ~"base"
        }
    }

    pub fn find(&self, str_key: &str) -> Option<ObjectKey>
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

    fn idump(&self, depth: int, node: ObjectKey)
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

    pub fn dump(&self)
    {
        self.idump(0, 0);
    }

    pub fn new_vertex_buffer(&mut self, parent: ObjectKey, name: &str, vb: VertexBuffer) -> ObjectKey
    {
        let oid = self.new_object(Some(parent), name);
        self.vertex.insert(oid, vb);
        oid
    }

    pub fn new_geometry(&mut self, parent: ObjectKey, name: &str, geo: Geometry) -> ObjectKey
    {
        let oid = self.new_object(Some(parent), name);
        self.geometry.insert(oid, geo);
        oid
    }

    pub fn geometry<'a>(&'a self, oid: ObjectKey) -> Option<&'a Geometry>
    {
        self.geometry.find(&oid)
    }

    pub fn material<'a>(&'a self, oid: ObjectKey) -> Option<&'a Material>
    {
        self.material.find(&oid)
    }

    pub fn new_material(&mut self, parent: ObjectKey, name: &str, material: Material) -> ObjectKey
    {
        let obj = self.new_object(Some(parent), name);
        self.material.insert(obj, material);
        obj
    }

    pub fn material_iter<'a>(&'a self) -> BTreeMapIterator<'a, ObjectKey, Material>
    {
        self.material.iter()
    }

    pub fn light<'a>(&'a self, oid: ObjectKey) -> Option<&'a Light>
    {
        self.light.find(&oid)
    }

    pub fn new_light(&mut self, parent: ObjectKey, name: &str, light: Light) -> ObjectKey
    {
        let obj = self.new_object(Some(parent), name);
        self.light.insert(obj, light);
        obj
    }

    pub fn set_draw(&mut self, oid: ObjectKey, geo: ObjectKey, material: ObjectKey)
    {
        let draw = Drawable {
            geometry: geo,
            material: material
        };

        self.draw.insert(oid, draw.clone());

        let pid = self.get_position_id(oid);
        let create = match self.draw_bins.find_mut(&draw.geometry) {
            Some(draw_bin) => {
                draw_bin.insert(pid.clone());
                false
            },
            None => true
        };

        if create {
            let mut set = BTreeSet::new();
            set.insert(pid.clone());
            self.draw_bins.insert(draw.geometry, set);
        }
    }

    pub fn walk_dir<'a>(&'a self, oid: ObjectKey) -> BTreeSetIterator<'a, ObjectKey>
    {
        let dir = self.index_parent_child.find(&oid).unwrap();

        dir.iter()
    }

    pub fn drawable_count(&self) -> uint
    {
        self.draw.len()
    }

    pub fn walk_drawables<'a>(&'a self) -> UnwrapKey<BTreeMapIterator<'a, ObjectKey, Drawable>>
    {
        UnwrapKey::new(self.draw.iter())
    }

    pub fn walk_drawables_and_pos<'a>(&'a self) -> 
        JoinMapIterator<BTreeMapIterator<'a, ObjectKey, Drawable>, BTreeMapIterator<'a, ObjectKey, position::Id>>
    {
        join_maps(self.draw.iter(), self.location.iter())
    }

    pub fn walk_scene<'a>(&'a self, oid: ObjectKey) -> IterObjs<'a>
    {
        let stack = match self.index_parent_child.find(&oid) {
            Some(set) => {
                ~[IterObjsLayer {
                    child_iter: join_set_to_map(
                                    set.iter(),
                                    self.location.iter())
                }]
            },
            None => ~[]
        };

        IterObjs {
            db: self,
            stack: stack
        }
    }

    pub fn walk_vertex_buffers<'a>(&'a self) -> BTreeMapIterator<'a, ObjectKey, VertexBuffer>
    {
        self.vertex.iter()
    }

    pub fn reset_time(&mut self)
    {
        self.timing.reset()
    }

    pub fn mark_time(&mut self, name: ~str)
    {
        self.timing.mark(name)
    }

    pub fn dump_time(&mut self)
    {
        self.timing.dump()
    }
}

struct IterObjsLayer<'a>
{
    child_iter: JoinMapSetIterator<
                    BTreeSetIterator<'a, ObjectKey>,
                    BTreeMapIterator<'a, ObjectKey, position::Id>>
}

pub struct IterObjs<'a>
{
    db: &'a Database,
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
                    match self.db.index_parent_child.find(oid) {
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
}

pub struct UnwrapKey<IN>
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