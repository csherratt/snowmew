
use cow::btree::{BTreeMapIterator, BTreeMap};

use common::{Common, ObjectKey};

use geometry::{Geometry, VertexBuffer};
use material::Material;

#[deriving(Clone, Default, Eq)]
pub struct Drawable {
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

#[deriving(Clone)]
pub struct GraphicsData {
    draw:     BTreeMap<ObjectKey, Drawable>,
    geometry: BTreeMap<ObjectKey, Geometry>,
    vertex:   BTreeMap<ObjectKey, VertexBuffer>,
    material: BTreeMap<ObjectKey, Material>,
}

impl GraphicsData {
    pub fn new() -> GraphicsData {
        GraphicsData {
            draw: BTreeMap::new(),
            geometry: BTreeMap::new(),
            vertex: BTreeMap::new(),
            material: BTreeMap::new()
        }
    }
}

pub trait Graphics: Common {
    fn get_graphics<'a>(&'a self) -> &'a GraphicsData;
    fn get_graphics_mut<'a>(&'a mut self) -> &'a mut GraphicsData;

    fn drawable<'a>(&'a self, key: ObjectKey) -> Option<&'a Drawable> {
        self.get_graphics().draw.find(&key)
    }

    fn new_vertex_buffer(&mut self, parent: ObjectKey, name: &str, vb: VertexBuffer) -> ObjectKey {
        let oid = self.new_object(Some(parent), name);
        self.get_graphics_mut().vertex.insert(oid, vb);
        oid
    }

    fn geometry<'a>(&'a self, oid: ObjectKey) -> Option<&'a Geometry> {
        self.get_graphics().geometry.find(&oid)
    }

    fn new_geometry(&mut self, parent: ObjectKey, name: &str, geo: Geometry) -> ObjectKey {
        let oid = self.new_object(Some(parent), name);
        self.get_graphics_mut().geometry.insert(oid, geo);
        oid
    }

    fn material<'a>(&'a self, oid: ObjectKey) -> Option<&'a Material> {
        self.get_graphics().material.find(&oid)
    }

    fn new_material(&mut self, parent: ObjectKey, name: &str, material: Material) -> ObjectKey {
        let obj = self.new_object(Some(parent), name);
        self.get_graphics_mut().material.insert(obj, material);
        obj
    }

    fn material_iter<'a>(&'a self) -> BTreeMapIterator<'a, ObjectKey, Material> {
        self.get_graphics().material.iter()
    }

    fn set_draw(&mut self, oid: ObjectKey, geo: ObjectKey, material: ObjectKey) {
        let draw = Drawable {
            geometry: geo,
            material: material
        };

        self.get_graphics_mut().draw.insert(oid, draw.clone());
    }

    fn drawable_count(&self) -> uint {
        self.get_graphics().draw.len()
    }

    fn drawable_iter<'a>(&'a self) -> BTreeMapIterator<'a, ObjectKey, Drawable> {
        self.get_graphics().draw.iter()
    }

    fn vertex_buffer_iter<'a>(&'a self) -> BTreeMapIterator<'a, ObjectKey, VertexBuffer> {
        self.get_graphics().vertex.iter()
    }
}
