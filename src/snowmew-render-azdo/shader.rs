//   Copyright 2014 Colin Sherratt
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

use std::ptr;
use std::vec::Vec;
use std::str;
use std::mem;

use gl;
use gl::types::GLuint;

pub const MATRIX_PROJECTION: i32 = 0;
pub const MATRIX_MODEL: i32 = 1;

pub const ATTR_POISTION: i32 = 0;
pub const ATTR_TEXTURE: i32 = 1;
pub const ATTR_NORMAL: i32 = 2;

pub fn compile_shader(header: Option<&str>, src: &str, ty: gl::types::GLenum) -> GLuint {
    let shader = unsafe { gl::CreateShader(ty) };
    unsafe {
        match header {
            Some(header) => {
                header.with_c_str(|header_ptr| {
                src.with_c_str(|ptr| {
                    let s_ptrs = &[header_ptr, ptr];
                    gl::ShaderSource(shader, 2, &s_ptrs[0], ptr::null());
                })});
            }
            None => {
                src.with_c_str(|ptr| {
                    gl::ShaderSource(shader, 1, &ptr, ptr::null());
                });
            }
        }


        gl::CompileShader(shader);

        let mut status = gl::FALSE as gl::types::GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        let mut len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

        if len != 0 {
            let mut buf = Vec::from_elem(len as uint, 0u8);     // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(shader,
                                 len,
                                 ptr::null_mut(),
                                 mem::transmute(buf.as_mut_slice().get_unchecked_mut(0)));
            if status == gl::FALSE as i32 {
                panic!("glsl error: {} {}", src, str::from_utf8_unchecked(buf.as_slice()));
            } else {
                println!("shader log {:}", str::from_utf8_unchecked(buf.as_slice()));
            }
        }
    }

    shader
}

#[deriving(Clone, Default)]
pub struct Shader {
    program: GLuint,
    shaders: Vec<GLuint>
}

impl Shader {
    fn _new(shaders: Vec<GLuint>, bind_attr: &[(u32, &str)], bind_frag: &[(u32, &str)]) -> Shader {
        let program = unsafe {
            let program = gl::CreateProgram();
            for s in shaders.iter() {
                 gl::AttachShader(program, *s);
            }

            for &(idx, name) in bind_attr.iter() {
                name.with_c_str(|ptr| gl::BindAttribLocation(program, idx, ptr));
            }
            for &(idx, name) in bind_frag.iter() {
                name.with_c_str(|ptr| gl::BindFragDataLocation(program, idx, ptr));
            }

            gl::LinkProgram(program);


            let mut status = gl::FALSE as gl::types::GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as gl::types::GLint) {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::from_elem(len as uint, 0u8);     // subtract 1 to skip the trailing null character
                gl::GetProgramInfoLog(program,
                                      len,
                                      ptr::null_mut(),
                                      mem::transmute(buf.as_mut_slice().get_unchecked_mut(0)));
                panic!("glsl error: {}", str::from_utf8_unchecked(buf.as_slice()));
            }
            program
        };


        Shader {
            program: program,
            shaders: shaders
        }
    }

    pub fn new(vert: &str,
               frag: &str,
               bind_attr: &[(u32, &str)],
               bind_frag: &[(u32, &str)],
               header: Option<&str>) -> Shader {
        let vert = compile_shader(header, vert, gl::VERTEX_SHADER);
        let frag = compile_shader(header, frag, gl::FRAGMENT_SHADER);
        Shader::_new(vec!(vert, frag), bind_attr, bind_frag)
    }

    pub fn new_geo(vert: &str,
                   frag: &str,
                   geo: &str,
                   bind_attr: &[(u32, &str)],
                   bind_frag: &[(u32, &str)],
                   header: Option<&str>) -> Shader {
        let vert = compile_shader(header, vert, gl::VERTEX_SHADER);
        let frag = compile_shader(header, frag, gl::FRAGMENT_SHADER);
        let geo = compile_shader(header, geo, gl::GEOMETRY_SHADER);
        Shader::_new(vec!(vert, geo, frag), bind_attr, bind_frag)
    }

    pub fn compute(cs: &str, header: Option<&str>) -> Shader {
        let cs = compile_shader(header, cs, gl::COMPUTE_SHADER);
        Shader::_new(vec!(cs), &[], &[])
    }

    pub fn uniform(&self, s: &str) -> i32 {
        unsafe {
            s.with_c_str(|c_str| {
                gl::GetUniformLocation(self.program, c_str)
            })
        }
    }

    pub fn uniform_block_index(&self, s: &str) -> u32 {
        unsafe {
            s.with_c_str(|c_str| {
                gl::GetUniformBlockIndex(self.program, c_str)
            })
        }
    }

    pub fn get_program_resouce_location(&self, gltype: u32, s: &str) -> u32 {
        unsafe {
            s.with_c_str(|c_str| {
                gl::GetProgramResourceLocation(self.program, gltype, c_str) as u32
            })
        }
    }

    pub fn uniform_block_data_size(&self, idx: u32) -> i32 {
        unsafe {
            let mut val = 0;
            gl::GetActiveUniformBlockiv(self.program, idx, gl::UNIFORM_BLOCK_DATA_SIZE, &mut val);
            val
        }
    }

    pub fn uniform_block_offset(&self, idx: u32) -> i32 {
        unsafe {
            let mut val = 0;
            gl::GetActiveUniformBlockiv(self.program, idx, gl::UNIFORM_OFFSET, &mut val);
            val
        }
    }

    pub fn uniform_block_bind(&self, idx: u32, buffer: u32) {
        unsafe { gl::UniformBlockBinding(self.program, idx, buffer); }
    }

    pub fn bind(&self) {
        unsafe { gl::UseProgram(self.program); }
    }

    pub fn validate(&self)  {
        unsafe {
            gl::ValidateProgram(self.program);
            let mut status = gl::FALSE as gl::types::GLint;
            gl::GetProgramiv(self.program, gl::VALIDATE_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as gl::types::GLint) {
                let mut len = 0;
                gl::GetProgramiv(self.program, gl::INFO_LOG_LENGTH, &mut len);
                if len > 0 {
                    let mut buf = Vec::from_elem(len as uint, 0u8);     // subtract 1 to skip the trailing null character
                    gl::GetProgramInfoLog(self.program,
                                          len,
                                          ptr::null_mut(),
                                          mem::transmute(buf.as_mut_slice().get_unchecked_mut(0)));
                    panic!("glsl error: {}", str::from_utf8_unchecked(buf.as_slice()));
                }
            }
        }
    }
}