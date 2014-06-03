use std::mem;
use std::ptr;
use std::slice::raw::mut_buf_as_slice;
use sync::Arc;

use OpenCL::hl::{CommandQueue, Context, Device, Event, EventList};
use OpenCL::mem::{Buffer, CLBuffer};
use OpenCL::CL::CL_MEM_READ_WRITE;
use cgmath::matrix::Matrix4;
use cgmath::vector::Vector4;
use gl;
use gl::types::{GLuint, GLsizeiptr};
use gl_cl;
use gl_cl::AcquireRelease;

use position::{CalcPositionsCl, MatrixManager};

use position::Positions;

use {Config, RenderData};

struct GLMatrix<'r> {
    x: &'r mut [Vector4<f32>],
    y: &'r mut [Vector4<f32>],
    z: &'r mut [Vector4<f32>],
    w: &'r mut [Vector4<f32>]
}

impl<'r> MatrixManager for GLMatrix<'r> {
    fn set(&mut self, idx: uint, mat: Matrix4<f32>) {
        assert!(idx < self.x.len());
        unsafe {
            self.x.unsafe_set(idx, mat.x);
            self.y.unsafe_set(idx, mat.y);
            self.z.unsafe_set(idx, mat.z);
            self.w.unsafe_set(idx, mat.w);
        }
    }

    fn get(&self, idx: uint) -> Matrix4<f32> {
        assert!(idx < self.x.len());
        unsafe {
            Matrix4 {
                x: *self.x.unsafe_ref(idx),
                y: *self.y.unsafe_ref(idx),
                z: *self.z.unsafe_ref(idx),
                w: *self.w.unsafe_ref(idx)
            }
        }
    }
}

pub struct MatrixBuffer {
    model_matrix: [GLuint, ..4],
    text_model_matrix: [GLuint, ..4],
    ptr_model_matrix: [*mut Vector4<f32>, ..4],
    size: uint,

    event: Option<Event>,
    cl: Option<(CalcPositionsCl, Arc<CommandQueue>, [CLBuffer<Vector4<f32>>, ..4])>,
}

impl MatrixBuffer {
    pub fn new(cfg: &Config,
               cl: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) -> MatrixBuffer {
        let buffer = &mut [0, 0, 0, 0];
        let texture = &mut [0, 0, 0, 0];

        unsafe {
            gl::GenBuffers(buffer.len() as i32, buffer.unsafe_mut_ref(0));
            gl::GenTextures(buffer.len() as i32, texture.unsafe_mut_ref(0));
      
            for i in range(0u, 4) {
                gl::BindBuffer(gl::TEXTURE_BUFFER, buffer[i]);
                gl::BindTexture(gl::TEXTURE_BUFFER, texture[i]);
                gl::TexBuffer(gl::TEXTURE_BUFFER, gl::RGBA32F, buffer[i]);
                gl::BufferData(gl::TEXTURE_BUFFER,
                               (mem::size_of::<Vector4<f32>>()*cfg.max_size()) as GLsizeiptr,
                               ptr::null(), gl::DYNAMIC_DRAW);
            }
        }

        let clpos = match cl {
            Some((ctx, cq, dev)) => {
                let calc = CalcPositionsCl::new(ctx.deref(), dev.deref());
                let buffers = [gl_cl::create_from_gl_buffer(ctx.deref(), buffer[0], CL_MEM_READ_WRITE),
                               gl_cl::create_from_gl_buffer(ctx.deref(), buffer[1], CL_MEM_READ_WRITE),
                               gl_cl::create_from_gl_buffer(ctx.deref(), buffer[2], CL_MEM_READ_WRITE),
                               gl_cl::create_from_gl_buffer(ctx.deref(), buffer[3], CL_MEM_READ_WRITE)];

                Some((calc, cq, buffers))
            },
            None => None
        };

        MatrixBuffer {
            model_matrix: [buffer[0], buffer[1], buffer[2], buffer[3]],
            text_model_matrix: [texture[0], texture[1], texture[2], texture[3]],
            ptr_model_matrix: [ptr::mut_null(), ptr::mut_null(), ptr::mut_null(), ptr::mut_null()],
            size: cfg.max_size(),
            cl: clpos,
            event: None,
        }
    }

    pub fn map(&mut self) {
        match self.cl {
            None => {
                for i in range(0u, 4) {
                    gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_matrix[i]);
                    self.ptr_model_matrix[i] = gl::MapBufferRange(gl::TEXTURE_BUFFER, 0, 
                            (mem::size_of::<Vector4<f32>>()*self.size) as GLsizeiptr,
                            gl::MAP_WRITE_BIT | gl::MAP_READ_BIT
                    ) as *mut Vector4<f32>;
                    assert!(0 == gl::GetError());
                }                
            }
            Some((_, ref cq, ref buf)) => {
                cq.acquire_gl_objects(buf.as_slice(), ()).wait()
            }
        }
    }

    pub fn unmap(&mut self) {
        let event = self.event.take();
        match (&self.cl, event) {
            (&None, None) => {
                for i in range(0u, 4) {
                    gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_matrix[i]);
                    gl::UnmapBuffer(gl::TEXTURE_BUFFER);
                    assert!(0 == gl::GetError());
                    self.ptr_model_matrix[i] = ptr::mut_null();
                }
            }
            (&Some((_, ref cq, ref buf)), Some(ref event)) => {
                cq.release_gl_objects(buf.as_slice(), event).wait();
            }
            _ => fail!("expected both an event and a queue")
        }
    }

    pub fn build<RD: RenderData>(&mut self, db: &RD) {
        self.event = unsafe {
            match self.cl {
                None => {
                    mut_buf_as_slice(self.ptr_model_matrix[0], self.size, |mat0| {
                    mut_buf_as_slice(self.ptr_model_matrix[1], self.size, |mat1| {
                    mut_buf_as_slice(self.ptr_model_matrix[2], self.size, |mat2| {
                    mut_buf_as_slice(self.ptr_model_matrix[3], self.size, |mat3| {
                        let mut mat = GLMatrix {
                            x: mat0, y: mat1, z: mat2, w: mat3
                        };
                        db.write_positions(&mut mat);
                        None
                    })})})})               
                }
                Some((ref mut ctx, ref cq, ref buf)) => {
                    let evt = db.write_positions_cl(cq.deref(), ctx, buf);
                    Some(evt)
                }
            }
        };
    }

    pub fn ids<'a>(&'a self) -> &'a [GLuint] {
        self.text_model_matrix.as_slice()
    }
}