#[crate_id = "snowmew-cube"];

#[feature(macro_rules)];
#[feature(globs)];

extern mod glfw;
extern mod gl;
extern mod snowmew;
extern mod render = "snowmew-render";
extern mod cgmath;
extern mod native;

use std::default;

use snowmew::core::{Object, Database};
use snowmew::shader::Shader;
use snowmew::geometry::Geometry;

use render::RenderManager;

use cgmath::quaternion::*;
use cgmath::transform::*;
use cgmath::vector::*;
use cgmath::angle::{ToRad, deg};

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
out vec4 colour;
in vec3 UV;\n \
void main() {
    colour = vec4(UV.x, UV.y, UV.z, 1);
}";

static LINE_VS_SRC: &'static str =
"#version 400
uniform mat4 mat; 
in vec3 position;
out vData {
    vec3 UV;
} vertex; 

void main() {
    gl_Position = mat * vec4(position, 1.);
    vertex.UV = vec3(position.x, position.y, position.z); 
}";

static LINE_FS_SRC: &'static str =
"#version 400
in fData {
    vec3 UV;
} frag;
out vec4 colour;
void main() {
    colour = vec4(frag.UV.x * 0.65, frag.UV.y * 0.65, frag.UV.z * 0.65, 1);
}";

static LINE_GS_SRC: &'static str =
"#version 400
layout(triangles_adjacency) in;
layout(triangle_strip, max_vertices = 24) out;

in vData {
    vec3 UV;
} vertices[];

out fData {
    vec3 UV;
} frag;

bool is_front(vec3 A, vec3 B, vec3 C, vec3 camera)
{
    return dot(cross(A-B, A-C), camera) > 0;
}

void emit(vec3 vec, vec3 UV)
{
    gl_Position = vec4(vec.xy, 0, 1.);
    frag.UV = UV;
    EmitVertex();
}

void emit_line(int a, int b)
{
    vec3 A = gl_in[a].gl_Position.xyz / gl_in[a].gl_Position.w;
    vec3 B = gl_in[b].gl_Position.xyz / gl_in[b].gl_Position.w;

    vec3 V = normalize(B - A);
    vec3 N = vec3(-V.y, V.x, 0.) * 0.002;

    vec3 vec_a = A - N;
    vec3 vec_b = A + N;
    vec3 vec_c = B - N;
    vec3 vec_d = B + N;

    vec3 UV_A = vertices[a].UV.xyz;
    vec3 UV_B = vertices[b].UV.xyz;

    vec3 UV_V = normalize(UV_B - UV_A);
    vec3 UV_N = vec3(-UV_V.y, UV_V.x, 0.) * 0.002;

    vec3 uv_a = UV_A - UV_N;
    vec3 uv_b = UV_A + UV_N;
    vec3 uv_c = UV_B - UV_N;
    vec3 uv_d = UV_B + UV_N;

    emit(vec_d, uv_d);
    emit(vec_b, uv_b); 
    emit(vec_c, uv_c);
    emit(vec_a, uv_a);

    EndPrimitive();
}

void main() {
    vec3 v0 = gl_in[0].gl_Position.xyz / gl_in[0].gl_Position.w;
    vec3 v1 = gl_in[1].gl_Position.xyz / gl_in[1].gl_Position.w;
    vec3 v2 = gl_in[2].gl_Position.xyz / gl_in[2].gl_Position.w;
    vec3 v3 = gl_in[3].gl_Position.xyz / gl_in[3].gl_Position.w;
    vec3 v4 = gl_in[4].gl_Position.xyz / gl_in[4].gl_Position.w;
    vec3 v5 = gl_in[5].gl_Position.xyz / gl_in[5].gl_Position.w;

    if (is_front(v0, v2, v4, vec3(0., 0., -1.))) {
        if (!is_front(v0, v1, v2, vec3(0., 0., -1.))) {
            emit_line(0, 2);
        }
        if (!is_front(v2, v3, v4, vec3(0., 0., -1.))) {
            emit_line(2, 4);
        }
        if (!is_front(v4, v5, v0, vec3(0., 0., -1.))) {
            emit_line(4, 0);
        }
    }
}
";


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

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

