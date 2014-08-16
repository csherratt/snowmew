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

use gfx;

use std::default::Default;
use cgmath::{Vector2, Vector3};
use snowmew::common::ObjectKey;

#[deriving(Clone)]
pub enum Primative {
    Point,
    Line,
    Triangle,
    TriangleAdjacency
}
#[vertex_format]
pub struct VertexGeo {
    pub position: [f32, ..3]
}

impl Clone for VertexGeo {
    fn clone(&self) -> VertexGeo {
        VertexGeo {
            position: self.position
        }
    }
}

#[vertex_format]
pub struct VertexGeoNorm {
    pub position: [f32, ..3],
    pub normal:   [f32, ..3]
}

impl Clone for VertexGeoNorm {
    fn clone(&self) -> VertexGeoNorm {
        VertexGeoNorm {
            position: self.position,
            normal: self.normal
        }
    }
}

#[vertex_format]
pub struct VertexGeoTex {
    pub position: [f32, ..3],
    pub texture:  [f32, ..2]
}

impl Clone for VertexGeoTex {
    fn clone(&self) -> VertexGeoTex {
        VertexGeoTex {
            position: self.position,
            texture: self.texture
        }
    }
}

#[vertex_format]
pub struct VertexGeoTexNorm {
    pub position: [f32, ..3],
    pub texture:  [f32, ..2],
    pub normal:   [f32, ..3]
}

impl Clone for VertexGeoTexNorm {
    fn clone(&self) -> VertexGeoTexNorm {
        VertexGeoTexNorm {
            position: self.position,
            texture: self.texture,
            normal: self.normal
        }
    }
}

#[vertex_format]
pub struct VertexGeoTexNormTan {
    pub position: [f32, ..3],
    pub texture:  [f32, ..2],
    pub normal:   [f32, ..3],
    pub tangent:  [f32, ..3],
}

impl Clone for VertexGeoTexNormTan {
    fn clone(&self) -> VertexGeoTexNormTan {
        VertexGeoTexNormTan {
            position: self.position,
            texture: self.texture,
            normal: self.normal,
            tangent: self.tangent
        }
    }
}


#[deriving(Clone)]
pub enum Vertex {
    Geo(Vec<VertexGeo>),
    GeoTex(Vec<VertexGeoTex>),
    GeoNorm(Vec<VertexGeoNorm>),
    GeoTexNorm(Vec<VertexGeoTexNorm>),
    GeoTexNormTan(Vec<VertexGeoTexNormTan>)
}

impl Default for Vertex {
    fn default() -> Vertex {
        return Geo(Vec::new())
    }
}

#[deriving(Clone, Default)]
pub struct VertexBuffer {
    pub vertex: Vertex,
    pub index: Vec<u32>
}


#[deriving(Clone, Default)]
pub struct Geometry {
    pub vb: ObjectKey,
    pub count: uint, // number of index elements
    pub offset: uint, // offset into the index buffer
    pub prim: Primative
}

impl Default for Primative {
    fn default() -> Primative {Point}
}

fn find_trig<IDX: Eq+Clone>(index: &[IDX], my_idx: uint, a: IDX, b: IDX) -> IDX {
    let my_idx = my_idx as int;
    for i in range(0, (index.len()/3) as int) {
        if i != my_idx {
            /* look for candidate */
            let mut found_a = -1;
            let mut found_b = -1;
            for j in range(0i, 3) {
                if a == index[(i*3+j) as uint] {
                    found_a = j;
                }
                if b == index[(i*3+j) as uint] {
                    found_b = j;
                }
            }

            /* found a candidate */
            if found_a != -1 && found_b != -1  {
                for j in range(0i, 3) {
                    if j != found_a && j != found_b {
                        return index[(i*3+j) as uint].clone();
                    }
                }
            }
        }
    }
    fail!("Did not find vertex!");
}

pub fn to_triangles_adjacency<IDX: Eq+Clone>(index: &[IDX]) -> Vec<IDX> {
    let mut vec = Vec::with_capacity(index.len()*2);
    for i in range(0, index.len()/3) {
        let a = &index[i*3];
        let b = &index[i*3+1];
        let c = &index[i*3+2];

        vec.push(a.clone());
        vec.push(find_trig(index, i, a.clone(), b.clone()).clone());
        vec.push(b.clone());
        vec.push(find_trig(index, i, b.clone(), c.clone()).clone());
        vec.push(c.clone());
        vec.push(find_trig(index, i, c.clone(), a.clone()).clone());
    }
    vec
}

impl Geometry {
    pub fn triangles(vb: ObjectKey, offset: uint, count: uint) -> Geometry {
        Geometry {
            vb: vb,
            count: count,
            offset: offset,
            prim: Triangle
        }
    }

    pub fn triangles_adjacency(vb: ObjectKey, offset: uint, count: uint) -> Geometry {
        Geometry {
            vb: vb,
            count: count,
            offset: offset,
            prim: TriangleAdjacency
        }
    }

    pub fn lines(vb: ObjectKey, offset: uint, count: uint) -> Geometry {
        Geometry {
            vb: vb,
            count: count,
            offset: offset,
            prim: Line
        }
    }

    pub fn points(vb: ObjectKey, offset: uint, count: uint) -> Geometry {
        Geometry {
            vb: vb,
            count: count,
            offset: offset,
            prim: Point
        }
    }
}

impl VertexBuffer {
    pub fn new_position(vert: Vec<VertexGeo>, idx: Vec<u32>) -> VertexBuffer {
        VertexBuffer {
            vertex: Geo(vert),
            index: idx
        }
    }

    pub fn new_position_texture(vert: Vec<VertexGeoTex>, idx: Vec<u32>) -> VertexBuffer {
        VertexBuffer {
            vertex: GeoTex(vert),
            index: idx
        }
    }

    pub fn new_position_normal(vert: Vec<VertexGeoNorm>, idx: Vec<u32>) -> VertexBuffer {
        VertexBuffer {
            vertex: GeoNorm(vert),
            index: idx
        }
    }

    pub fn new_position_texture_normal(vert: Vec<VertexGeoTexNorm>, idx: Vec<u32>) -> VertexBuffer {
        VertexBuffer {
            vertex: GeoTexNorm(vert),
            index: idx
        }
    }

    pub fn new_position_texture_normal_tangent(vert: Vec<VertexGeoTexNormTan>, idx: Vec<u32>) -> VertexBuffer {
        VertexBuffer {
            vertex: GeoTexNormTan(vert),
            index: idx
        }
    }
}
