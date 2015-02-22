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
#![crate_type = "lib"]
#![feature(plugin)]

#![plugin(gfx_macros)]
extern crate gfx_macros;
extern crate gfx;
extern crate cgmath;
extern crate genmesh;
extern crate image;
extern crate "rustc-serialize" as rustc_serialize;

extern crate "snowmew-core" as snowmew;

use std::cmp::Ordering::{Less, Equal, Greater};
use std::cmp::Ordering;

use snowmew::common::{Common, Entity, Duplicate, Delete};
use snowmew::table::{Static, StaticIterator};

pub use geometry::{Geometry, VertexBuffer};
pub use material::Material;
pub use texture::Texture;
pub use light::Light;

pub use light::{
    Directional,
    Point
};

pub mod geometry;
pub mod material;
pub mod standard;
pub mod texture;
pub mod light;

#[derive(Clone, Default, Eq, PartialEq, PartialOrd, Hash, Debug, RustcEncodable, RustcDecodable, Copy)]
pub struct Drawable {
    pub geometry: Entity,
    pub material: Entity
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

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct GraphicsData {
    draw:               Static<Drawable>,
    geometry:           Static<Geometry>,
    vertex:             Static<VertexBuffer>,
    material:           Static<Material>,
    material_index:     Static<i32>,
    material_idx_last:  i32,
    texture:            Static<Texture>,
    lights:             Static<light::Light>,
    standard:           Option<standard::Standard>
}

impl GraphicsData {
    pub fn new() -> GraphicsData {
        GraphicsData {
            draw: Static::new(),
            geometry: Static::new(),
            vertex: Static::new(),
            material: Static::new(),
            material_index: Static::new(),
            texture: Static::new(),
            lights: Static::new(),
            material_idx_last: 0,
            standard: None
        }
    }
}

pub trait Graphics: Common + Sized {
    fn get_graphics<'a>(&'a self) -> &'a GraphicsData;
    fn get_graphics_mut<'a>(&'a mut self) -> &'a mut GraphicsData;

    fn load_standard_graphics(&mut self) {
        let standard = standard::Standard::new(self);
        self.get_graphics_mut().standard = Some(standard);
    }

    fn standard_graphics(&self) -> &standard::Standard {
        self.get_graphics().standard.as_ref().expect("Standard graphics not loaded")
    }

    fn drawable<'a>(&'a self, key: Entity) -> Option<&'a Drawable> {
        self.get_graphics().draw.get(key)
    }

    fn new_vertex_buffer(&mut self, vb: VertexBuffer) -> Entity {
        let oid = self.new_object(None);
        self.get_graphics_mut().vertex.insert(oid, vb);
        oid
    }

    fn geometry<'a>(&'a self, oid: Entity) -> Option<&'a Geometry> {
        self.get_graphics().geometry.get(oid)
    }

    fn new_geometry(&mut self, geo: Geometry) -> Entity {
        let oid = self.new_object(None);
        self.get_graphics_mut().geometry.insert(oid, geo);
        oid
    }

    fn material<'a>(&'a self, oid: Entity) -> Option<&'a Material> {
        self.get_graphics().material.get(oid)
    }

    fn material_index(&self, oid: Entity) -> Option<i32> {
        match self.get_graphics().material_index.get(oid) {
            Some(idx) => Some(*idx),
            None => None
        }
    }

    fn new_material(&mut self, material: Material) -> Entity {
        let obj = self.new_object(None);
        self.get_graphics_mut().material.insert(obj, material);
        let idx = self.get_graphics().material_idx_last;
        self.get_graphics_mut().material_idx_last += 1;
        self.get_graphics_mut().material_index.insert(obj, idx);
        obj
    }

    fn material_iter<'a>(&'a self) -> StaticIterator<'a, Material> {
        self.get_graphics().material.iter()
    }

    fn set_draw(&mut self, oid: Entity, geo: Entity, material: Entity) {
        let draw = Drawable {
            geometry: geo,
            material: material
        };

        self.get_graphics_mut().draw.insert(oid, draw.clone());
    }

    fn get_draw(&self, oid: Entity) -> Option<Drawable> {
        match self.get_graphics().draw.get(oid) {
            Some(d) => Some(d.clone()),
            None => None
        }
    }

    fn drawable_count(&self) -> usize {
        self.get_graphics().draw.len()
    }

    fn drawable_iter<'a>(&'a self) -> StaticIterator<'a, Drawable> {
        self.get_graphics().draw.iter()
    }

    fn vertex_buffer_iter<'a>(&'a self) -> StaticIterator<'a, VertexBuffer> {
        self.get_graphics().vertex.iter()
    }

    fn geometry_vertex_iter<'a>(&'a self, oid: Entity) -> Option<VertexBufferIter<'a>> {
        let geo = match self.get_graphics().geometry.get(oid) {
            None => return None,
            Some(geo) => geo
        };

        let vb = match self.get_graphics().vertex.get(geo.vb) {
            None => return None,
            Some(vb) => vb
        };

        Some(
            VertexBufferIter {
                vb: vb,
                idx_iter: vb.index[geo.offset..geo.offset + geo.count].iter()
            }
        )
    }

    fn new_texture(&mut self, texture: Texture) -> Entity {
        let oid = self.new_object(None);
        self.get_graphics_mut().texture.insert(oid, texture);
        oid
    }

    fn get_texture<'a>(&'a self, oid: Entity) -> Option<&'a Texture> {
        self.get_graphics().texture.get(oid)
    }

    fn texture_iter<'a>(&'a self) -> StaticIterator<'a, Texture> {
        self.get_graphics().texture.iter()
    }

    fn new_light(&mut self, light: Light) -> Entity {
        let oid = self.new_object(None);
        self.get_graphics_mut().lights.insert(oid, light);
        oid
    }

    fn get_light<'a>(&'a self, oid: Entity) -> Option<&'a Light> {
        self.get_graphics().lights.get(oid)
    }

    fn light_iter<'a>(&'a self) -> StaticIterator<'a, Light> {
        self.get_graphics().lights.iter()
    }
}


