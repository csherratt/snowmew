
use std::default::Default;
use cgmath::vector::{Vector2, Vector3};

use snowmew::common::ObjectKey;

#[deriving(Clone)]
pub enum Primative {
    Point,
    Line,
    Triangle,
    TriangleAdjacency
}

#[deriving(Clone)]
pub struct VertexGeo {
    pub position: Vector3<f32>
}

#[deriving(Clone)]
pub struct VertexGeoTex {
    pub position: Vector3<f32>,
    pub texture: Vector2<f32>
}

#[deriving(Clone)]
pub struct VertexGetTexNorm {
    pub position: Vector3<f32>,
    pub texture: Vector2<f32>,
    pub normal: Vector3<f32>
}

#[deriving(Clone)]
pub enum Vertex {
    Geo(Vec<VertexGeo>),
    GeoTex(Vec<VertexGeoTex>),
    GeoTexNorm(Vec<VertexGetTexNorm>)
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
            for j in range(0, 3) {
                if a == index[(i*3+j) as uint] {
                    found_a = j;
                }
                if b == index[(i*3+j) as uint] {
                    found_b = j;
                }
            }

            /* found a candidate */
            if found_a != -1 && found_b != -1  {
                for j in range(0, 3) {
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

    pub fn new_position_texture_normal(vert: Vec<VertexGetTexNorm>, idx: Vec<u32>) -> VertexBuffer {
        VertexBuffer {
            vertex: GeoTexNorm(vert),
            index: idx
        }
    }
}