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

use position::ComputedPositionGL;
use shader::compile_shader;

use time::precise_time_ns;

static position_shader: &'static str = "
#version 430

struct transform {
    vec4 pos_scale;
    vec4 rot;
    int parent;
};

mat4 transform_to_mat4(vec4 rot, float scale, vec3 pos)
{
    float x = rot.y;
    float y = rot.z;
    float z = rot.w;
    float s = rot.x;

    float x2 = x + x;
    float y2 = y + y;
    float z2 = z + z;

    float xx2 = x2 * x;
    float xy2 = x2 * y;
    float xz2 = x2 * z;

    float yy2 = y2 * y;
    float yz2 = y2 * z;
    float zz2 = z2 * z;

    float sy2 = y2 * s;
    float sz2 = z2 * s;
    float sx2 = x2 * s;

    return mat4(
        (1. - yy2 - zz2) * scale,   (xy2 + sz2) * scale,        (xz2 - sy2) * scale,        0.,
        (xy2 - sz2) * scale,        (1. - xx2 - zz2) * scale,   (yz2 + sx2) * scale,        0.,
        (xy2 + sy2) * scale,        (yz2 - sx2) * scale,        (1. - xx2 - yy2) * scale,   0.,
        pos.x,                      pos.y,                      pos.z,                      1.
    );
}

layout (std430, binding=0) buffer Transforms
{
    transform transforms[];
};

layout (std430, binding=1) buffer Matries
{
    mat4 matrices[];
};

layout(location=0) uniform int offset_last;
layout(location=1) uniform int offset_this;
layout(location=2) uniform int len;

layout(local_size_x = 1, local_size_y = 1) in;

void main()
{
    uint id = gl_WorkGroupID.x + gl_WorkGroupID.y * 1024;

    if (id < len) {
        if (offset_this == 0) {
            matrices[id] = mat4(1.0);
        } else {
            mat4 parent = matrices[offset_last+transforms[id+offset_this].parent];
            mat4 current = transform_to_mat4(
                transforms[id+offset_this].rot,
                transforms[id+offset_this].pos_scale.x,
                transforms[id+offset_this].pos_scale.yzw
            );

            matrices[offset_this+id] = parent * current;
        }
    }
}
";

pub struct PositionGlAccelerator {
    program: GLuint,
    shader: GLuint
}

impl PositionGlAccelerator {
    pub fn new() -> PositionGlAccelerator {
        let program = gl::CreateProgram();
        let shader = compile_shader(None, position_shader, gl::COMPUTE_SHADER);
        gl::AttachShader(program, shader);
        gl::LinkProgram(program);

        unsafe {
            let mut status = gl::FALSE as gl::types::GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as gl::types::GLint) {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::from_elem(len as uint, 0u8);     // subtract 1 to skip the trailing null character
                gl::GetProgramInfoLog(program,
                                      len,
                                      ptr::mut_null(),
                                      mem::transmute(buf.as_mut_slice().unsafe_mut_ref(0)));
                fail!("glsl error: {:s}", str::raw::from_utf8(buf.as_slice()));
            }
        }

        PositionGlAccelerator {
            program: program,
            shader: shader
        }
    }

    pub fn calc(&self, pos_gl: &ComputedPositionGL, delta: GLuint, pos: GLuint) {
        let start = precise_time_ns();

        gl::UseProgram(self.program);

        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, delta);
        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, pos);
    
        gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);

        let mut last_off = 0;
        for &(off, len) in pos_gl.gen.iter() {
            gl::Uniform1i(0, last_off as i32);
            gl::Uniform1i(1, off as i32);
            gl::Uniform1i(2, len as i32);
            if len > 1024 {
                gl::DispatchCompute(1024, len / 1024 + 1, 1);
            } else {
                gl::DispatchCompute(len, 1, 1);
            }
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
            last_off = off;
        }

        let end = precise_time_ns();
        println!("PositionGlAccelerator {}", end - start);
    }
}