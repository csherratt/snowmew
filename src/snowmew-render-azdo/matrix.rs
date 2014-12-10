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

use std::mem;
use std::ptr;
use std::slice;
use std::sync::Arc;

use opencl::hl::{CommandQueue, Context, Device, Event, EventList};
use opencl::mem::{Buffer, CLBuffer};
use opencl::cl::CL_MEM_READ_WRITE;
use cgmath::Matrix4;
use cgmath::Vector4;
use gl;
use gl::types::{GLuint, GLsizeiptr};
use gl_cl;
use gl_cl::AcquireRelease;

use position::{MatrixManager, Positions};
use position::cl::Accelerator;
use render_data::Renderable;


use Config;

struct GLTextureMatrix<'r> {
    x: &'r mut [Vector4<f32>],
    y: &'r mut [Vector4<f32>],
    z: &'r mut [Vector4<f32>],
    w: &'r mut [Vector4<f32>],
}

impl<'r> MatrixManager for GLTextureMatrix<'r> {
    fn size(&mut self, size: uint) {assert!(self.x.len() > size)}

    fn set(&mut self, idx: uint, mat: Matrix4<f32>) {
        assert!(idx < self.x.len());
        unsafe { *self.x.as_mut_ptr().offset(idx as int) = mat.x; }
        unsafe { *self.y.as_mut_ptr().offset(idx as int) = mat.y; }
        unsafe { *self.z.as_mut_ptr().offset(idx as int) = mat.z; }
        unsafe { *self.w.as_mut_ptr().offset(idx as int) = mat.w; }
    }

    fn get(&self, idx: uint) -> Matrix4<f32> {
        assert!(idx < self.x.len());
        unsafe {
            Matrix4 {
                x: self.x.unsafe_get(idx).clone(),
                y: self.y.unsafe_get(idx).clone(),
                z: self.z.unsafe_get(idx).clone(),
                w: self.w.unsafe_get(idx).clone(),
            }
        }
    }
}

struct GLSSBOMatrix<'r> {
    mat: &'r mut [Matrix4<f32>],
}

impl<'r> MatrixManager for GLSSBOMatrix<'r> {
    fn size(&mut self, size: uint) {assert!(self.mat.len() > size)}

    fn set(&mut self, idx: uint, mat: Matrix4<f32>) {
        assert!(idx < self.mat.len());
        unsafe { *self.mat.as_mut_ptr().offset(idx as int) = mat; }
    }

    fn get(&self, idx: uint) -> Matrix4<f32> {
        assert!(idx < self.mat.len());
        unsafe { self.mat.unsafe_get(idx).clone() }
    }
}

pub struct MatrixSSBOBuffer {
    model_matrix: GLuint,
    ptr_model_matrix: *mut Matrix4<f32>,
    size: uint,

    event: Option<Event>,
    cl: Option<(Accelerator, Arc<CommandQueue>, [CLBuffer<Matrix4<f32>>, ..1])>,
}

impl MatrixSSBOBuffer {
    pub fn new(cfg: &Config,
               cl: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) -> MatrixSSBOBuffer {
        let buffer = &mut [0];

        unsafe {
            gl::GenBuffers(buffer.len() as i32, buffer.unsafe_mut(0));

            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, buffer[0]);
            gl::BufferData(gl::SHADER_STORAGE_BUFFER,
                           (mem::size_of::<Matrix4<f32>>()*cfg.max_size()) as GLsizeiptr,
                           ptr::null(), gl::DYNAMIC_DRAW);
        }

        let clpos = match cl {
            Some((ctx, cq, dev)) => {
                let calc = Accelerator::new(ctx.deref(), dev.deref());
                let buffers = gl_cl::create_from_gl_buffer(ctx.deref(), buffer[0], CL_MEM_READ_WRITE);

                Some((calc, cq, [buffers]))
            },
            None => None
        };

        MatrixSSBOBuffer {
            model_matrix: buffer[0],
            ptr_model_matrix: ptr::null_mut(),
            size: cfg.max_size(),
            cl: clpos,
            event: None,
        }
    }

    pub fn map(&mut self) {
        match self.cl {
            None => unsafe {
                gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.model_matrix);
                self.ptr_model_matrix = gl::MapBufferRange(gl::SHADER_STORAGE_BUFFER, 0,
                        (mem::size_of::<Matrix4<f32>>()*self.size) as GLsizeiptr,
                        gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
                ) as *mut Matrix4<f32>;
            },
            Some((_, ref cq, ref buf)) => {
                cq.acquire_gl_objects(buf.as_slice(), ()).wait()
            }
        }
    }

    pub fn unmap(&mut self) {
        let event = self.event.take();
        match (&self.cl, event) {
            (&None, None) => unsafe {
                gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.model_matrix);
                gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
                assert!(0 == gl::GetError());
                self.ptr_model_matrix = ptr::null_mut();
            },
            (&Some((_, ref cq, ref buf)), Some(ref event)) => {
                cq.release_gl_objects(buf.as_slice(), event).wait();
            }
            _ => panic!("expected both an event and a queue")
        }
    }

    pub fn build<RD: Renderable>(&mut self, db: &RD) {
        self.event = unsafe {
            match self.cl {
                None => {
                    let mat = slice::from_raw_mut_buf(&self.ptr_model_matrix, self.size);
                    let mut mat = GLSSBOMatrix {
                        mat: mat
                    };
                    db.write_positions(&mut mat);
                    None
                }
                Some((ref mut ctx, ref cq, ref buf)) => {
                    let evt = ctx.compute_mat(db, cq.deref(), &buf[0]);
                    Some(evt)
                }
            }
        };
    }

    pub fn id(&self) -> GLuint { self.model_matrix }
}