/*
struct Cube
{
    shader: Shader,
    geometry: Geometry,
    lines_shader: Shader,
    mat: i32
}

impl Cube
{
    fn new() -> Cube
    {
        let shader = snowmew::shader::Shader::new(VS_SRC, FS_SRC, (gl::ONE, gl::ZERO));
        let geometry = snowmew::geometry::Geometry::triangles_adjacency(VERTEX_DATA, 
            snowmew::geometry::to_triangles_adjacency(INDEX_DATA));
        let lines_shader = snowmew::shader::Shader::new_geo(LINE_VS_SRC, LINE_FS_SRC, LINE_GS_SRC, (gl::ONE, gl::ZERO));


        Cube {
            shader: shader,
            geometry: geometry,
            lines_shader: lines_shader,
            mat: shader.uniform(&"mat")
        }
    }
}

impl Object for Cube
{
    fn draw(&self, ren: &Database, ctx: &mut Context, frame: &FrameInfo, target: &mut DrawTarget)
    {
        let projection = cgmath::projection::perspective(
            cgmath::angle::deg(60f32), 16f32/9f32, 0.01f32, 25f32
        );

        let time = frame.time as f32;
        let transform = cgmath::matrix::Mat4::scale(1f32, 1f32, 1f32);
        let transform = cgmath::matrix::Mat3::from_angle_x(cgmath::angle::rad(time/2f32)).to_mat4().mul_m(&transform);
        let transform = cgmath::matrix::Mat3::from_angle_y(cgmath::angle::rad(time/8f32)).to_mat4().mul_m(&transform);
        let transform = cgmath::matrix::Mat3::from_angle_z(cgmath::angle::rad(time)).to_mat4().mul_m(&transform);
        let transform = cgmath::matrix::Mat4::translate(0f32, 0f32, -5f32).mul_m(&transform);
        let transform = projection.mul_m(&transform);


        target.draw(ctx,
                    &self.lines_shader,
                    &self.geometry,
                    &[(self.mat, &transform as &snowmew::coregl::Uniforms)],
                    &[]
        );

        target.draw(ctx,
                    &self.shader,
                    &self.geometry,
                    &[(self.mat, &transform as &snowmew::coregl::Uniforms)],
                    &[]
        );
    }
}

fn main() {
    do glfw::start {
        let width = 2560 as uint;
        let height = 1440 as uint;
        println(format!("{} {}", width, height));
        glfw::window_hint::context_version(4, 0);
        glfw::window_hint::opengl_profile(glfw::OpenGlCoreProfile);
        glfw::window_hint::opengl_forward_compat(true);

        let window = glfw::Window::create(width as u32, height as u32, "OpenGL", glfw::Windowed).unwrap();
        window.make_context_current();
        glfw::set_swap_interval(0);

        gl::load_with(glfw::get_proc_address);

        let mut cube = Cube::new();
        
        let mut fb = snowmew::coregl::FrameBuffer {
            id: 0,
            width: width,
            height: height
        };

        gl::Enable(gl::SCISSOR_TEST);
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::LINE_SMOOTH);
        gl::Enable(gl::BLEND);
        gl::CullFace(gl::BACK);


        gl::ClearColor(0.05, 0.05, 0.05, 1.);

        let mut count: uint = 0;
        let mut time = glfw::get_time();
        let mut time_last = time;

        let mut ctx = snowmew::render::Context::new();

        let mut db = snowmew::core::Database::new();

        while !window.should_close() {
            glfw::poll_events();
            time = glfw::get_time();

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let fi = FrameInfo {
                count: count,
                time: time,
                delta: time - time_last
            };

            //render.add
            fb.viewport(&mut ctx, (0, 0), (width, height), |dt, ctx| {
                cube.draw(&mut db, ctx, &fi, dt);
            });

            gl::Finish();
            let end_time = glfw::get_time();

            print(format!("Frame Budget %{:f}          \r", 100. * (end_time - time) / (1./120.)));
            window.swap_buffers();
            time_last = time;
            count += 1;
        }
    }
}
*/

fn main() {
    do glfw::start {
        let width = 1024 as uint;
        let height = 768 as uint;
        println!("{} {}", width, height);
        glfw::window_hint::context_version(4, 0);
        glfw::window_hint::opengl_profile(glfw::OpenGlCoreProfile);
        glfw::window_hint::opengl_forward_compat(true);

        let window = glfw::Window::create(width as u32, height as u32, "OpenGL", glfw::Windowed).unwrap();
        window.make_context_current();
        glfw::set_swap_interval(1);

        gl::load_with(glfw::get_proc_address);

        let mut db = Database::new();

        let shader = Shader::new(VS_SRC.into_owned(), FS_SRC.into_owned());
        let indexs = snowmew::geometry::to_triangles_adjacency(INDEX_DATA);
        let len = indexs.len();
        let vbo = snowmew::geometry::VertexBuffer::new(VERTEX_DATA.into_owned(), indexs);

        let assets = db.new_object(None, ~"asserts");
        let shader = db.add_shader(assets, ~"shader", shader);
        let vbo = db.add_vertex_buffer(assets, ~"vbo", vbo);
        let geometry = snowmew::geometry::Geometry::triangles_adjacency(vbo, 0, len);

        let geometry = db.add_geometry(assets, ~"geo", geometry);

        let camera = db.new_object(None, ~"camera");
        let scene = db.new_object(None, ~"scene");
        for y in range(-5, 5) {
            for x in range(-5, 5) {
                let cube_id = db.new_object(Some(scene), format!("cube_{}_{}", x, y));
                let x = (x*5) as f32;
                let y = (y*5) as f32;
                db.update_location(cube_id, Transform3D::new(1f32, Quat::from_euler(deg(45f32).to_rad(), deg(45f32).to_rad(), deg(45f32).to_rad()), Vec3::new(y, x, 0.)));
                db.set_draw(cube_id, geometry, shader);
            }
        }

        db.update_location(camera,
            Transform3D::new(1f32,
                             Quat::from_euler(deg(0f32).to_rad(), deg(0f32).to_rad(), deg(0f32).to_rad()),
                             Vec3::new(0f32, 0f32, -10f32)));


        let mut ren = RenderManager::new(&window, db.clone());
        ren.load();

        let mut x = 0f32;

        while !window.should_close() {
            glfw::poll_events();

            x += 0.1;

            db.update_location(camera,
                Transform3D::new(1f32,
                                 Quat::from_euler(deg(x).to_rad(), deg(x).to_rad(), deg(x).to_rad()),
                                 Vec3::new(0f32, 0f32, -10f32)));

            ren.update(db.clone());


            gl::ClearColor(0.05, 0.05, 0.05, 1.);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            ren.render(scene, camera);
            window.swap_buffers();
        }
    }
}