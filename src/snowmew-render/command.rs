
use std::ptr;
use std::mem;
use std::slice::raw::mut_buf_as_slice;

use libc::c_void;

use gl;
use gl::types::{GLsizei, GLuint};

use cgmath::matrix::Matrix4;
use cgmath::array::Array2;
use cgmath::vector::{Vector, EuclideanVector};

use config::Config;
use graphics::Graphics;
use snowmew::common::ObjectKey;

use db::GlState;

struct DrawElementsIndirectCommand {
    pub count: GLuint,
    pub instrance_count: GLuint,
    pub first_index: GLuint,
    pub base_vertex: GLuint,
    pub base_instance: GLuint
}

pub struct CommandBufferIndirect {
    command: GLuint,
    ptr: *mut DrawElementsIndirectCommand,
    size: uint,
    batches: Vec<Batch>
}

#[deriving(Clone)]
pub struct Batch {
    vbo: ObjectKey,
    offset: uint,
    count: uint
}

impl Batch {
    pub fn vbo(&self) -> ObjectKey {self.vbo}

    pub fn offset(&self) -> *c_void {
        (self.offset * mem::size_of::<DrawElementsIndirectCommand>()) as *c_void
    }

    pub fn drawcount(&self) -> GLsizei {
        self.count as GLsizei
    }

    pub fn stride(&self) -> GLsizei {
        mem::size_of::<DrawElementsIndirectCommand>() as GLsizei
    }

    pub fn offset_int(&self) -> uint {self.offset}
}


impl CommandBufferIndirect {
    pub fn new(cfg: &Config) -> CommandBufferIndirect {
        let cb = &mut [0];
        unsafe {
            gl::GenBuffers(1, cb.unsafe_mut_ref(0));
            gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, cb[0]);
            gl::BufferData(gl::DRAW_INDIRECT_BUFFER,
                           (mem::size_of::<DrawElementsIndirectCommand>() *
                           cfg.max_size()) as i64,
                           ptr::null(),
                           gl::DYNAMIC_DRAW);
            gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, 0);
        }

        CommandBufferIndirect {
            command: cb[0],
            ptr: ptr::mut_null(),
            size: cfg.max_size(),
            batches: Vec::new()
        }
    }

    pub fn map(&mut self) {
        gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, self.command);
        self.ptr = gl::MapBufferRange(
            gl::DRAW_INDIRECT_BUFFER, 0,
            (mem::size_of::<DrawElementsIndirectCommand>() *
            self.size) as i64,
            gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
        ) as *mut DrawElementsIndirectCommand;
        gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, 0);
    }

    pub fn unmap(&mut self) {
        self.ptr = ptr::mut_null();
        gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, self.command);
        gl::UnmapBuffer(gl::DRAW_INDIRECT_BUFFER);
        gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, 0);
    }

    pub fn build<GD: Graphics>(&mut self, db: &GD) {
        let mut batch = Batch {
            vbo: 0,
            offset: 0,
            count: 0
        };

        unsafe {
            self.batches.truncate(0);
            mut_buf_as_slice(self.ptr, self.size, |b| {
                for (count, (_, draw)) in db.drawable_iter().enumerate() {
                    let draw_geo = db.geometry(draw.geometry).expect("geometry not found");

                    b[count] = DrawElementsIndirectCommand {
                        count: draw_geo.count as GLuint,
                        instrance_count: 1,
                        first_index: draw_geo.offset as GLuint,
                        base_vertex: 0,
                        base_instance: count as GLuint
                    };

                    if batch.vbo == 0 {
                        batch.vbo = draw_geo.vb;
                        batch.offset = count;
                        batch.count = 1;
                    } else if batch.vbo == draw_geo.vb {
                        batch.count += 1;
                    } else {
                        self.batches.push(batch.clone());
                        batch.vbo = draw_geo.vb;
                        batch.offset = count;
                        batch.count = 1;
                    }
                }
            });
        }
      
        self.batches.push(batch)
    }

    pub fn cull(&self, draw: GLuint, matrix: GLuint, dat: &GlState, mat: &Matrix4<f32>) {
        let to_plane = |x, scale| {
            let plane = mat.r(x).mul_s(scale).add_v(&mat.r(3));
            plane.normalize()
        };

        let planes = &[
            to_plane(0,  1.),
            to_plane(0, -1.),
            to_plane(1,  1.),
            to_plane(1, -1.),
            to_plane(2,  1.),
            to_plane(2, -1.)
        ];

        let shader = dat.compute_cull.as_ref().expect("Could not get cull");

        let size = self.batches.iter().fold(0, |a, b| a + b.count);

        let x = 256;
        let y = size / 256 + 1;

        shader.bind();
        unsafe {
            gl::Uniform4fv(shader.uniform("plane"), 6, &planes[0].x);
            gl::Uniform1i(shader.uniform("max_id"), size as i32);
        }

        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, draw);
        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, matrix);
        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, self.command);
        gl::DispatchCompute(x as u32, y as u32, 1);
        gl::MemoryBarrier(gl::COMMAND_BARRIER_BIT);
    }

    pub fn batches<'a>(&'a self) -> &'a [Batch] {
        self.batches.slice(0, self.batches.len())
    }

    pub fn id(&self) -> GLuint {
        self.command
    }
}

pub struct CommandBufferEmulated {
    commands: Vec<DrawElementsIndirectCommand>,
    batches: Vec<Batch>
}

impl CommandBufferEmulated {
    pub fn new(_: &Config) -> CommandBufferEmulated {
        CommandBufferEmulated {
            commands: Vec::new(),
            batches: Vec::new()
        }
    }

    pub fn map(&mut self) {}
    pub fn unmap(&mut self) {}

    pub fn build<GD: Graphics>(&mut self, db: &GD) {
        let mut batch = Batch {
            vbo: 0,
            offset: 0,
            count: 0
        };

        self.batches.truncate(0);
        self.commands.truncate(0);
        for (count, (_, draw)) in db.drawable_iter().enumerate() {
            let draw_geo = db.geometry(draw.geometry).expect("geometry not found");

            self.commands.push(DrawElementsIndirectCommand {
                count: draw_geo.count as GLuint,
                instrance_count: 1,
                first_index: draw_geo.offset as GLuint,
                base_vertex: 0,
                base_instance: count as GLuint
            });

            if batch.vbo == 0 {
                batch.vbo = draw_geo.vb;
                batch.offset = count;
                batch.count = 1;
            } else if batch.vbo == draw_geo.vb {
                batch.count += 1;
            } else {
                self.batches.push(batch.clone());
                batch.vbo = draw_geo.vb;
                batch.offset = count;
                batch.count = 1;
            }
        }
      
        self.batches.push(batch)
    }

    pub fn batches<'a>(&'a self) -> &'a [Batch] {
        self.batches.slice(0, self.batches.len())
    }

    pub fn commands<'a>(&'a self) -> &'a [DrawElementsIndirectCommand] {
        self.commands.slice(0, self.commands.len())
    }
}