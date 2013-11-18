extern mod glfw;
extern mod gl;
extern mod snowmew;

static VS_SRC: &'static str =
"#version 150\n\
in vec3 position;\n\
out vec2 UV;\n\
void main() {\n\
    gl_Position = vec4(position, 1.);\n\
    UV = vec2(position.x, position.y); \n\
}";

static FS_SRC: &'static str =
"#version 150\n\
out vec4 out_color;\n\
in vec2 UV;\n \
void main() {\n\
    gl_FragColor = vec4(UV.x, UV.y, 1, 0);\n\
}";

static VERTEX_DATA: [f32, ..12] = [
     -1.,  -1., 0.,
     -1.,   1., 0.,
      1.,   1., 0.,
      1.,  -1., 0.
];

static INDEX_DATA: [u16, ..6] = [
    0, 1, 2,
    2, 3, 0
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
        glfw::window_hint::context_version(3, 2);
        glfw::window_hint::opengl_profile(glfw::OpenGlCoreProfile);
        glfw::window_hint::opengl_forward_compat(true);

        let window = glfw::Window::create(1024, 1024, "OpenGL", glfw::Windowed).unwrap();
        window.make_context_current();

        gl::load_with(glfw::get_proc_address);

        let geo = snowmew::geometry::Geometry::triangles(VERTEX_DATA, INDEX_DATA);
        let shader = snowmew::shader::Shader::new(VS_SRC, FS_SRC, (gl::ONE, gl::ZERO));
        let fb = snowmew::coregl::FrameBuffer {
            id: 0,
            width: 1024,
            height: 1024
        };

        gl::Viewport(0, 0, 1024, 1024);

        while !window.should_close() {
            glfw::poll_events();
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            shader.bind();
            geo.draw();
            window.swap_buffers();
        }
    }

}