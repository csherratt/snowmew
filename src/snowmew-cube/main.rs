#[crate_id = "snowmew-cube"];

#[feature(macro_rules)];
#[feature(globs)];

extern mod glfw;
extern mod gl;
extern mod snowmew;
extern mod render = "snowmew-render";
extern mod cgmath;
extern mod native;
extern mod extra;

use snowmew::core::Database;
use snowmew::shader::Shader;

use render::RenderManager;

use cgmath::quaternion::*;
use cgmath::transform::*;
use cgmath::vector::*;
use cgmath::matrix::*;
use cgmath::angle::{ToRad, deg};

use extra::time::precise_time_s;


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
            time = glfw::get_time();p

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
        let width = 1920 as uint;
        let height = 1080 as uint;
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

        let size = 25;

        for y in range(-size, size) { for x in range(-size, size) {for z in range(-size, size) {
            let cube_id = db.new_object(Some(scene), format!("cube_{}_{}_{}", x, y, z));
            let x = (x*5) as f32;
            let y = (y*5) as f32;
            let z = (z*5) as f32;
            db.update_location(cube_id,
                Transform3D::new(0.5f32, Quat::from_euler(deg(15f32).to_rad(), deg(0f32).to_rad(), deg(0f32).to_rad()), Vec3::new(y, x, z)));
            db.set_draw(cube_id, geometry, shader);
        }}}

        db.update_location(camera,
            Transform3D::new(1f32,
                             Quat::from_euler(deg(0f32).to_rad(), deg(0f32).to_rad(), deg(0f32).to_rad()),
                             Vec3::new(0f32, 0f32, 0f32)));


        let mut ren = RenderManager::new(db.clone());
        ren.load();

        window.set_cursor_mode(glfw::CursorDisabled);

        glfw::poll_events();
        let (wx, wy) = window.get_size();
        let (wx, wy) = (wx as f64, wy as f64);
        window.set_cursor_pos(wx/2., wy/2.);

        let (mut rot_x, mut rot_y) = (0_f64, 0_f64);

        let mut pos = Vec3::new(0f32, 0f32, 0f32);

        while !window.should_close() {
            glfw::poll_events();
            let start = precise_time_s();

            let (x, y) = window.get_cursor_pos();
            let (wx, wy) = window.get_size();
            let (wx, wy) = (wx as f64, wy as f64);
            window.set_cursor_pos(wx/2., wy/2.);

            rot_x += (x - wx/2.) / 3.;
            rot_y += (y - wy/2.) / 3.;

            rot_y = rot_y.max(&-90.).min(&90.);

            let input_vec = Vec4::new(
                if window.get_key(glfw::KeyA) == glfw::Press {0.5f32} else {0f32} +
                if window.get_key(glfw::KeyD) == glfw::Press {-0.5f32} else {0f32}, 
                0f32,
                if window.get_key(glfw::KeyW) == glfw::Press {0.5f32} else {0f32} +
                if window.get_key(glfw::KeyS) == glfw::Press {-0.5f32} else {0f32},
                1f32
            );

            let rot =  Quat::from_axis_angle(&Vec3::new(0f32, 1f32, 0f32), deg(-rot_x as f32).to_rad()).mul_q(
                      &Quat::from_axis_angle(&Vec3::new(1f32, 0f32, 0f32), deg(-rot_y as f32).to_rad()));

            let trans = Transform3D::new(0f32,
                                 rot.normalize(),
                                 Vec3::new(0f32, 0f32, 0f32));
            let pos_v = trans.rotate().to_mat4().mul_v(&input_vec);
            let new_pos = Vec3::new(pos_v.x, pos_v.y, pos_v.z);
            pos = pos.add_v(&new_pos);


            let rot =  Quat::from_axis_angle(&Vec3::new(1f32, 0f32, 0f32), deg(rot_y as f32).to_rad()).mul_q(
                      &Quat::from_axis_angle(&Vec3::new(0f32, 1f32, 0f32), deg(rot_x as f32).to_rad())).normalize();


            let trans = Transform3D::new(0f32,
                                 rot.normalize(),
                                 pos);

            db.update_location(camera, trans);

            ren.update(db.clone());


            gl::ClearColor(0.05, 0.05, 0.05, 1.);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            ren.render(scene, camera);
            window.swap_buffers();
            let end = precise_time_s();

            let time = (end-start);

            print!("\rfps: {:0.2f} time: {:0.3f}ms, budget: {:0.2f}                 ",
                1./time, time*1000., time/(1./60.));
        }
    }
}