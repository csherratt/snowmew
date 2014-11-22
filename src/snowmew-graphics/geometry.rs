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


use std::default::Default;
use snowmew::common::Entity;
use serialize::{Encodable, Decodable, Encoder, Decoder};

struct F32v3([f32, ..3]);

impl <E, S: Encoder<E>> Encodable<S, E> for F32v3 {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_seq(3, |s| {
            try!(s.emit_seq_elt(0, |s| self.0[0].encode(s)));
            try!(s.emit_seq_elt(1, |s| self.0[1].encode(s)));
            try!(s.emit_seq_elt(2, |s| self.0[2].encode(s)));
            Ok(())
        })
    }
}

impl <E, D: Decoder<E>> Decodable<D, E> for F32v3 {
    fn decode(d: &mut D) -> Result<F32v3, E> {
        d.read_seq(|d, _| {
            let a = try!(d.read_seq_elt(0, |d| Decodable::decode(d)));
            let b = try!(d.read_seq_elt(1, |d| Decodable::decode(d)));
            let c = try!(d.read_seq_elt(2, |d| Decodable::decode(d)));
            Ok(F32v3([a, b, c]))
        })
    }
}

struct F32v2([f32, ..2]);

impl <E, S: Encoder<E>> Encodable<S, E> for F32v2 {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_seq(2, |s| {
            try!(s.emit_seq_elt(0, |s| self.0[0].encode(s)));
            try!(s.emit_seq_elt(1, |s| self.0[1].encode(s)));
            Ok(())
        })
    }
}

impl <E, D: Decoder<E>> Decodable<D, E> for F32v2 {
    fn decode(d: &mut D) -> Result<F32v2, E> {
        d.read_seq(|d, _| {
            let a = try!(d.read_seq_elt(0, |d| Decodable::decode(d)));
            let b = try!(d.read_seq_elt(1, |d| Decodable::decode(d)));
            Ok(F32v2([a, b]))
        })
    }
}


#[deriving(Clone, Encodable, Decodable)]
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

impl <E, S: Encoder<E>> Encodable<S, E> for VertexGeo {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        F32v3(self.position).encode(s)
    }
}

impl <E, D: Decoder<E>> Decodable<D, E> for VertexGeo {
    fn decode(d: &mut D) -> Result<VertexGeo, E> {
        let pos: F32v3 = try!(Decodable::decode(d));
        Ok(VertexGeo { position: pos.0 })
    }
}

impl Clone for VertexGeo {
    fn clone(&self) -> VertexGeo {
        VertexGeo {
            position: self.position
        }
    }
}

impl PartialEq for VertexGeo {
    fn eq(&self, other: &VertexGeo) -> bool {
        self.position.as_slice() == other.position.as_slice()
    }
}

#[vertex_format]
pub struct VertexGeoNorm {
    pub position: [f32, ..3],
    pub normal:   [f32, ..3]
}

impl <E, S: Encoder<E>> Encodable<S, E> for VertexGeoNorm {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_seq(2, |s| {
            try!(s.emit_seq_elt(0, |s| F32v3(self.position).encode(s)));
            try!(s.emit_seq_elt(1, |s| F32v3(self.normal).encode(s)));
            Ok(())
        })
    }
}

impl <E, D: Decoder<E>> Decodable<D, E> for VertexGeoNorm {
    fn decode(d: &mut D) -> Result<VertexGeoNorm, E> {
        d.read_seq(|d, _| {
            let a: F32v3 = try!(d.read_seq_elt(0, |d| Decodable::decode(d)));
            let b: F32v3 = try!(d.read_seq_elt(1, |d| Decodable::decode(d)));
            Ok(VertexGeoNorm {
                position: a.0,
                normal: b.0
            })
        })
    }
}

impl Clone for VertexGeoNorm {
    fn clone(&self) -> VertexGeoNorm {
        VertexGeoNorm {
            position: self.position,
            normal: self.normal
        }
    }
}

