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

#![crate_name = "snowmew-graphics"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "A graphics collection for snowmew"]
#![feature(phase)]

#[phase(plugin)]
extern crate gfx_macros;
extern crate gfx;
extern crate cow;
extern crate cgmath;
extern crate collision;
extern crate image = "stb_image";

extern crate snowmew  = "snowmew-core";

use std::slice;

use cgmath::vector::{Vector3, Vector2};
use cgmath::point::Point3;
use collision::sphere::Sphere;

use cow::btree::{BTreeMapIterator, BTreeMap};
use snowmew::common::{Common, ObjectKey};

pub use geometry::{Geometry, VertexBuffer};
pub use material::Material;
pub use texture::Texture;
pub use light::Light;

pub mod geometry;
pub mod material;
pub mod default;
pub mod texture;
pub mod texture_atlas;
pub mod light;

#[deriving(Clone, Default, Eq, PartialEq, PartialOrd)]
pub struct Drawable {
    pub geometry: ObjectKey,
    pub material: ObjectKey
}

impl Ord for Drawable {
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
    draw:               BTreeMap<ObjectKey, Drawable>,
    geometry:           BTreeMap<ObjectKey, Geometry>,
    sphere:             BTreeMap<ObjectKey, Sphere<f32>>,
    vertex:             BTreeMap<ObjectKey, VertexBuffer>,
    material:           BTreeMap<ObjectKey, Material>,
    material_index:     BTreeMap<ObjectKey, i32>,
    material_idx_last:  i32,
    texture:            BTreeMap<ObjectKey, Texture>,
    texture_to_atlas:   BTreeMap<ObjectKey, (uint, uint)>,
    atlases:            Vec<texture_atlas::Atlas>,
    lights:             BTreeMap<ObjectKey, light::Light>
}

impl GraphicsData {
    pub fn new() -> GraphicsData {
        GraphicsData {
            draw: BTreeMap::new(),
            geometry: BTreeMap::new(),
            vertex: BTreeMap::new(),
            material: BTreeMap::new(),
            material_index: BTreeMap::new(),
            texture: BTreeMap::new(),
            lights: BTreeMap::new(),
            atlases: Vec::new(),
            texture_to_atlas: BTreeMap::new(),
            material_idx_last: 0,
            sphere: BTreeMap::new()
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
        let sphere = self.geometry_to_collider(oid)
            .expect("Could not create sphere collider");
        println!("sphere: {}", sphere);
        self.get_graphics_mut().sphere.insert(oid, sphere);
        oid
    }

    fn sphere(&self, geo: ObjectKey) -> Sphere<f32> {
        match self.get_graphics().sphere.find(&geo) {
            Some(s) => { s.clone() }
            None => Sphere::new(Point3::new(0f32, 0., 0.,), 0f32)
        }
    }

    fn material<'a>(&'a self, oid: ObjectKey) -> Option<&'a Material> {
        self.get_graphics().material.find(&oid)
    }

    fn material_index(&self, oid: ObjectKey) -> Option<i32> {
        match self.get_graphics().material_index.find(&oid) {
            Some(idx) => Some(*idx),
            None => None
        }
    }

    fn new_material(&mut self, parent: ObjectKey, name: &str, material: Material) -> ObjectKey {
        let obj = self.new_object(Some(parent), name);
        self.get_graphics_mut().material.insert(obj, material);
        let idx = self.get_graphics().material_idx_last;
        self.get_graphics_mut().material_idx_last += 1;
        self.get_graphics_mut().material_index.insert(obj, idx);
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

    fn geometry_to_collider<B: FromIterator<Point3<f32>>>(&self, oid: ObjectKey) -> Option<B> {
        let iter = match self.geometry_vertex_iter(oid) {
            None => return None,
            Some(iter) => iter
        };

        Some(iter.map(|(_, &[x, y, z], _, _)| Point3::new(x, y, z)).collect())
    }

    fn new_texture(&mut self, parent: ObjectKey, name: &str, texture: Texture) -> ObjectKey {
        let oid = self.new_object(Some(parent), name);
        let mut found = None;
        for (idx, atlas) in self.get_graphics_mut().atlases.mut_iter().enumerate() {
            if atlas.check_texture(&texture) {
                found = Some((idx, atlas.add_texture(oid, &texture)));
                break;
            }
        }
        if found.is_none() {
            let mut atlas = texture_atlas::Atlas::new(texture.width(), texture.height(), texture.depth());
            let idx = atlas.add_texture(oid, &texture);
            let idx_atlas = self.get_graphics().atlases.len();
            self.get_graphics_mut().atlases.push(atlas);
            found = Some((idx_atlas, idx))
        }

        self.get_graphics_mut().texture.insert(oid, texture);
        self.get_graphics_mut().texture_to_atlas.insert(oid, found.unwrap());
        oid
    }

    fn get_texture<'a>(&'a self, oid: ObjectKey) -> Option<&'a Texture> {
        self.get_graphics().texture.find(&oid)
    }

    fn get_texture_atlas_index<'a>(&'a self, oid: ObjectKey) -> Option<&'a (uint, uint)> {
        self.get_graphics().texture_to_atlas.find(&oid)
    }

    fn texture_iter<'a>(&'a self) -> BTreeMapIterator<'a, ObjectKey, Texture> {
        self.get_graphics().texture.iter()
    }

    fn texture_atlas_iter<'a>(&'a self) -> slice::Items<'a, texture_atlas::Atlas> {
        self.get_graphics().atlases.iter()
    }

    fn new_light(&mut self, parent: ObjectKey, name: &str, light: Light) -> ObjectKey {
        let oid = self.new_object(Some(parent), name);
        self.get_graphics_mut().lights.insert(oid, light);
        oid
    }

    fn get_light<'a>(&'a self, oid: ObjectKey) -> Option<&'a Light> {
        self.get_graphics().lights.find(&oid)
    }

    fn light_iter<'a>(&'a self) -> BTreeMapIterator<'a, ObjectKey, Light> {
        self.get_graphics().lights.iter()
    }
}

pub struct VertexBufferIter<'a> {
    vb: &'a VertexBuffer,
    idx_iter: std::slice::Items<'a, u32>
}

impl<'a> Iterator<(u32,
                   &'a [f32, ..3],
                   Option<&'a [f32, ..2]>,
                   Option<&'a [f32, ..3]>)> for VertexBufferIter<'a> {
    fn next(&mut self) -> Option<(u32,
                                  &'a [f32, ..3],
                                  Option<&'a [f32, ..2]>,
                                  Option<&'a [f32, ..3]>)> {

        let idx = match self.idx_iter.next() {
            None => return None,
            Some(idx) => idx,
        };

        match self.vb.vertex {
            geometry::Geo(ref v) => {
                let v = &v[*idx as uint];
                Some((*idx, &v.position, None, None))
            }
            geometry::GeoTex(ref v) => {
                let v = &v[*idx as uint];
                Some((*idx, &v.position, Some(&v.texture), None))
            }
            geometry::GeoNorm(ref v) => {
                let v = &v[*idx as uint];
                Some((*idx, &v.position, None, Some(&v.normal)))
            }
            geometry::GeoTexNorm(ref v) => {
                let v = &v[*idx as uint];
                Some((*idx, &v.position, Some(&v.texture), Some(&v.normal)))
            }
            geometry::GeoTexNormTan(ref v) => {
                let v = &v[*idx as uint];
                Some((*idx, &v.position, Some(&v.texture), Some(&v.normal)))
            }
        }
    }
}