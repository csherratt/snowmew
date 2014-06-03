
use std::ptr;
use std::mem;
use std::slice::raw::mut_buf_as_slice;

use cow::join::join_maps;

use gl;
use gl::types::{GLsizeiptr, GLuint};

use collision::sphere::Sphere;

use Config;
use RenderData;

struct ModelInfo {
    id: u32,
    matrix: u32,
    material: u32,
    _padd: u32,
    sphere: Sphere<f32>
}

pub struct ModelInfoBuffer {
    ptr_model_info: *mut ModelInfo,
    model_info: GLuint,
    size: uint
}

impl ModelInfoBuffer {
    pub fn new(cfg: &Config) -> ModelInfoBuffer {
        let buffer = &mut [0];

        unsafe {
            gl::GenBuffers(buffer.len() as i32, buffer.unsafe_mut_ref(0));
        }

        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, buffer[0]);
        unsafe {
            gl::BufferData(gl::SHADER_STORAGE_BUFFER,
                           (mem::size_of::<ModelInfo>()*cfg.max_size()) as GLsizeiptr,
                           ptr::null(), gl::DYNAMIC_DRAW);
        }
        assert!(0 == gl::GetError());

        ModelInfoBuffer {
            ptr_model_info: ptr::mut_null(),
            model_info: buffer[0],
            size: cfg.max_size()
        }
    }

    pub fn map(&mut self) {
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.model_info);
        self.ptr_model_info = gl::MapBufferRange(gl::SHADER_STORAGE_BUFFER, 0, 
                (mem::size_of::<ModelInfo>()*self.size) as GLsizeiptr,
                gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
        ) as *mut ModelInfo;
    }

    pub fn unmap(&mut self) {
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.model_info);
        gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
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
                        sphere: db.sphere(draw.geometry),
                        _padd: 0
                    };
                }
            });
        }
    }

    pub fn id(&self) -> GLuint {self.model_info}
}