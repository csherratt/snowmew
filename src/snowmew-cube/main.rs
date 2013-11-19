#[feature(macro_rules)];
#[feature(globs)];

extern mod glfw;
extern mod gl;
extern mod snowmew;
extern mod cgmath;

use snowmew::core::{FrameBuffer, Object, FrameInfo, DrawTarget};
use snowmew::shader::Shader;
use snowmew::geometry::Geometry;
use cgmath::matrix::*;

static VS_SRC: &'static str =
"#version 150\n\
uniform mat4 mat; \n\
in vec3 position;\n\
out vec3 UV;\n\
void main() {\n\
    gl_Position = mat * vec4(position, 1.);\n\
    UV = vec3(position.x, position.y, position.z); \n\
}";

static FS_SRC: &'static str =
"#version 150\n\
out vec4 out_color;\n\
in vec3 UV;\n \
void main() {\n\
    gl_FragColor = vec4(UV.x, UV.y, UV.z, 0);\n\
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

static INDEX_DATA: [u16, ..36] = [
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
    std::rt::start_on_main_thread(argc, argv, main)
}

struct Cube
{
    shader: Shader,
    geometry: Geometry,
    mat: i32
}

impl Cube
{
    fn new() -> Cube
    {
        let shader = snowmew::shader::Shader::new(VS_SRC, FS_SRC, (gl::ONE, gl::ZERO));
        let geometry = snowmew::geometry::Geometry::triangles(VERTEX_DATA, INDEX_DATA);

        Cube {
            shader: shader,
            geometry: geometry,
            mat: shader.uniform(&"mat")
        }
    }
}

impl Object for Cube
{
    fn setup(&mut self, frame: &FrameInfo)
    {

    }

    fn draw(&mut self, frame: &FrameInfo, target: &mut DrawTarget)
    {
        let projection = cgmath::projection::perspective(
            cgmath::angle::deg(90f32), 16f32/9f32, 0.01f32, 10f32
        );

        let time = frame.time as f32;
        let transform = cgmath::matrix::Mat4::scale(1f32, 1f32, 1f32);
        let transform = cgmath::matrix::Mat3::from_angle_x(cgmath::angle::rad(time/2f32)).to_mat4().mul_m(&transform);
        let transform = cgmath::matrix::Mat3::from_angle_y(cgmath::angle::rad(time/8f32)).to_mat4().mul_m(&transform);
        let transform = cgmath::matrix::Mat3::from_angle_z(cgmath::angle::rad(time)).to_mat4().mul_m(&transform);
        let transform = cgmath::matrix::Mat4::translate(0f32, 0f32, -5f32).mul_m(&transform);
        let transform = projection.mul_m(&transform);

        target.draw(&self.shader,
                    &self.geometry,
                    &[(self.mat, &transform as &snowmew::coregl::Uniforms)],
                    &[]
        );
    }
}

fn main() {
    do glfw::set_error_callback |_, description| {
        print(format!("GLFW Error: {}", description));
    }

    do glfw::start {
        let screen = glfw::Monitor::get_primary().unwrap();
        let modes = screen.get_video_modes();
        let mut mode = &modes[0];
        for m in modes.iter() {
            if m.width == m.width {
                mode = m;
            }
        }
        let width = mode.width as uint;
        let height = mode.height as uint;
        println(format!("{} {}", width, height));
        glfw::window_hint::context_version(4, 0);
        glfw::window_hint::opengl_profile(glfw::OpenGlCoreProfile);
        glfw::window_hint::opengl_forward_compat(true);

        let window = glfw::Window::create(width, height, "OpenGL", glfw::Windowed).unwrap();
        window.make_context_current();
        glfw::set_swap_interval(0);

        gl::load_with(glfw::get_proc_address);

        let mut cube = Cube::new();
        
        let mut fb = snowmew::coregl::FrameBuffer {
            id: 0,
            width: width,
            height: height
        };

        let mat: Mat4<f32> = cgmath::matrix::Mat4::identity();

        let projection = cgmath::projection::perspective(
            cgmath::angle::deg(90f32), 1f32, 0.01f32, 10f32
        );

        gl::Enable(gl::SCISSOR_TEST);
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);

        let mut rot = 0f32;

        gl::ClearColor(0.3, 0.3, 0.3, 1.0);

        let mut count: uint = 0;
        let mut time = glfw::get_time();
        let mut time_last = time;

        while !window.should_close() {
            glfw::poll_events();
            time = glfw::get_time();

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let fi = FrameInfo {
                count: count,
                time: time,
                delta: time - time_last
            };

            for x in range(0u, 1u) {
                for y in range(0u, 2u) {
                    do fb.viewport((0, 0), (width, height)) |dt| {
                            cube.setup(&fi);
                            cube.draw(&fi, dt);
                    }
                }
            }

            print(format!("{}          \r", 1./(time - time_last)));
            window.swap_buffers();
            time_last = time;
        }
    }

}