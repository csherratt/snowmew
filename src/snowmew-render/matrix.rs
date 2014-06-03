use std::mem;
use std::ptr;
use std::slice::raw::mut_buf_as_slice;
use sync::Arc;

use OpenCL::hl::{CommandQueue, Context, Device, Event, EventList};
use OpenCL::mem::{Buffer, CLBuffer};
use OpenCL::CL::CL_MEM_READ_WRITE;
use cgmath::matrix::Matrix4;
use gl;
use gl::types::{GLuint, GLsizeiptr};
use gl_cl;
use gl_cl::AcquireRelease;

use position::{CalcPositionsCl, MatrixManager};

use position::Positions;

use {Config, RenderData};

struct GLMatrix<'r> {
    mat: &'r mut [Matrix4<f32>],
}

impl<'r> MatrixManager for GLMatrix<'r> {
    fn set(&mut self, idx: uint, mat: Matrix4<f32>) {
        assert!(idx < self.mat.len());
        unsafe { self.mat.unsafe_set(idx, mat); }
    }

    fn get(&self, idx: uint) -> Matrix4<f32> {
        assert!(idx < self.mat.len());
        unsafe { self.mat.unsafe_ref(idx).clone() }
    }
}

pub struct MatrixBuffer {
    model_matrix: GLuint,
    ptr_model_matrix: *mut Matrix4<f32>,
    size: uint,

    event: Option<Event>,
    cl: Option<(CalcPositionsCl, Arc<CommandQueue>, [CLBuffer<Matrix4<f32>>, ..1])>,
}

impl MatrixBuffer {
    pub fn new(cfg: &Config,
               cl: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) -> MatrixBuffer {
        let buffer = &mut [0];

        unsafe {
            gl::GenBuffers(buffer.len() as i32, buffer.unsafe_mut_ref(0));
      
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, buffer[0]);
            gl::BufferData(gl::SHADER_STORAGE_BUFFER,
                           (mem::size_of::<Matrix4<f32>>()*cfg.max_size()) as GLsizeiptr,
                           ptr::null(), gl::DYNAMIC_DRAW);
        }

        let clpos = match cl {
            Some((ctx, cq, dev)) => {
                let calc = CalcPositionsCl::new(ctx.deref(), dev.deref());
                let buffers = gl_cl::create_from_gl_buffer(ctx.deref(), buffer[0], CL_MEM_READ_WRITE);

                Some((calc, cq, [buffers]))
            },
            None => None
        };

        MatrixBuffer {
            model_matrix: buffer[0],
            ptr_model_matrix: ptr::mut_null(),
            size: cfg.max_size(),
            cl: clpos,
            event: None,
        }
    }

    pub fn map(&mut self) {
        match self.cl {
            None => {
                gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.model_matrix);
                self.ptr_model_matrix = gl::MapBufferRange(gl::SHADER_STORAGE_BUFFER, 0, 
                        (mem::size_of::<Matrix4<f32>>()*self.size) as GLsizeiptr,
                        gl::MAP_WRITE_BIT | gl::MAP_READ_BIT
                ) as *mut Matrix4<f32>;
                assert!(0 == gl::GetError());           
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
                gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.model_matrix);
                gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
                assert!(0 == gl::GetError());
                self.ptr_model_matrix = ptr::mut_null();
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
                    mut_buf_as_slice(self.ptr_model_matrix, self.size, |mat| {
                        let mut mat = GLMatrix {
                            mat: mat
                        };
                        db.write_positions(&mut mat);
                        None
                    })               
                }
                Some((ref mut ctx, ref cq, ref buf)) => {
                    let evt = db.write_positions_cl_mat4(cq.deref(), ctx, buf.as_slice());
                    Some(evt)
                }
            }
        };
    }

    pub fn id(&self) -> GLuint {
        self.model_matrix
    }
}