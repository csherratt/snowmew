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

use cgmath::{Vector3, Vector2};

static VERTEX_DATA: [VertexGeoTexNorm, ..30] = [
    VertexGeoTexNorm{position: [1f32, -1f32, -1f32], //0
                     texture:  [0.666667f32, 0f32],
                     normal:   [0f32, -1f32, 0f32]},
    VertexGeoTexNorm{position: [1f32, -1f32, 1f32],  //1
                     texture:  [1f32, 0f32],
                     normal:   [0f32, -1f32, 0f32]},
    VertexGeoTexNorm{position: [-1f32, -1f32, -1f32],  //2
                     texture:  [0.666667f32, 0.333333f32],
                     normal:   [0f32, -1f32, 0f32]},
    VertexGeoTexNorm{position: [1f32, 1f32, -1f32],  //3
                     texture:  [0f32, 0.666667f32],
                     normal:   [0f32, 1f32, 0f32]},
    VertexGeoTexNorm{position: [-1f32, 1f32, -1f32],  //4
                     texture:  [0f32, 0.333333f32],
                     normal:   [0f32, 1f32, 0f32]},
    VertexGeoTexNorm{position: [1f32, 1f32, 1f32], //5
                     texture:  [0.333333f32, 0.666667f32],
                     normal:   [0f32, 1f32, 0f32]},
    VertexGeoTexNorm{position: [1f32, -1f32, -1f32],  //6
                     texture:  [0.666667f32, 0.333333f32],
                     normal:   [1f32, 0f32, 0f32]},
    VertexGeoTexNorm{position: [1f32, 1f32, -1f32],  //7
                     texture:  [0.333333f32, 0.333333f32],
                     normal:   [1f32, 0f32, 0f32]},
    VertexGeoTexNorm{position: [1f32, -1f32, 1f32], //8
                     texture:  [0.666667f32, 0f32],
                     normal:   [1f32, 0f32, 0f32]},
    VertexGeoTexNorm{position: [1f32, -1f32, 1f32],  //9
                     texture:  [0f32, 0.333333f32],
                     normal:   [0f32, 0f32, 1f32]},
    VertexGeoTexNorm{position: [1f32, 1f32, 1f32],  //10
                     texture:  [0f32, 0f32],
                     normal:   [0f32, 0f32, 1f32]},
    VertexGeoTexNorm{position: [-1f32, -1f32, 1f32],  //11
                     texture:  [0.333333f32, 0.333333f32],
                     normal:   [0f32, 0f32, 1f32]},
    VertexGeoTexNorm{position: [-1f32, -1f32, 1f32],  //12
                     texture:  [0.666667f32, 0.333333f32],
                     normal:   [-1f32, 0f32, 0f32]},
    VertexGeoTexNorm{position: [-1f32, 1f32, 1f32], //13
                     texture:  [1f32, 0.333333f32],
                     normal:   [-1f32, 0f32, 0f32]},
    VertexGeoTexNorm{position: [-1f32, -1f32, -1f32],  //14
                     texture:  [0.666667f32, 0.666667f32],
                     normal:   [-1f32, 0f32, 0f32]},
    VertexGeoTexNorm{position: [1f32, 1f32, -1f32],  //15
                     texture:  [0.333333f32, 0.333333f32],
                     normal:   [0f32, 0f32, -1f32]},
    VertexGeoTexNorm{position: [1f32, -1f32, -1f32], //16
                     texture:  [0.666667f32, 0.333333f32],
                     normal:   [0f32, 0f32, -1f32]},
    VertexGeoTexNorm{position: [-1f32, 1f32, -1f32], //17
                     texture:  [0.333333f32, 0.666667f32],
                     normal:   [0f32, 0f32, -1f32]},
    VertexGeoTexNorm{position: [-1f32, -1f32, 1f32], //18
                     texture:  [1f32, 0.333333f32],
                     normal:   [0f32, -1f32, 0f32]},
    VertexGeoTexNorm{position: [-1f32, 1f32, 1f32], //19
                     texture:  [0.333333f32, 0.333334f32],
                     normal:   [0f32, 1f32, 0f32]},
    VertexGeoTexNorm{position: [1f32, 1f32, -1f32],  //20
                     texture:  [0.333333f32, 0.333333f32],
                     normal:   [1f32, 0f32, 0.000001f32]},
    VertexGeoTexNorm{position: [1f32, 1f32, 1f32],  //21
                     texture:  [0.333333f32, 0f32],
                     normal:   [1f32, 0f32, 0.000001f32]},
    VertexGeoTexNorm{position: [1f32, -1f32, 1f32],  //22
                     texture:  [0.666667f32, 0f32],
                     normal:   [1f32, 0f32, 0.000001f32]},
    VertexGeoTexNorm{position: [-1f32, 1f32, 1f32],  //23
                     texture:  [0.333333f32, 0f32],
                     normal:   [0f32, 0f32, 1f32]},
    VertexGeoTexNorm{position: [-1f32, 1f32, -1f32], //24
                     texture:  [1f32, 0.666667f32],
                     normal:   [-1f32, 0f32, 0f32]},
    VertexGeoTexNorm{position: [-1f32, -1f32, -1f32],  //25
                     texture:  [0.666667f32, 0.666667f32],
                     normal:   [0f32, 0f32, -1f32]},

    VertexGeoTexNorm{position: [-1f32, -1f32, 0f32],
                     texture:  [-1f32, -1f32],
                     normal:   [0f32, 0f32, 1f32]},
    VertexGeoTexNorm{position: [-1f32, 1f32, 0f32],
                     texture:  [-1f32, 1f32],
                     normal:   [0f32, 0f32, 1f32]},
    VertexGeoTexNorm{position: [1f32, -1f32, 0f32],
                     texture:  [1f32, -1f32],
                     normal:   [0f32, 0f32, 1f32]},
    VertexGeoTexNorm{position: [1f32, 1f32, 0f32],
                     texture:  [1f32, 1f32],
                     normal:   [0f32, 0f32, 1f32]},

];

static INDEX_DATA: [u32, ..42] = [
    0,  1,  2,
    3,  4,  5,
    6,  7,  8,
    9,  10, 11,
    12, 13, 14,
    15, 16, 17,
    1,  18, 2,
    4,  19, 5,
    20, 21, 22,
    10, 23, 11,
    13, 24, 14,
    16, 25, 17,

    26, 28, 27,
    28, 29, 27,
];


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

pub fn load_default(db: &mut Graphics) {
    let core_dir = db.add_dir(None, "core");
    let mat_dir = db.add_dir(Some(core_dir), "material");
    let flat_dir = db.add_dir(Some(mat_dir), "flat");

    for &(name, ref color) in WEB_COLORS.iter() {
        db.new_material(flat_dir, name, Material::simple(*color));
    }

    let geo_dir = db.add_dir(Some(core_dir), "geometry");
    let vbo = VertexBuffer::new_position_texture_normal(Vec::from_slice(VERTEX_DATA), Vec::from_slice(INDEX_DATA));
    let vbo = db.new_vertex_buffer(geo_dir, "vbo", vbo);
    db.new_geometry(geo_dir, "cube", Geometry::triangles(vbo, 0, 36));
    db.new_geometry(geo_dir, "plane", Geometry::triangles(vbo, 36, 6));
}