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

use snowmew::Entity;
use geometry::{VertexBuffer, Geometry, VertexGeoTexNorm};
use material::Material;
use Graphics;

use genmesh::generators::{Plane, Cube, SphereUV};
use genmesh::{MapToVertices, Indexer, LruIndexer};
use genmesh::{Vertices, Triangulate, Quad, Polygon};
use cgmath::{Vector3, EuclideanVector};

fn build_vectors<T: Iterator<Quad<VertexGeoTexNorm>>>(input: T)
    -> (Vec<VertexGeoTexNorm>, Vec<u32>) {

    let mut mesh_data: Vec<VertexGeoTexNorm> = Vec::new();
    let index: Vec<u32> = {
        let mut indexer = LruIndexer::new(16, |_, v| mesh_data.push(v));
        input.map(|mut p| {
            let a = Vector3::new(p.x.position[0],
                                 p.x.position[1],
                                 p.x.position[2]);
            let b = Vector3::new(p.y.position[0],
                                 p.y.position[1],
                                 p.y.position[2]);
            let c = Vector3::new(p.z.position[0],
                                 p.z.position[1],
                                 p.z.position[2]);

            let normal = (a - b).cross(&(b - c)).normalize();

            p.x.normal = [normal.x, normal.y, normal.z];
            p.y.normal = [normal.x, normal.y, normal.z];
            p.z.normal = [normal.x, normal.y, normal.z];
            p.w.normal = [normal.x, normal.y, normal.z];

            p.x.texture = [-1., -1.];
            p.y.texture = [-1.,  1.];
            p.z.texture = [ 1.,  1.];
            p.w.texture = [ 1., -1.];

            p
        })
        .vertex(|v| indexer.index(v) as u32)
        .triangulate()
        .vertices()
        .collect()
    };

    (mesh_data, index)
}

fn build_vectors_poly<T: Iterator<Polygon<(f32, f32, f32)>>>(input: T)
    -> (Vec<VertexGeoTexNorm>, Vec<u32>) {

    let mut mesh_data: Vec<VertexGeoTexNorm> = Vec::new();
    let index: Vec<u32> = {
        let mut indexer = LruIndexer::new(16, |_, v| mesh_data.push(v));
        input
        .vertex(|(x, y, z)| {
            let n = Vector3::new(x, y, z).normalize();
            VertexGeoTexNorm {
                position: [x, y, z],
                texture: [0., 0.],
                normal: [n.x, n.y, n.z]
            }
        })
        .vertex(|v| indexer.index(v) as u32)
        .triangulate()
        .vertices()
        .collect()
    };

    (mesh_data, index)
}

#[deriving(Clone, RustcEncodable, RustcDecodable, Copy)]
pub struct StandardColors {
    pub white: Entity,
    pub silver: Entity,
    pub gray: Entity,
    pub black: Entity,
    pub red: Entity,
    pub maroon: Entity,
    pub yellow: Entity,
    pub olive: Entity,
    pub line: Entity,
    pub green: Entity,
    pub aqua: Entity,
    pub teal: Entity,
    pub blue: Entity,
    pub navy: Entity,
    pub fuchsia: Entity,
    pub purple: Entity,
}

#[deriving(Clone, RustcEncodable, RustcDecodable, Copy)]
pub struct Materials {
    pub flat: StandardColors
}

#[deriving(Clone, RustcEncodable, RustcDecodable, Copy)]
pub struct Spheres {
    pub uv_2: Entity,
    pub uv_4: Entity,
    pub uv_8: Entity,
    pub uv_16: Entity,
    pub uv_32: Entity,
    pub uv_64: Entity,
    pub uv_128: Entity,
    pub uv_256: Entity,
}

#[deriving(Clone, RustcEncodable, RustcDecodable, Copy)]
pub struct Shapes {
    pub cube: Entity,
    pub plane: Entity,
    pub sphere: Spheres
}

#[deriving(Clone, RustcEncodable, RustcDecodable, Copy)]
pub struct Standard {
    pub materials: Materials,
    pub shapes: Shapes
}

fn build_sphere<G: Graphics>(db: &mut G, size: uint) -> Entity {
    let (sphere_v, sphere_i) = build_vectors_poly(SphereUV::new(size, size));
    let sphere_len = sphere_i.len();
    let sphere_vb = VertexBuffer::new_position_texture_normal(sphere_v, sphere_i);
    let sphere_vbo = db.new_vertex_buffer(sphere_vb);
    db.new_geometry(Geometry::triangles(sphere_vbo, 0, sphere_len))
}

impl Standard {
    pub fn new<G: Graphics>(db: &mut G) -> Standard {
        let flat = StandardColors {
            white:   db.new_material(Material::simple([1., 1., 1.])),
            silver:  db.new_material(Material::simple([0.75, 0.75, 0.75])),
            gray:    db.new_material(Material::simple([0.5, 0.5, 0.5])),
            black:   db.new_material(Material::simple([0., 0., 0.])),
            red:     db.new_material(Material::simple([1., 0., 0.])),
            maroon:  db.new_material(Material::simple([0.5, 0., 0.])),
            yellow:  db.new_material(Material::simple([1., 1., 0.])),
            olive:   db.new_material(Material::simple([0.5, 0.5, 0.])),
            line:    db.new_material(Material::simple([0., 1., 0.])),
            green:   db.new_material(Material::simple([0., 0.5, 0.])),
            aqua:    db.new_material(Material::simple([0., 1., 1.])),
            teal:    db.new_material(Material::simple([0., 0.5, 0.5])),
            blue:    db.new_material(Material::simple([0., 0., 1.])),
            navy:    db.new_material(Material::simple([0., 0., 0.5])),
            fuchsia: db.new_material(Material::simple([1., 0., 1.])),
            purple:  db.new_material(Material::simple([0.5, 0., 0.5]))
        };

        let spheres = Spheres {
            uv_2: build_sphere(db, 2),
            uv_4: build_sphere(db, 4),
            uv_8: build_sphere(db, 8),
            uv_16: build_sphere(db, 16),
            uv_32: build_sphere(db, 32),
            uv_64: build_sphere(db, 64),
            uv_128: build_sphere(db, 128),
            uv_256: build_sphere(db, 256)
        };

        let cube = {
            let (cube_v, cube_i) = build_vectors(
                Cube::new().vertex(|(x, y, z)| {
                    VertexGeoTexNorm {
                        position: [x, y, z],
                        texture: [0., 0.],
                        normal: [0., 0., 0.]
                    }
                }
            ));
            let cube_len = cube_i.len();
            let cube_vb = VertexBuffer::new_position_texture_normal(cube_v, cube_i);
            let cube_vbo = db.new_vertex_buffer(cube_vb);
            db.new_geometry(Geometry::triangles(cube_vbo, 0, cube_len))
        };

        let plane = {
            let (plane_v, plane_i) = build_vectors(
                Plane::new().vertex(|(x, y)| {
                    VertexGeoTexNorm {
                        position: [x, y, 0.],
                        texture: [0., 0.],
                        normal: [0., 0., 0.]
                    }
                }
            ));
            let plane_len = plane_i.len();
            let plane_vb = VertexBuffer::new_position_texture_normal(plane_v, plane_i);
            let plane_vbo = db.new_vertex_buffer(plane_vb);
            db.new_geometry(Geometry::triangles(plane_vbo, 0, plane_len))
        };


        Standard {
            materials: Materials {
                flat: flat
            },
            shapes: Shapes {
                cube: cube,
                plane: plane,
                sphere: spheres
            }
        }
    }
}