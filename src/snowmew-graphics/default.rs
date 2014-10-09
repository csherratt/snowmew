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

use geometry::{VertexBuffer, Geometry, VertexGeoTexNorm};
use material::Material;
use Graphics;

use genmesh::generators::{Plane, Cube, SphereUV};
use genmesh::{MapToVertices, Indexer, LruIndexer};
use genmesh::{Vertices, Triangulate, Quad, Polygon};
use cgmath::{Vector3, EuclideanVector};

use std::num::pow;

static WEB_COLORS: [(&'static str, [f32, ..3]), ..16] = [
    ("white",   [1., 1., 1.]),
    ("silver",  [0.75, 0.75, 0.75]),
    ("gray",    [0.5, 0.5, 0.5]),
    ("black",   [0., 0., 0.]),
    ("red",     [1., 0., 0.]),
    ("maroon",  [0.5, 0., 0.]),
    ("yellow",  [1., 1., 0.]),
    ("olive",   [0.5, 0.5, 0.]),
    ("line",    [0., 1., 0.]),
    ("green",   [0., 0.5, 0.]),
    ("aqua",    [0., 1., 1.]),
    ("teal",    [0., 0.5, 0.5]),
    ("blue",    [0., 0., 1.]),
    ("navy",    [0., 0., 0.5]),
    ("fuchsia", [1., 0., 1.]),
    ("pruple",  [0.5, 0., 0.5]),
];

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

pub fn load_default(db: &mut Graphics) {
    let core_dir = db.add_dir(None, "core");
    let mat_dir = db.add_dir(Some(core_dir), "material");
    let flat_dir = db.add_dir(Some(mat_dir), "flat");

    for &(name, ref color) in WEB_COLORS.iter() {
        db.new_material(flat_dir, name, Material::simple(*color));
    }

    let geo_dir = db.add_dir(Some(core_dir), "geometry");

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
    let cube_vbo = db.new_vertex_buffer(geo_dir, "cube_vbo", cube_vb);
    db.new_geometry(geo_dir, "cube", Geometry::triangles(cube_vbo, 0, cube_len));

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
    let plane_vbo = db.new_vertex_buffer(geo_dir, "plane_vbo", plane_vb);
    db.new_geometry(geo_dir, "plane", Geometry::triangles(plane_vbo, 0, plane_len));

    for i in range(2, 8) {
        for j in range(2, 8) {
            let i = pow(2, i);
            let j = pow(2, j);
            let (sphere_v, sphere_i) = build_vectors_poly(SphereUV::new(i, j));
            let sphere_len = sphere_i.len();
            let sphere_vb = VertexBuffer::new_position_texture_normal(sphere_v, sphere_i);
            let sphere_vbo = db.new_vertex_buffer(geo_dir,
                format!("sphere_uv_{}_{}_vbo", i, j).as_slice(),
                sphere_vb
            );
            db.new_geometry(geo_dir,
                format!("sphere_uv_{}_{}", i, j).as_slice(),
                Geometry::triangles(sphere_vbo, 0, sphere_len)
            );
        }
    }
}