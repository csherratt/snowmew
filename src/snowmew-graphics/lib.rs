#![crate_id = "github.com/csherratt/snowmew#snowmew-graphics:0.1"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "A graphics collection for snowmew"]

extern crate cow;
extern crate snowmew;
extern crate cgmath;
extern crate collision;
extern crate image = "stb_image";

use cgmath::vector::{Vector3, Vector2};
use cgmath::point::Point3;
use collision::aabb::Aabb3;

use cow::btree::{BTreeMapIterator, BTreeMap};
use snowmew::common::{Common, ObjectKey};

pub use geometry::{Geometry, VertexBuffer};
pub use material::Material;
pub use texture::Texture;
pub use light::PointLight;

pub mod geometry;
pub mod material;
pub mod default;
pub mod texture;
pub mod light;

#[deriving(Clone, Default, Eq)]
pub struct Drawable {
    pub geometry: ObjectKey,
    pub material: ObjectKey
}

impl Ord for Drawable {
    fn lt(&self, other: &Drawable) -> bool {
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
    fn cmp(&self, other: &Drawable) -> Ordering {
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
    texture:  BTreeMap<ObjectKey, Texture>,
    lights:   BTreeMap<ObjectKey, PointLight>,
}

impl GraphicsData {
    pub fn new() -> GraphicsData {
        GraphicsData {
            draw: BTreeMap::new(),
            geometry: BTreeMap::new(),
            vertex: BTreeMap::new(),
            material: BTreeMap::new(),
            texture: BTreeMap::new(),
            lights: BTreeMap::new()
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

    fn get_draw(&self, oid: ObjectKey) -> Option<Drawable> {
        match self.get_graphics().draw.find(&oid) {
            Some(d) => Some(d.clone()),
            None => None
        }
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

    fn geometry_vertex_iter<'a>(&'a self, oid: ObjectKey) -> Option<VertexBufferIter<'a>> {
        let geo = match self.get_graphics().geometry.find(&oid) {
            None => return None,
            Some(geo) => geo
        };

        let vb = match self.get_graphics().vertex.find(&geo.vb) {
            None => return None,
            Some(vb) => vb
        };

        Some(
            VertexBufferIter {
                vb: vb,
                idx_iter: vb.index.slice(geo.offset, geo.offset + geo.count).iter()
            }
        )
    }

    fn geometry_to_aabb3(&self, oid: ObjectKey) -> Option<Aabb3<f32>> {
        let iter = match self.geometry_vertex_iter(oid) {
            None => return None,
            Some(iter) => iter
        };

        Some(iter.map(|(_, p, _, _)| Point3::new(p.x, p.y, p.z)).collect())
    }

    fn new_texture(&mut self, parent: ObjectKey, name: &str, texture: Texture) -> ObjectKey {
        let oid = self.new_object(Some(parent), name);
        self.get_graphics_mut().texture.insert(oid, texture);
        oid
    }

    fn get_texture<'a>(&'a self, oid: ObjectKey) -> Option<&'a Texture> {
        self.get_graphics().texture.find(&oid)
    }

    fn texture_iter<'a>(&'a self) -> BTreeMapIterator<'a, ObjectKey, Texture> {
        self.get_graphics().texture.iter()
    }

    fn new_light(&mut self, parent: ObjectKey, name: &str, light: PointLight) -> ObjectKey {
        let oid = self.new_object(Some(parent), name);
        self.get_graphics_mut().lights.insert(oid, light);
        oid
    }

    fn get_light<'a>(&'a self, oid: ObjectKey) -> Option<&'a PointLight> {
        self.get_graphics().lights.find(&oid)
    }

    fn light_iter<'a>(&'a self) -> BTreeMapIterator<'a, ObjectKey, PointLight> {
        self.get_graphics().lights.iter()
    }
}

pub struct VertexBufferIter<'a> {
    vb: &'a VertexBuffer,
    idx_iter: std::slice::Items<'a, u32>
}

impl<'a> Iterator<(u32, &'a Vector3<f32>, Option<&'a Vector2<f32>>, Option<&'a Vector3<f32>>)> for VertexBufferIter<'a> {
    fn next(&mut self) -> Option<(u32, &'a Vector3<f32>, Option<&'a Vector2<f32>>, Option<&'a Vector3<f32>>)> {
        let idx = match self.idx_iter.next() {
            None => return None,
            Some(idx) => idx,
        };

        match self.vb.vertex {
            geometry::Geo(ref v) => {
                let v = v.get(*idx as uint);
                Some((*idx, &v.position, None, None))
            }
            geometry::GeoTex(ref v) => {
                let v = v.get(*idx as uint);
                Some((*idx, &v.position, Some(&v.texture), None))
            }
            geometry::GeoNorm(ref v) => {
                let v = v.get(*idx as uint);
                Some((*idx, &v.position, None, Some(&v.normal)))
            }
            geometry::GeoTexNorm(ref v) => {
                let v = v.get(*idx as uint);
                Some((*idx, &v.position, Some(&v.texture), Some(&v.normal)))
            }
            geometry::GeoTexNormTan(ref v) => {
                let v = v.get(*idx as uint);
                Some((*idx, &v.position, Some(&v.texture), Some(&v.normal)))
            }
        }
    }
}