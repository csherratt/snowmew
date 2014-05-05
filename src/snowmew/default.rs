use geometry::{VertexBuffer, Geometry, VertexGeoTex};
use material::Material;
use graphics::Graphics;

use cgmath::vector::{Vector3, Vector2};

static VERTEX_DATA: [VertexGeoTex, ..8] = [
    // CUBE
    VertexGeoTex{position: Vector3{x: -1., y: -1., z:  1.}, texture: Vector2{x: -1., y: -1.}}, // 0
    VertexGeoTex{position: Vector3{x: -1., y:  1., z:  1.}, texture: Vector2{x: -1., y:  1.}},
    VertexGeoTex{position: Vector3{x:  1., y: -1., z:  1.}, texture: Vector2{x:  1., y: -1.}},
    VertexGeoTex{position: Vector3{x:  1., y:  1., z:  1.}, texture: Vector2{x:  1., y:  1.}},
    VertexGeoTex{position: Vector3{x: -1., y: -1., z: -1.}, texture: Vector2{x: -1., y: -1.}},
    VertexGeoTex{position: Vector3{x: -1., y:  1., z: -1.}, texture: Vector2{x: -1., y:  1.}},
    VertexGeoTex{position: Vector3{x:  1., y: -1., z: -1.}, texture: Vector2{x:  1., y: -1.}},
    VertexGeoTex{position: Vector3{x:  1., y:  1., z: -1.}, texture: Vector2{x:  1., y:  1.}},
];

static INDEX_DATA: [u32, ..36] = [
    // cube top
    0, 2, 1,
    2, 3, 1,
    // cube bottom
    5, 7, 4,
    7, 6, 4,
    // cube right
    1, 3, 5,
    3, 7, 5,
    // cube left
    4, 6, 0,
    6, 2, 0,
    // cube front
    4, 0, 5,
    0, 1, 5,
    // cube back
    2, 6, 3,
    6, 7, 3,
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

pub fn load_default(db: &mut Graphics)
{
    let core_dir = db.add_dir(None, "core");
    let mat_dir = db.add_dir(Some(core_dir), "material");
    let flat_dir = db.add_dir(Some(mat_dir), "flat");

    for &(ref name, ref color) in WEB_COLORS.iter() {
        db.new_material(flat_dir, name.to_owned(), Material::flat(color.clone()));
    }

    let geo_dir = db.add_dir(Some(core_dir), "geometry");
    let vbo = VertexBuffer::new_position_texture(Vec::from_slice(VERTEX_DATA), Vec::from_slice(INDEX_DATA));
    let vbo = db.new_vertex_buffer(geo_dir, "vbo", vbo);
    db.new_geometry(geo_dir, "cube", Geometry::triangles(vbo, 0, 36));
    db.new_geometry(geo_dir, "billboard", Geometry::triangles(vbo, 0, 6));
}