pub struct MatrixTextureBuffer {
    model_matrix: [GLuint, ..4],
    texture_model_matrix: [GLuint, ..4],
    ptr_model_matrix: [*mut Vector4<f32>, ..4],
    size: uint,

    event: Option<Event>,
    cl: Option<(Accelerator, Arc<CommandQueue>, [CLBuffer<Vector4<f32>>, ..4])>,
}

impl MatrixTextureBuffer {
    pub fn new(cfg: &Config,
               cl: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) -> MatrixTextureBuffer {
        let buffer = &mut [0, 0, 0, 0];
        let texture = &mut [0, 0, 0, 0];

        unsafe {
            gl::GenBuffers(buffer.len() as i32, buffer.unsafe_mut(0));
            gl::GenTextures(texture.len() as i32, texture.unsafe_mut(0));

            for (b, t) in buffer.iter().zip(texture.iter()) {
                gl::BindBuffer(gl::TEXTURE_BUFFER, *b);
                gl::BindTexture(gl::TEXTURE_BUFFER, *t);
                gl::TexBuffer(gl::TEXTURE_BUFFER, gl::RGBA32F, *b);
                gl::BufferData(gl::TEXTURE_BUFFER,
                               (mem::size_of::<Vector4<f32>>()*cfg.max_size()) as GLsizeiptr,
                               ptr::null(), gl::DYNAMIC_DRAW);
            }
        }

        let clpos = match cl {
            Some((ctx, cq, dev)) => {
                let calc = Accelerator::new(ctx.deref(), dev.deref());
                let buffers = [gl_cl::create_from_gl_buffer(ctx.deref(), buffer[0], CL_MEM_READ_WRITE),
                               gl_cl::create_from_gl_buffer(ctx.deref(), buffer[1], CL_MEM_READ_WRITE),
                               gl_cl::create_from_gl_buffer(ctx.deref(), buffer[2], CL_MEM_READ_WRITE),
                               gl_cl::create_from_gl_buffer(ctx.deref(), buffer[3], CL_MEM_READ_WRITE)];

                Some((calc, cq, buffers))
            },
            None => None
        };

        MatrixTextureBuffer {
            model_matrix: [buffer[0], buffer[1], buffer[2], buffer[3]],
            texture_model_matrix: [texture[0], texture[1], texture[2], texture[3]],
            ptr_model_matrix: [ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), ptr::null_mut()],
            size: cfg.max_size(),
            cl: clpos,
            event: None,
        }
    }

    pub fn map(&mut self) {
        match self.cl {
            None => {
                for i in range(0u, 4) { unsafe {
                    gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_matrix[i]);
                    self.ptr_model_matrix[i] = gl::MapBufferRange(
                        gl::TEXTURE_BUFFER, 0,
                        (mem::size_of::<Vector4<f32>>()*self.size) as GLsizeiptr,
                        gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
                    ) as *mut Vector4<f32>;
                }}
            }
            Some((_, ref cq, ref buf)) => {
                cq.acquire_gl_objects(buf.as_slice(), ()).wait()
            }
        }
    }

    pub fn unmap(&mut self) {
        let event = self.event.take();
        match (&self.cl, event) {
            (&None, None) => unsafe {
                for i in range(0u, 4) {
                    gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_matrix[i]);
                    gl::UnmapBuffer(gl::TEXTURE_BUFFER);
                    assert!(0 == gl::GetError());
                    self.ptr_model_matrix[i] = ptr::null_mut();
                }
            },
            (&Some((_, ref cq, ref buf)), Some(ref event)) => {
                cq.release_gl_objects(buf.as_slice(), event).wait();
            }
            _ => panic!("expected both an event and a queue")
        }
    }

    pub fn build<RD: Renderable>(&mut self, db: &RD) {
        self.event = unsafe {
            match self.cl {
                None => {
                    let mut mat = GLTextureMatrix {
                        x: slice::from_raw_mut_buf(&self.ptr_model_matrix[0], self.size),
                        y: slice::from_raw_mut_buf(&self.ptr_model_matrix[1], self.size),
                        z: slice::from_raw_mut_buf(&self.ptr_model_matrix[2], self.size),
                        w: slice::from_raw_mut_buf(&self.ptr_model_matrix[3], self.size)
                    };
                    db.write_positions(&mut mat);
                    None
                }
                Some((ref mut ctx, ref cq, ref buf)) => {
                    let evt = ctx.compute_vec4x4(db, cq.deref(), buf);
                    Some(evt)
                }
            }
        };
    }

    pub fn ids<'a>(&'a self) -> &'a [GLuint] { self.texture_model_matrix.as_slice() }
}