impl PartialEq for VertexGeoNorm {
    fn eq(&self, other: &VertexGeoNorm) -> bool {
        self.position.as_slice() == other.position.as_slice() &&
        self.normal.as_slice() == other.normal.as_slice()
    }
}

#[vertex_format]
pub struct VertexGeoTex {
    pub position: [f32, ..3],
    pub texture:  [f32, ..2]
}

impl <E, S: Encoder<E>> Encodable<S, E> for VertexGeoTex {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_seq(2, |s| {
            try!(s.emit_seq_elt(0, |s| F32v3(self.position).encode(s)));
            try!(s.emit_seq_elt(1, |s| F32v2(self.texture).encode(s)));
            Ok(())
        })
    }
}

impl <E, D: Decoder<E>> Decodable<D, E> for VertexGeoTex {
    fn decode(d: &mut D) -> Result<VertexGeoTex, E> {
        d.read_seq(|d, _| {
            let a: F32v3 = try!(d.read_seq_elt(0, |d| Decodable::decode(d)));
            let b: F32v2 = try!(d.read_seq_elt(1, |d| Decodable::decode(d)));
            Ok(VertexGeoTex {
                position: a.0,
                texture: b.0
            })
        })
    }
}

impl Clone for VertexGeoTex {
    fn clone(&self) -> VertexGeoTex {
        VertexGeoTex {
            position: self.position,
            texture: self.texture
        }
    }
}

impl PartialEq for VertexGeoTex {
    fn eq(&self, other: &VertexGeoTex) -> bool {
        self.position.as_slice() == other.position.as_slice() &&
        self.texture.as_slice() == other.texture.as_slice()
    }
}


#[vertex_format]
pub struct VertexGeoTexNorm {
    pub position: [f32, ..3],
    pub texture:  [f32, ..2],
    pub normal:   [f32, ..3]
}

impl <E, S: Encoder<E>> Encodable<S, E> for VertexGeoTexNorm {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_seq(3, |s| {
            try!(s.emit_seq_elt(0, |s| F32v3(self.position).encode(s)));
            try!(s.emit_seq_elt(1, |s| F32v2(self.texture).encode(s)));
            try!(s.emit_seq_elt(2, |s| F32v3(self.normal).encode(s)));
            Ok(())
        })
    }
}

impl <E, D: Decoder<E>> Decodable<D, E> for VertexGeoTexNorm {
    fn decode(d: &mut D) -> Result<VertexGeoTexNorm, E> {
        d.read_seq(|d, _| {
            let a: F32v3 = try!(d.read_seq_elt(0, |d| Decodable::decode(d)));
            let b: F32v2 = try!(d.read_seq_elt(1, |d| Decodable::decode(d)));
            let c: F32v3 = try!(d.read_seq_elt(2, |d| Decodable::decode(d)));
            Ok(VertexGeoTexNorm {
                position: a.0,
                texture: b.0,
                normal: c.0
            })
        })
    }
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

impl PartialEq for VertexGeoTexNorm {
    fn eq(&self, other: &VertexGeoTexNorm) -> bool {
        self.position.as_slice() == other.position.as_slice() &&
        self.normal.as_slice() == other.normal.as_slice() &&
        self.texture.as_slice() == other.texture.as_slice()
    }
}


#[vertex_format]
pub struct VertexGeoTexNormTan {
    pub position: [f32, ..3],
    pub texture:  [f32, ..2],
    pub normal:   [f32, ..3],
    pub tangent:  [f32, ..3],
}

impl <E, S: Encoder<E>> Encodable<S, E> for VertexGeoTexNormTan {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_seq(4, |s| {
            try!(s.emit_seq_elt(0, |s| F32v3(self.position).encode(s)));
            try!(s.emit_seq_elt(1, |s| F32v2(self.texture).encode(s)));
            try!(s.emit_seq_elt(2, |s| F32v3(self.normal).encode(s)));
            try!(s.emit_seq_elt(3, |s| F32v3(self.tangent).encode(s)));
            Ok(())
        })
    }
}

