use gl;

use std::ptr;
use std::vec;
use std::str;

fn compile_shader(src: &str, ty: gl::types::GLenum) -> gl::types::GLuint {
    let shader = gl::CreateShader(ty);
    unsafe {
        src.with_c_str(|ptr| {
            gl::ShaderSource(shader, 1, &ptr, ptr::null());
        });

        gl::CompileShader(shader);

        let mut status = gl::FALSE as gl::types::GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as gl::types::GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec::from_elem(len as uint, 0u8);     // subtract 1 to skip the trailing null character
            println(format!("size {}", len));
            gl::GetShaderInfoLog(shader,
                                 len,
                                 ptr::mut_null(),
                                 vec::raw::to_mut_ptr(buf) as *mut gl::types::GLchar);
            fail!(format!("glsl error: {:s} {:s}", src, str::raw::from_utf8(buf)));
        }
    }

    shader
}

pub struct Shader {
    program: gl::types::GLuint,
    fs: gl::types::GLuint,
    vs: gl::types::GLuint,
    gs: gl::types::GLuint,
    blend: (gl::types::GLenum, gl::types::GLenum)
}

impl Shader {
    pub fn new(vert: &str, frag: &str, blend: (gl::types::GLenum, gl::types::GLenum)) -> Shader
    {
        let program = gl::CreateProgram();
        let vert = compile_shader(vert, gl::VERTEX_SHADER);
        let frag = compile_shader(frag, gl::FRAGMENT_SHADER);
        gl::AttachShader(program, vert);
        gl::AttachShader(program, frag);
        gl::LinkProgram(program);
        Shader {
            program: program,
            fs: frag,
            vs: vert,
            gs: 0,
            blend: blend
        }
    }

    pub fn new_geo(vert: &str, frag: &str, geo: &str, blend: (gl::types::GLenum, gl::types::GLenum)) -> Shader
    {
        let program = gl::CreateProgram();
        let vert = compile_shader(vert, gl::VERTEX_SHADER);
        let frag = compile_shader(frag, gl::FRAGMENT_SHADER);
        let geo = compile_shader(geo, gl::GEOMETRY_SHADER);
        gl::AttachShader(program, vert);
        gl::AttachShader(program, frag);
        gl::AttachShader(program, geo);
        gl::LinkProgram(program);
        Shader {
            program: program,
            fs: frag,
            vs: vert,
            gs: geo,
            blend: blend
        }
    }

    pub fn uniform(&self, s: &str) -> i32
    {
        unsafe {
            s.with_c_str(|c_str| {
                gl::GetUniformLocation(self.program, c_str)
            })
        }
    }
}