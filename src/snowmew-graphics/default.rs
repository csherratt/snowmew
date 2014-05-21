use geometry::{VertexBuffer, Geometry, VertexGeoTexNorm};
use material::Material;
use Graphics;

use cgmath::vector::{Vector3, Vector2};

static VERTEX_DATA: [VertexGeoTexNorm, ..30] = [
    VertexGeoTexNorm{position: Vector3{x: 1f32, y: -1f32, z: -1f32}, //0
                     texture:  Vector2{x: 0.666667f32, y: 0f32},
                     normal:   Vector3{x: 0f32, y: -1f32, z: 0f32}},
    VertexGeoTexNorm{position: Vector3{x: 1f32, y: -1f32, z: 1f32},  //1
                     texture:  Vector2{x: 1f32, y: 0f32},
                     normal:   Vector3{x: 0f32, y: -1f32, z: 0f32}},
    VertexGeoTexNorm{position: Vector3{x: -1f32, y: -1f32, z: -1f32},  //2
                     texture:  Vector2{x: 0.666667f32, y: 0.333333f32},
                     normal:   Vector3{x: 0f32, y: -1f32, z: 0f32}},
    VertexGeoTexNorm{position: Vector3{x: 1f32, y: 1f32, z: -1f32},  //3
                     texture:  Vector2{x: 0f32, y: 0.666667f32},
                     normal:   Vector3{x: 0f32, y: 1f32, z: 0f32}},
    VertexGeoTexNorm{position: Vector3{x: -1f32, y: 1f32, z: -1f32},  //4
                     texture:  Vector2{x: 0f32, y: 0.333333f32},
                     normal:   Vector3{x: 0f32, y: 1f32, z: 0f32}},
    VertexGeoTexNorm{position: Vector3{x: 1f32, y: 1f32, z: 1f32}, //5
                     texture:  Vector2{x: 0.333333f32, y: 0.666667f32},
                     normal:   Vector3{x: 0f32, y: 1f32, z: 0f32}},
    VertexGeoTexNorm{position: Vector3{x: 1f32, y: -1f32, z: -1f32},  //6
                     texture:  Vector2{x: 0.666667f32, y: 0.333333f32},
                     normal:   Vector3{x: 1f32, y: 0f32, z: 0f32}},
    VertexGeoTexNorm{position: Vector3{x: 1f32, y: 1f32, z: -1f32},  //7
                     texture:  Vector2{x: 0.333333f32, y: 0.333333f32},
                     normal:   Vector3{x: 1f32, y: 0f32, z: 0f32}},
    VertexGeoTexNorm{position: Vector3{x: 1f32, y: -1f32, z: 1f32}, //8
                     texture:  Vector2{x: 0.666667f32, y: 0f32},
                     normal:   Vector3{x: 1f32, y: 0f32, z: 0f32}},
    VertexGeoTexNorm{position: Vector3{x: 1f32, y: -1f32, z: 1f32},  //9
                     texture:  Vector2{x: 0f32, y: 0.333333f32},
                     normal:   Vector3{x: 0f32, y: 0f32, z: 1f32}},
    VertexGeoTexNorm{position: Vector3{x: 1f32, y: 1f32, z: 1f32},  //10
                     texture:  Vector2{x: 0f32, y: 0f32},
                     normal:   Vector3{x: 0f32, y: 0f32, z: 1f32}},
    VertexGeoTexNorm{position: Vector3{x: -1f32, y: -1f32, z: 1f32},  //11
                     texture:  Vector2{x: 0.333333f32, y: 0.333333f32},
                     normal:   Vector3{x: 0f32, y: 0f32, z: 1f32}},
    VertexGeoTexNorm{position: Vector3{x: -1f32, y: -1f32, z: 1f32},  //12
                     texture:  Vector2{x: 0.666667f32, y: 0.333333f32},
                     normal:   Vector3{x: -1f32, y: 0f32, z: 0f32}},
    VertexGeoTexNorm{position: Vector3{x: -1f32, y: 1f32, z: 1f32}, //13
                     texture:  Vector2{x: 1f32, y: 0.333333f32},
                     normal:   Vector3{x: -1f32, y: 0f32, z: 0f32}},
    VertexGeoTexNorm{position: Vector3{x: -1f32, y: -1f32, z: -1f32},  //14
                     texture:  Vector2{x: 0.666667f32, y: 0.666667f32},
                     normal:   Vector3{x: -1f32, y: 0f32, z: 0f32}},
    VertexGeoTexNorm{position: Vector3{x: 1f32, y: 1f32, z: -1f32},  //15
                     texture:  Vector2{x: 0.333333f32, y: 0.333333f32},
                     normal:   Vector3{x: 0f32, y: 0f32, z: -1f32}},
    VertexGeoTexNorm{position: Vector3{x: 1f32, y: -1f32, z: -1f32}, //16
                     texture:  Vector2{x: 0.666667f32, y: 0.333333f32},
                     normal:   Vector3{x: 0f32, y: 0f32, z: -1f32}},
    VertexGeoTexNorm{position: Vector3{x: -1f32, y: 1f32, z: -1f32}, //17
                     texture:  Vector2{x: 0.333333f32, y: 0.666667f32},
                     normal:   Vector3{x: 0f32, y: 0f32, z: -1f32}},
    VertexGeoTexNorm{position: Vector3{x: -1f32, y: -1f32, z: 1f32}, //18
                     texture:  Vector2{x: 1f32, y: 0.333333f32},
                     normal:   Vector3{x: 0f32, y: -1f32, z: 0f32}},
    VertexGeoTexNorm{position: Vector3{x: -1f32, y: 1f32, z: 1f32}, //19
                     texture:  Vector2{x: 0.333333f32, y: 0.333334f32},
                     normal:   Vector3{x: 0f32, y: 1f32, z: 0f32}},
    VertexGeoTexNorm{position: Vector3{x: 1f32, y: 1f32, z: -1f32},  //20
                     texture:  Vector2{x: 0.333333f32, y: 0.333333f32},
                     normal:   Vector3{x: 1f32, y: 0f32, z: 0.000001f32}},
    VertexGeoTexNorm{position: Vector3{x: 1f32, y: 1f32, z: 1f32},  //21
                     texture:  Vector2{x: 0.333333f32, y: 0f32},
                     normal:   Vector3{x: 1f32, y: 0f32, z: 0.000001f32}},
    VertexGeoTexNorm{position: Vector3{x: 1f32, y: -1f32, z: 1f32},  //22
                     texture:  Vector2{x: 0.666667f32, y: 0f32},
                     normal:   Vector3{x: 1f32, y: 0f32, z: 0.000001f32}},
    VertexGeoTexNorm{position: Vector3{x: -1f32, y: 1f32, z: 1f32},  //23
                     texture:  Vector2{x: 0.333333f32, y: 0f32},
                     normal:   Vector3{x: 0f32, y: 0f32, z: 1f32}},
    VertexGeoTexNorm{position: Vector3{x: -1f32, y: 1f32, z: -1f32}, //24
                     texture:  Vector2{x: 1f32, y: 0.666667f32},
                     normal:   Vector3{x: -1f32, y: 0f32, z: 0f32}},
    VertexGeoTexNorm{position: Vector3{x: -1f32, y: -1f32, z: -1f32},  //25
                     texture:  Vector2{x: 0.666667f32, y: 0.666667f32},
                     normal:   Vector3{x: 0f32, y: 0f32, z: -1f32}},

    VertexGeoTexNorm{position: Vector3{x: -1f32, y: -1f32, z:  0f32},
                     texture:  Vector2{x: -1f32, y: -1f32},
                     normal:   Vector3{x:  0f32, y:  0f32, z:  1f32 }},
    VertexGeoTexNorm{position: Vector3{x: -1f32, y:  1f32, z:  0f32},
                     texture:  Vector2{x: -1f32, y:  1f32},
                     normal:   Vector3{x:  0f32, y:  0f32, z:  1f32 }},
    VertexGeoTexNorm{position: Vector3{x:  1f32, y: -1f32, z:  0f32},
                     texture:  Vector2{x:  1f32, y: -1f32},
                     normal:   Vector3{x:  0f32, y:  0f32, z:  1f32 }},
    VertexGeoTexNorm{position: Vector3{x:  1f32, y:  1f32, z:  0f32},
                     texture:  Vector2{x:  1f32, y:  1f32},
                     normal:   Vector3{x:  0f32, y:  0f32, z:  1f32 }},

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


static WEB_COLORS: [(&'static str, Vector3<f32>), ..16] = [
    ("white",   Vector3{x: 1.,   y: 1.,   z: 1.}),
    ("silver",  Vector3{x: 0.75, y: 0.75, z: 0.75}),
    ("gray",    Vector3{x: 0.5,  y: 0.5,  z: 0.5}),
    ("black",   Vector3{x: 0.,   y: 0.,   z: 0.}),
    ("red",     Vector3{x: 1.,   y: 0.,   z: 0.}),
    ("maroon",  Vector3{x: 0.5,  y: 0.,   z: 0.}),
    ("yellow",  Vector3{x: 1.,   y: 1.,   z: 0.}),
    ("olive",   Vector3{x: 0.5,  y: 0.5,  z: 0.}),
    ("line",    Vector3{x: 0.,   y: 1.,   z: 0.}),
    ("green",   Vector3{x: 0.,   y: 0.5,  z: 0.}),
    ("aqua",    Vector3{x: 0.,   y: 1.,   z: 1.}),
    ("teal",    Vector3{x: 0.,   y: 0.5,  z: 0.5}),
    ("blue",    Vector3{x: 0.,   y: 0.,   z: 1.}),
    ("navy",    Vector3{x: 0.,   y: 0.,   z: 0.5}),
    ("fuchsia", Vector3{x: 1.,   y: 0.,   z: 1.}),
    ("pruple",  Vector3{x: 0.5,   y: 0.,   z: 0.5}),
];

pub fn load_default(db: &mut Graphics) {
    let core_dir = db.add_dir(None, "core");
    let mat_dir = db.add_dir(Some(core_dir), "material");
    let flat_dir = db.add_dir(Some(mat_dir), "flat");

    for &(ref name, ref color) in WEB_COLORS.iter() {
        db.new_material(flat_dir, name.to_owned(), Material::simple(color.clone()));
    }

    let geo_dir = db.add_dir(Some(core_dir), "geometry");
    let vbo = VertexBuffer::new_position_texture_normal(Vec::from_slice(VERTEX_DATA), Vec::from_slice(INDEX_DATA));
    let vbo = db.new_vertex_buffer(geo_dir, "vbo", vbo);
    db.new_geometry(geo_dir, "cube", Geometry::triangles(vbo, 0, 36));
    db.new_geometry(geo_dir, "plane", Geometry::triangles(vbo, 36, 6));
}