impl <E, D: Decoder<E>> Decodable<D, E> for VertexGeoTexNormTan {
    fn decode(d: &mut D) -> Result<VertexGeoTexNormTan, E> {
        d.read_seq(|d, _| {
            let a: F32v3 = try!(d.read_seq_elt(0, |d| Decodable::decode(d)));
            let b: F32v2 = try!(d.read_seq_elt(1, |d| Decodable::decode(d)));
            let c: F32v3 = try!(d.read_seq_elt(2, |d| Decodable::decode(d)));
            let d: F32v3 = try!(d.read_seq_elt(3, |d| Decodable::decode(d)));
            Ok(VertexGeoTexNormTan {
                position: a.0,
                texture: b.0,
                normal: c.0,
                tangent: d.0
            })
        })
    }
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


#[deriving(Clone, Encodable, Decodable)]
pub enum Vertex {
    Geo(Vec<VertexGeo>),
    GeoTex(Vec<VertexGeoTex>),
    GeoNorm(Vec<VertexGeoNorm>),
    GeoTexNorm(Vec<VertexGeoTexNorm>),
    GeoTexNormTan(Vec<VertexGeoTexNormTan>)
}

impl Default for Vertex {
    fn default() -> Vertex {
        return Vertex::Geo(Vec::new())
    }
}

#[deriving(Clone, Default, Encodable, Decodable)]
pub struct VertexBuffer {
    pub vertex: Vertex,
    pub index: Vec<u32>
}


#[deriving(Clone, Default, Encodable, Decodable)]
pub struct Geometry {
    pub vb: Entity,
    pub count: uint, // number of index elements
    pub offset: uint, // offset into the index buffer
    pub prim: Primative
}

impl Default for Primative {
    fn default() -> Primative {Primative::Point}
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
    panic!("Did not find vertex!");
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
    pub fn triangles(vb: Entity, offset: uint, count: uint) -> Geometry {
        Geometry {
            vb: vb,
            count: count,
            offset: offset,
            prim: Primative::Triangle
        }
    }

    pub fn triangles_adjacency(vb: Entity, offset: uint, count: uint) -> Geometry {
        Geometry {
            vb: vb,
            count: count,
            offset: offset,
            prim: Primative::TriangleAdjacency
        }
    }

    pub fn lines(vb: Entity, offset: uint, count: uint) -> Geometry {
        Geometry {
            vb: vb,
            count: count,
            offset: offset,
            prim: Primative::Line
        }
    }

    pub fn points(vb: Entity, offset: uint, count: uint) -> Geometry {
        Geometry {
            vb: vb,
            count: count,
            offset: offset,
            prim: Primative::Point
        }
    }
}

impl VertexBuffer {
    pub fn new_position(vert: Vec<VertexGeo>, idx: Vec<u32>) -> VertexBuffer {
        VertexBuffer {
            vertex: Vertex::Geo(vert),
            index: idx
        }
    }

    pub fn new_position_texture(vert: Vec<VertexGeoTex>, idx: Vec<u32>) -> VertexBuffer {
        VertexBuffer {
            vertex: Vertex::GeoTex(vert),
            index: idx
        }
    }

    pub fn new_position_normal(vert: Vec<VertexGeoNorm>, idx: Vec<u32>) -> VertexBuffer {
        VertexBuffer {
            vertex: Vertex::GeoNorm(vert),
            index: idx
        }
    }

    pub fn new_position_texture_normal(vert: Vec<VertexGeoTexNorm>, idx: Vec<u32>) -> VertexBuffer {
        VertexBuffer {
            vertex: Vertex::GeoTexNorm(vert),
            index: idx
        }
    }

    pub fn new_position_texture_normal_tangent(vert: Vec<VertexGeoTexNormTan>, idx: Vec<u32>) -> VertexBuffer {
        VertexBuffer {
            vertex: Vertex::GeoTexNormTan(vert),
            index: idx
        }
    }
}
