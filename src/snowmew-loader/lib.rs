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

#![crate_name = "snowmew-loader"]
#![crate_type = "lib"]
#![allow(unstable)]

extern crate collections;
extern crate core;
extern crate cgmath;
extern crate genmesh;
extern crate image;
extern crate "obj" as obj;
extern crate "snowmew-core" as snowmew;
extern crate "snowmew-graphics" as graphics;

use std::old_io::{BufferedReader, File, IoResult};
use std::collections::HashMap;

use snowmew::common::Common;
use snowmew::Entity;
use graphics::{Graphics};
use graphics::geometry::VertexGeoTexNorm;


use genmesh::{
    Triangulate,
    MapToVertices,
    Vertices,
    LruIndexer,
    Indexer
};

mod texture;

pub struct Obj {
    path: Path,
    materials: Vec<obj::Material>,
    obj: obj::Obj<String>
}

impl Obj {
    pub fn load(path: &Path) -> IoResult<Obj> {
        File::open(path).map(|f| {
            let mut f = BufferedReader::new(f);
            let obj = obj::Obj::load(&mut f);

            let mut materials = Vec::new();

            for m in obj.materials().iter() {
                let mut p = path.clone();
                p.pop();
                p.push(&m[]);
                let file = File::open(&p).ok().expect("failed to open material");
                let mut f = BufferedReader::new(file);
                let m = obj::Mtl::load(&mut f);

                for m in m.materials.into_iter() {
                    materials.push(m);
                }
            }

            Obj {
                path: path.clone(),
                materials: materials,
                obj: obj
            }
        })
    }

    fn write_textures<G: Graphics>(&self, db: &mut G) -> HashMap<String, Entity> {
        let mut map = HashMap::new();
        for m in self.materials.iter() {
            let text = [&m.map_ka, &m.map_kd, &m.map_ks, &m.map_ke];
            for t in text.iter() {
                match *t {
                    &None => (),
                    &Some(ref t) => {
                        let insert = map.get(t).is_none();
                        if insert {
                            let mut path = self.path.clone();
                            drop(path.pop());
                            let text = texture::load_texture(&path.join(&Path::new(t.clone())));
                            let id = db.new_texture(text);
                            map.insert(t.clone(), id);
                        }
                    }
                }
            }
        }
        map
    }

    fn write_materials<G: Graphics>(&self, db: &mut G, text: &HashMap<String, Entity>)
            -> HashMap<String, snowmew::Entity> {

        let mut name_to_id = HashMap::new();

        let lookup = |&: name| {
            *text.get(name).expect("texture not found")
        };

        for m in self.materials.iter() {
            let mut mat = graphics::Material::new();
            if m.ka.is_some() { mat.set_ka(*m.ka.as_ref().unwrap()); }
            if m.kd.is_some() { mat.set_kd(*m.kd.as_ref().unwrap()); }
            if m.ks.is_some() { mat.set_ks(*m.ks.as_ref().unwrap()); }
        if m.ke.is_some() { mat.set_ke(*m.ke.as_ref().unwrap()); }
            if m.ni.is_some() { mat.set_ni(*m.ni.as_ref().unwrap()); }
            if m.ns.is_some() { mat.set_ns(*m.ns.as_ref().unwrap()); }
            if m.map_ka.is_some() { mat.set_map_ka(lookup(m.map_ka.as_ref().unwrap())); }
            if m.map_kd.is_some() { mat.set_map_kd(lookup(m.map_kd.as_ref().unwrap())); }
            if m.map_ks.is_some() { mat.set_map_ks(lookup(m.map_ks.as_ref().unwrap())); }
            if m.map_ke.is_some() { mat.set_map_ke(lookup(m.map_ke.as_ref().unwrap())); }
            let id = db.new_material(mat);
            name_to_id.insert(m.name.clone(), id);
        }

        name_to_id
    }

    pub fn import<G: Graphics>(&self, gd: &mut G) -> HashMap<String, Entity> {
        let textures = self.write_textures(gd);
        let materials = self.write_materials(gd, &textures);

        self.obj.object_iter().map(|obj| {
            let g = obj.group_iter().next().unwrap(); // expect one group only
            let mut vertices = Vec::new();
            let indices: Vec<u32> = {
                 let mut indexer = LruIndexer::new(64, |_, v| {
                    let (p, t, n): (usize, Option<usize>, Option<usize>) = v;
                    let vert = match (t, n) {
                        (Some(t), Some(n)) => {
                            VertexGeoTexNorm {
                                position: self.obj.position()[p],
                                texture: self.obj.texture()[t],
                                normal: self.obj.normal()[n]
                            }
                        }
                        (Some(t), _) => {
                            VertexGeoTexNorm {
                                position: self.obj.position()[p],
                                texture: self.obj.texture()[t],
                                normal: [1., 0., 0.]
                            }
                        }
                        (_, Some(n)) => {
                            VertexGeoTexNorm {
                                position: self.obj.position()[p],
                                texture: [0., 0.],
                                normal: self.obj.normal()[n]
                            }
                        }
                        (_, _) => {
                            VertexGeoTexNorm {
                                position: self.obj.position()[p],
                                texture: [0., 0.],
                                normal: [1., 0., 0.]
                            }
                        }
                    };
                    vertices.push(vert)
                });

                g.indices().iter()
                    .map(|x| *x)
                    .triangulate()
                    .vertex(|v| indexer.index(v) as u32)
                    .vertices()
                    .collect()
            };

            let len = indices.len();
            let vbo = gd.new_vertex_buffer(
                graphics::VertexBuffer::new_position_texture_normal(vertices, indices)
            );

            let geo = gd.new_geometry(
                graphics::Geometry::triangles(vbo, 0, len)
            );

            let mat_name = g.material.clone().expect("no material");
            let mat = materials.get(&mat_name).expect("unresolved mat");
            let o = gd.new_object(None);
            gd.set_draw(o, geo, *mat);
            (obj.name.clone(), o)
        }).collect()
    }
}