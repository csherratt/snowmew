#[feature(macro_rules)];
#[feature(globs)];

extern mod glfw;
extern mod gl;
extern mod snowmew;
extern mod cgmath;

use snowmew::core::FrameBuffer;
use cgmath::matrix::*;

static VS_SRC: &'static str =
"#version 150\n\
uniform mat4 mat; \n\
in vec3 position;\n\
out vec2 UV;\n\
void main() {\n\
    gl_Position = mat * vec4(position, 1.);\n\
    UV = vec2(position.x, position.y); \n\
}";

static FS_SRC: &'static str =
"#version 150\n\
out vec4 out_color;\n\
in vec2 UV;\n \
void main() {\n\
    gl_FragColor = vec4(UV.x, UV.y, 1, 0);\n\
}";

static VERTEX_DATA: [f32, ..24] = [
    -1., -1., -1.,
    -1.,  1., -1.,
     1., -1., -1.,
     1.,  1., -1.,
    -1., -1.,  1.,
    -1.,  1.,  1.,
     1., -1.,  1.,
     1.,  1.,  1.,
];

static INDEX_DATA: [u16, ..36] = [
    // top
    0, 1, 2,
    1, 2, 3,

    // bottom
    4, 5, 6,
    5, 6, 7,

    // right
    1, 3, 5,
    3, 5, 7,

    // left
    0, 2, 4,
    2, 4, 6,

    // front
    2, 3, 6,
    3, 6, 7,

    // back
    0, 1, 4,
    1, 4, 5
];


#[start]
fn start(argc: int, argv: **u8) -> int {
    std::rt::start_on_main_thread(argc, argv, main)
}

fn main() {
    do glfw::set_error_callback |_, description| {
        print(format!("GLFW Error: {}", description));
    }

    do glfw::start {
        glfw::window_hint::context_version(4, 0);
        glfw::window_hint::opengl_profile(glfw::OpenGlCoreProfile);
        glfw::window_hint::opengl_forward_compat(true);

        let window = glfw::Window::create(1024, 1024, "OpenGL", glfw::Windowed).unwrap();
        window.make_context_current();

        gl::load_with(glfw::get_proc_address);

        let geo = snowmew::geometry::Geometry::triangles(VERTEX_DATA, INDEX_DATA);
        let shader = snowmew::shader::Shader::new(VS_SRC, FS_SRC, (gl::ONE, gl::ZERO));
        let mut fb = snowmew::coregl::FrameBuffer {
            id: 0,
            width: 1024,
            height: 1024
        };

        let mat: Mat4<f32> = cgmath::matrix::Mat4::identity();

        let projection = cgmath::projection::perspective(
            cgmath::angle::deg(90f32), 1f32, 0.01f32, 10f32
        );

        gl::Enable(gl::SCISSOR_TEST);
        gl::Enable(gl::DEPTH_TEST);

        let mut rot = 0f32;

        while !window.should_close() {
            glfw::poll_events();

            let transform = cgmath::matrix::Mat4::scale(0.5f32, 0.5f32, 0.5f32);
            let transform = cgmath::matrix::Mat3::from_angle_x(cgmath::angle::rad(rot)).to_mat4().mul_m(&transform);
            let transform = cgmath::matrix::Mat3::from_angle_y(cgmath::angle::rad(rot)).to_mat4().mul_m(&transform);
            let transform = cgmath::matrix::Mat3::from_angle_z(cgmath::angle::rad(rot)).to_mat4().mul_m(&transform);
            let transform = cgmath::matrix::Mat4::translate(0f32, 0f32, -5f32).mul_m(&transform);
            let transform = projection.mul_m(&transform);

            do fb.viewport((0, 0), (1024, 1024)) |dt| {
                gl::ClearColor(0.3, 0.3, 0.3, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                dt.draw(&shader, &geo, &[(&"mat", &transform as &snowmew::coregl::Uniforms)], &[]);             
            }
            window.swap_buffers();
            rot += 0.01f32;
        }
    }

}