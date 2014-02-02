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
}
";

static FS_SRC: &'static str =
"#version 400
out vec4 color;
in vec3 UV;
void main() {
    color = vec4(UV.x, UV.y, UV.z, 1);
}
";


static VR_VS_SRC: &'static str =
"#version 400
in vec3 pos;
out vec2 UV;

void main() {
    gl_Position = vec4(pos.x, pos.y, 0.5, 1.);
    UV = vec2((pos.x+1)/2, (pos.y+1)/2); 
}
";

static VR_FS_SRC: &'static str =
"#version 400
uniform vec2 LensCenter;
uniform vec2 ScreenCenter;
uniform vec2 ScaleIn;
uniform vec2 ScaleOut;
uniform vec4 HmdWarpParam;
uniform vec4 ChromAbParam;
uniform sampler2D Texture0;

in vec2 UV;
out vec4 color;

void main()
{
    vec2 pos = vec2(UV.x, UV.y);
    
    vec2 theta = (pos - LensCenter) * ScaleIn;
    float rSq = theta.x * theta.x + theta.y * theta.y;
    vec2 theta1 = theta * (HmdWarpParam.x + HmdWarpParam.y * rSq +
                            HmdWarpParam.z * rSq * rSq +
                            HmdWarpParam.w * rSq * rSq * rSq);
    
    vec2 thetaBlue = theta1 * (ChromAbParam.z + ChromAbParam.w * rSq);
    vec2 tcBlue = LensCenter + ScaleOut * thetaBlue;


    if (!all(equal(clamp(tcBlue, ScreenCenter-vec2(0.25,0.5), ScreenCenter+vec2(0.25,0.5)), tcBlue))) {
        color = vec4(0.);
        return;
    }


    float blue = texture2D(Texture0, tcBlue).b;
    
    vec2  tcGreen = LensCenter + ScaleOut * theta1;
    vec4  center = texture2D(Texture0, tcGreen);
    
    vec2  thetaRed = theta1 * (ChromAbParam.x + ChromAbParam.y * rSq);
    vec2  tcRed = LensCenter + ScaleOut * thetaRed;
    float red = texture2D(Texture0, tcRed).r;
    
    color = vec4(red, center.g, blue, center.a);
}
";

static VERTEX_DATA: [f32, ..36] = [
    // CUBE
    -1., -1.,  1., // 0
    -1.,  1.,  1.,
     1., -1.,  1.,
     1.,  1.,  1.,
    -1., -1., -1.,
    -1.,  1., -1.,
     1., -1., -1.,
     1.,  1., -1.,

     // BILL BOARD
    -1., -1.,  0., // 8
    -1.,  1.,  0., 
     1., -1.,  0.,
     1.,  1.,  0.,
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

    let vbo = VertexBuffer::new(VERTEX_DATA.into_owned(), INDEX_DATA.into_owned());

    db.add_shader(shader_dir, ~"rainbow", Shader::new(VS_SRC.into_owned(), FS_SRC.into_owned()));
    db.add_shader(shader_dir, ~"ovr_hmd", Shader::new(VR_VS_SRC.into_owned(), VR_FS_SRC.into_owned()));

    let vbo = db.add_vertex_buffer(geo_dir, ~"vbo", vbo);
    db.add_geometry(geo_dir, ~"cube", Geometry::triangles(vbo, 0, 36));
    db.add_geometry(geo_dir, ~"billboard", Geometry::triangles(vbo, 36, 6));
}