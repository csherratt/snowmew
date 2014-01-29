use core;
use geometry::{VertexBuffer, Geometry};
use shader::Shader;

static VS_SRC: &'static str =
"#version 400
uniform mat4 position;
uniform mat4 projection;
in vec3 pos;
out vec3 UV;
void main() {
    gl_Position = projection * position * vec4(pos, 1.);
    UV = vec3(pos.x, pos.y, pos.z); 
}";

static FS_SRC: &'static str =
"#version 400
out vec4 color;
in vec3 UV;\n \
void main() {
    color = vec4(UV.x, UV.y, UV.z, 1);
}";

static VERTEX_DATA: [f32, ..24] = [
    -1., -1.,  1.,
    -1.,  1.,  1.,
     1., -1.,  1.,
     1.,  1.,  1.,
    -1., -1., -1.,
    -1.,  1., -1.,
     1., -1., -1.,
     1.,  1., -1.,
];

static INDEX_DATA: [u32, ..36] = [
    // top
    0, 2, 1,
    2, 3, 1,

    // bottom
    5, 7, 4,
    7, 6, 4,

    // right
    1, 3, 5,
    3, 7, 5,

    // left
    4, 6, 0,
    6, 2, 0,

    // front
    4, 0, 5,
    0, 1, 5,

    // back
    2, 6, 3,
    6, 7, 3,
];


pub fn load_default(db: &mut core::Database)
{
    let core_dir = db.add_dir(None, ~"core");
    let shader_dir = db.add_dir(Some(core_dir), ~"shaders");
    let geo_dir = db.add_dir(Some(core_dir), ~"geometry");
    let vbo_dir = db.add_dir(Some(core_dir), ~"geometry");

    let vbo = VertexBuffer::new(VERTEX_DATA.into_owned(), INDEX_DATA.into_owned());

    db.add_shader(shader_dir, ~"rainbow", Shader::new(VS_SRC.into_owned(), FS_SRC.into_owned()));
    let vbo = db.add_vertex_buffer(geo_dir, ~"cube_vbo", vbo);
    db.add_geometry(geo_dir, ~"cube", Geometry::triangles(vbo, 0, INDEX_DATA.len()));
}