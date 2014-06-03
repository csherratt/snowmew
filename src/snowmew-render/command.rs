
use std::ptr;
use std::mem;
use std::slice::raw::mut_buf_as_slice;

use libc::c_void;

use gl;
use gl::types::{GLsizei, GLuint};

use config::Config;

use graphics::Graphics;

use snowmew::common::ObjectKey;


struct DrawElementsIndirectCommand {
    count: GLuint,
    instrance_count: GLuint,
    first_index: GLuint,
    base_vertex: GLuint,
    base_instance: GLuint
}

pub struct CommandBuffer {
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
}


impl CommandBuffer {
    pub fn new(cfg: &Config) -> CommandBuffer {
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

        CommandBuffer {
            command: cb[0],
            ptr: ptr::mut_null(),
            size: cfg.max_size(),
            batches: Vec::new()
        }
    }

    pub fn map(&mut self) {
        gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, self.command);
        self.ptr = gl::MapBufferRange(gl::DRAW_INDIRECT_BUFFER, 0,
                                      (mem::size_of::<DrawElementsIndirectCommand>() *
                                      self.size) as i64,
                                      gl::MAP_WRITE_BIT) as *mut DrawElementsIndirectCommand;
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

        let mut idx = -1;
        let mut last_geo = None;
        let mut command = DrawElementsIndirectCommand {
            count: 0,
            instrance_count: 1,
            first_index: 0,
            base_vertex: 0,
            base_instance: 0
        };

        unsafe {
            self.batches.truncate(0);
            mut_buf_as_slice(self.ptr, self.size, |b| {
                for (count, (_, draw)) in db.drawable_iter().enumerate() {
                    if idx == -1 {
                        let draw_geo = db.geometry(draw.geometry).expect("geometry not found");
                        last_geo = Some(draw.geometry);
                        command = DrawElementsIndirectCommand {
                            count: draw_geo.count as GLuint,
                            instrance_count: 1,
                            first_index: draw_geo.offset as GLuint,
                            base_vertex: 0,
                            base_instance: count as GLuint
                        };

                        batch.vbo = draw_geo.vb;
                        batch.count = 1;

                        idx = 0;
                    } else if last_geo == Some(draw.geometry) {
                        command.instrance_count += 1;
                    } else {
                        let draw_geo = db.geometry(draw.geometry).expect("geometry not found");
                        last_geo = Some(draw.geometry);

                        b[idx] = command;
                        idx += 1; 

                        command = DrawElementsIndirectCommand {
                            count: draw_geo.count as GLuint,
                            instrance_count: 1,
                            first_index: draw_geo.offset as GLuint,
                            base_vertex: 0,
                            base_instance: count as GLuint
                        };

                        if batch.vbo == 0 {
                            batch.vbo = draw_geo.vb;
                            batch.offset = idx;
                            batch.count = 1;
                        } else if batch.vbo == draw_geo.vb {
                            batch.count += 1;
                        } else {
                            self.batches.push(batch.clone());
                            batch.vbo = draw_geo.vb;
                            batch.offset = idx;
                            batch.count = 1;
                        }

                    }
                }
                b[idx] = command;
            });
        }
      
        self.batches.push(batch)
    }

    pub fn batches<'a>(&'a self) -> &'a [Batch] {
        self.batches.as_slice()
    }

    pub fn id(&self) -> GLuint {
        self.command
    }
}