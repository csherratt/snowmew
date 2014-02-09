use core;
use geometry::{VertexBuffer, Geometry, VertexGeoTex};
use shader::Shader;

use cgmath::vector::{Vec3, Vec2};

use ovr;

static VS_SRC: &'static str =
"#version 400
uniform mat4 mat_model;
uniform mat4 mat_proj_view;

in vec3 position;
in vec2 in_texture;
in vec3 in_normal;

out vec2 fs_texture;
out vec3 fs_normal;

void main() {
    gl_Position = mat_proj_view * mat_model * vec4(position, 1.);
    fs_texture = in_texture;
    fs_normal = in_normal;
}
";

static FS_RAINBOW_NORMAL_SRC: &'static str =
"#version 400

in vec2 fs_texture;
in vec3 fs_normal;

out vec4 color;

void main() {
    color = vec4(fs_normal, 1);
}
";

static FS_RAINBOW_TEXTURE_SRC: &'static str =
"#version 400

in vec2 fs_texture;
in vec3 fs_normal;

out vec4 color;

void main() {
    color = vec4(fs_texture, 0.5, 1);
}
";

static VR_VS_SRC: &'static str =
"#version 400
in vec3 pos;
out vec2 TexPos;

void main() {
    gl_Position = vec4(pos.x, pos.y, 0.5, 1.);
    TexPos = vec2((pos.x+1)/2, (pos.y+1)/2); 
}
";

static VR_FS_SRC: &'static str = ovr::SHADER_FRAG_CHROMAB;

static VERTEX_DATA: [VertexGeoTex, ..12] = [
    // CUBE
    VertexGeoTex{position: Vec3{x: -1., y: -1., z:  1.}, texture: Vec2{x: -1., y: -1.}}, // 0
    VertexGeoTex{position: Vec3{x: -1., y:  1., z:  1.}, texture: Vec2{x: -1., y:  1.}},
    VertexGeoTex{position: Vec3{x:  1., y: -1., z:  1.}, texture: Vec2{x:  1., y: -1.}},
    VertexGeoTex{position: Vec3{x:  1., y:  1., z:  1.}, texture: Vec2{x:  1., y:  1.}},
    VertexGeoTex{position: Vec3{x: -1., y: -1., z: -1.}, texture: Vec2{x: -1., y: -1.}},
    VertexGeoTex{position: Vec3{x: -1., y:  1., z: -1.}, texture: Vec2{x: -1., y:  1.}},
    VertexGeoTex{position: Vec3{x:  1., y: -1., z: -1.}, texture: Vec2{x:  1., y: -1.}},
    VertexGeoTex{position: Vec3{x:  1., y:  1., z: -1.}, texture: Vec2{x:  1., y:  1.}},

     // BILL BOARD
    VertexGeoTex{position: Vec3{x: -1., y: -1., z:  0.}, texture: Vec2{x: -1., y: -1.}}, // 8
    VertexGeoTex{position: Vec3{x: -1., y:  1., z:  0.}, texture: Vec2{x: -1., y:  1.}}, 
    VertexGeoTex{position: Vec3{x:  1., y: -1., z:  0.}, texture: Vec2{x:  1., y: -1.}},
    VertexGeoTex{position: Vec3{x:  1., y:  1., z:  0.}, texture: Vec2{x:  1., y:  1.}},
];

static INDEX_DATA: [u32, ..42] = [
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

    // billboard
    8,  10, 9,
    10, 12, 9
];


pub fn load_default(db: &mut core::Database)
{
    let core_dir = db.add_dir(None, ~"core");
    let shader_dir = db.add_dir(Some(core_dir), ~"shaders");
    let geo_dir = db.add_dir(Some(core_dir), ~"geometry");

    let vbo = VertexBuffer::new_position_texture(VERTEX_DATA.into_owned(), INDEX_DATA.into_owned());

    db.add_shader(shader_dir, ~"rainbow_normal", Shader::new(VS_SRC.into_owned(), FS_RAINBOW_NORMAL_SRC.into_owned()));
    db.add_shader(shader_dir, ~"rainbow_texture", Shader::new(VS_SRC.into_owned(), FS_RAINBOW_TEXTURE_SRC.into_owned()));
    db.add_shader(shader_dir, ~"ovr_hmd", Shader::new(VR_VS_SRC.into_owned(), VR_FS_SRC.into_owned()));

    let vbo = db.add_vertex_buffer(geo_dir, ~"vbo", vbo);
    db.add_geometry(geo_dir, ~"cube", Geometry::triangles(vbo, 0, 36));
    db.add_geometry(geo_dir, ~"billboard", Geometry::triangles(vbo, 36, 6));
}