use std::ptr;
use std::vec;
use std::str;
use std::cast;

use gl;
use gl::types::GLuint;

use cgmath::matrix::Mat4;
use cgmath::ptr::Ptr;

use snowmew::material::{Material, Flat};

pub fn compile_shader(src: &str, ty: gl::types::GLenum) -> GLuint {
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
            gl::GetShaderInfoLog(shader,
                                 len,
                                 ptr::mut_null(),
                                 cast::transmute(buf.unsafe_mut_ref(0)));
            fail!("glsl error: {:s} {:s}", src, str::raw::from_utf8(buf.as_slice()));
        }
    }

    shader
}

fn uniform(program: GLuint, s: &str) -> i32
{
    unsafe {
        s.with_c_str(|c_str| {
            gl::GetUniformLocation(program, c_str)
        })
    }
}

#[deriving(Clone, Default)]
pub struct Shader {
    program: GLuint,
    fs: GLuint,
    vs: GLuint,
    gs: GLuint,

    uniform_position: i32,
    uniform_projection: i32
}

impl Shader {
    fn _new(vs: GLuint, gs: GLuint, fs: GLuint) -> Shader
    {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
 
        "color".with_c_str(|ptr| {unsafe {gl::BindFragDataLocation(program, 0, ptr)}});

        "in_position".with_c_str(|ptr|{ unsafe {gl::BindAttribLocation(program, 0, ptr)}});
        "in_texture".with_c_str(|ptr|{ unsafe {gl::BindAttribLocation(program, 1, ptr)}});
        "in_normal".with_c_str(|ptr|{ unsafe {gl::BindAttribLocation(program, 2, ptr)}});

        gl::LinkProgram(program);

        unsafe {
            let mut status = gl::FALSE as gl::types::GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as gl::types::GLint) {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = vec::from_elem(len as uint, 0u8);     // subtract 1 to skip the trailing null character
                gl::GetProgramInfoLog(program,
                                      len,
                                      ptr::mut_null(),
                                      cast::transmute(buf.unsafe_mut_ref(0)));
                fail!("glsl error: {:s}", str::raw::from_utf8(buf.as_slice()));
            }
        }

        let pos = uniform(program, "mat_model");
        let proj = uniform(program, "mat_proj_view");




        Shader {
            program: program,
            fs: fs,
            vs: vs,
            gs: gs,
            uniform_position: pos,
            uniform_projection: proj
        }
    }

    pub fn new(vert: &str, frag: &str) -> Shader
    {
        let vert = compile_shader(vert, gl::VERTEX_SHADER);
        let frag = compile_shader(frag, gl::FRAGMENT_SHADER);
        Shader::_new(vert, 0, frag)
    }

    pub fn new_geo(vert: &str, frag: &str, geo: &str) -> Shader
    {
        let vert = compile_shader(vert, gl::VERTEX_SHADER);
        let frag = compile_shader(frag, gl::FRAGMENT_SHADER);
        let geo = compile_shader(geo, gl::GEOMETRY_SHADER);
        Shader::_new(vert, geo, frag)
    }

    pub fn uniform(&self, s: &str) -> i32
    {
        uniform(self.program, s)
    }

    pub fn bind(&self)
    {
        gl::UseProgram(self.program);
    }

    pub fn set_projection(&self, mat: &Mat4<f32>)
    {
        unsafe {
            gl::UniformMatrix4fv(self.uniform_projection, 1, gl::FALSE, mat.ptr());
        }
    }

    pub fn set_model(&self, mat: &Mat4<f32>)
    {
        unsafe {
            gl::UniformMatrix4fv(self.uniform_position, 1, gl::FALSE, mat.ptr());
        }        
    }

    pub fn set_material(&self, m: &Material)
    {
        match *m {
            Flat(ref color) => {
                let id = self.uniform("ambient");
                gl::Uniform3f(id, color.x, color.y, color.z);
            }
            _ => (),
        }
    }
}