
use std::ptr;
use std::mem;
use std::slice::raw::mut_buf_as_slice;

use cow::join::join_maps;

use gl;
use gl::types::{GLsizeiptr, GLuint};

use Config;
use RenderData;

struct ModelInfo {
    id: u32,
    matrix: u32,
    material: u32,
    _padding: u32
}

pub struct ModelInfoBuffer {
    ptr_model_info: *mut ModelInfo,
    text_model_info: GLuint,
    model_info: GLuint,
    size: uint
}

impl ModelInfoBuffer {
    pub fn new(cfg: &Config) -> ModelInfoBuffer {
        let buffer = &mut [0];
        let texture = &mut [0];

        unsafe {
            gl::GenBuffers(buffer.len() as i32, buffer.unsafe_mut_ref(0));
            gl::GenTextures(buffer.len() as i32, texture.unsafe_mut_ref(0));
        }

        gl::BindBuffer(gl::TEXTURE_BUFFER, buffer[0]);
        gl::BindTexture(gl::TEXTURE_BUFFER, texture[0]);
        gl::TexBuffer(gl::TEXTURE_BUFFER, gl::RGBA32UI, buffer[0]);
        unsafe {
            gl::BufferData(gl::TEXTURE_BUFFER,
                           (mem::size_of::<ModelInfo>()*cfg.max_size()) as GLsizeiptr,
                           ptr::null(), gl::DYNAMIC_DRAW);
        }
        assert!(0 == gl::GetError());

        ModelInfoBuffer {
            ptr_model_info: ptr::mut_null(),
            text_model_info: texture[0],
            model_info: buffer[0],
            size: cfg.max_size()
        }
    }

    pub fn map(&mut self) {
        gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_info);
        self.ptr_model_info = gl::MapBufferRange(gl::TEXTURE_BUFFER, 0, 
                (mem::size_of::<ModelInfo>()*self.size) as GLsizeiptr,
                gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
        ) as *mut ModelInfo;
    }

    pub fn unmap(&mut self) {
        gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_info);
        gl::UnmapBuffer(gl::TEXTURE_BUFFER);
        self.ptr_model_info = ptr::mut_null();
    }

    pub fn build(&mut self, db: &RenderData) {
        let position = db.compute_positions();
        unsafe {
            mut_buf_as_slice(self.ptr_model_info, self.size, |info| {
                for (idx, (id, (draw, pos))) in join_maps(db.drawable_iter(), db.location_iter()).enumerate() {
                    info[idx] = ModelInfo {
                        id: id.clone(),
                        matrix: position.get_loc(*pos) as u32,
                        material: db.material_index(draw.material).unwrap() as u32,
                        _padding: 0
                    };
                }
            });
        }
    }

    pub fn id(&self) -> GLuint {self.text_model_info}
}