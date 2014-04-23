use geometry::{VertexBuffer, Geometry, VertexGeoTex};
use material::Material;
use graphics::Graphics;

use cgmath::vector::{Vec3, Vec2};

static VERTEX_DATA: [VertexGeoTex, ..8] = [
    // CUBE
    VertexGeoTex{position: Vec3{x: -1., y: -1., z:  1.}, texture: Vec2{x: -1., y: -1.}}, // 0
    VertexGeoTex{position: Vec3{x: -1., y:  1., z:  1.}, texture: Vec2{x: -1., y:  1.}},
    VertexGeoTex{position: Vec3{x:  1., y: -1., z:  1.}, texture: Vec2{x:  1., y: -1.}},
    VertexGeoTex{position: Vec3{x:  1., y:  1., z:  1.}, texture: Vec2{x:  1., y:  1.}},
    VertexGeoTex{position: Vec3{x: -1., y: -1., z: -1.}, texture: Vec2{x: -1., y: -1.}},
    VertexGeoTex{position: Vec3{x: -1., y:  1., z: -1.}, texture: Vec2{x: -1., y:  1.}},
    VertexGeoTex{position: Vec3{x:  1., y: -1., z: -1.}, texture: Vec2{x:  1., y: -1.}},
    VertexGeoTex{position: Vec3{x:  1., y:  1., z: -1.}, texture: Vec2{x:  1., y:  1.}},
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


static WEB_COLORS: [(&'static str, Vec3<f32>), ..16] = [
    ("white",   Vec3{x: 1.,   y: 1.,   z: 1.}),
    ("silver",  Vec3{x: 0.75, y: 0.75, z: 0.75}),
    ("gray",    Vec3{x: 0.5,  y: 0.5,  z: 0.5}),
    ("black",   Vec3{x: 0.,   y: 0.,   z: 0.}),
    ("red",     Vec3{x: 1.,   y: 0.,   z: 0.}),
    ("maroon",  Vec3{x: 0.5,  y: 0.,   z: 0.}),
    ("yellow",  Vec3{x: 1.,   y: 1.,   z: 0.}),
    ("olive",   Vec3{x: 0.5,  y: 0.5,  z: 0.}),
    ("line",    Vec3{x: 0.,   y: 1.,   z: 0.}),
    ("green",   Vec3{x: 0.,   y: 0.5,  z: 0.}),
    ("aqua",    Vec3{x: 0.,   y: 1.,   z: 1.}),
    ("teal",    Vec3{x: 0.,   y: 0.5,  z: 0.5}),
    ("blue",    Vec3{x: 0.,   y: 0.,   z: 1.}),
    ("navy",    Vec3{x: 0.,   y: 0.,   z: 0.5}),
    ("fuchsia", Vec3{x: 1.,   y: 0.,   z: 1.}),
    ("pruple",  Vec3{x: 0.5,   y: 0.,   z: 0.5}),
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
    let vbo = VertexBuffer::new_position_texture(VERTEX_DATA.into_owned(), INDEX_DATA.into_owned());
    let vbo = db.new_vertex_buffer(geo_dir, "vbo", vbo);
    db.new_geometry(geo_dir, "cube", Geometry::triangles(vbo, 0, 36));
    db.new_geometry(geo_dir, "billboard", Geometry::triangles(vbo, 0, 6));
}