impl Duplicate for GraphicsData {
    fn duplicate(&mut self, src: Entity, dst: Entity) {
        let x = self.draw.get(src).map(|x| x.clone());
        x.map(|x| self.draw.insert(dst, x));
        let x = self.geometry.get(src).map(|x| x.clone());
        x.map(|x| self.geometry.insert(dst, x));
        let x = self.vertex.get(src).map(|x| x.clone());
        x.map(|x| self.vertex.insert(dst, x));
        let x = self.material.get(src).map(|x| x.clone());
        x.map(|x| self.material.insert(dst, x));
        let x = self.material_index.get(src).map(|x| x.clone());
        x.map(|x| self.material_index.insert(dst, x));
        let x = self.texture.get(src).map(|x| x.clone());
        x.map(|x| self.texture.insert(dst, x));
        let x = self.lights.get(src).map(|x| x.clone());
        x.map(|x| self.lights.insert(dst, x));
    }
}


impl Delete for GraphicsData {
    fn delete(&mut self, oid: Entity) -> bool {
        self.draw.remove(oid)             |
        self.geometry.remove(oid)         |
        self.vertex.remove(oid)           |
        self.material.remove(oid)         |
        self.material_index.remove(oid)   |
        self.texture.remove(oid)          |
        self.lights.remove(oid)
    }
}

pub struct VertexBufferIter<'a> {
    vb: &'a VertexBuffer,
    idx_iter: std::slice::Iter<'a, u32>
}

impl<'a> Iterator for VertexBufferIter<'a> {
    type Item = (u32, &'a [f32; 3], Option<&'a [f32; 2]>, Option<&'a [f32; 3]>);

    fn next(&mut self) -> Option<(u32,
                                  &'a [f32; 3],
                                  Option<&'a [f32; 2]>,
                                  Option<&'a [f32; 3]>)> {

        let idx = match self.idx_iter.next() {
            None => return None,
            Some(idx) => idx,
        };

        match self.vb.vertex {
            geometry::Vertex::Geo(ref v) => {
                let v = &v[*idx as usize];
                Some((*idx, &v.position, None, None))
            }
            geometry::Vertex::GeoTex(ref v) => {
                let v = &v[*idx as usize];
                Some((*idx, &v.position, Some(&v.texture), None))
            }
            geometry::Vertex::GeoNorm(ref v) => {
                let v = &v[*idx as usize];
                Some((*idx, &v.position, None, Some(&v.normal)))
            }
            geometry::Vertex::GeoTexNorm(ref v) => {
                let v = &v[*idx as usize];
                Some((*idx, &v.position, Some(&v.texture), Some(&v.normal)))
            }
            geometry::Vertex::GeoTexNormTan(ref v) => {
                let v = &v[*idx as usize];
                Some((*idx, &v.position, Some(&v.texture), Some(&v.normal)))
            }
        }
    }
}
