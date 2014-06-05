
use std::ptr;
use std::mem;
use std::slice::raw::mut_buf_as_slice;

use cow::join::join_maps;

use gl;
use gl::types::{GLsizeiptr, GLuint};

use collision::sphere::Sphere;

use Config;
use RenderData;

struct ModelInfoSSBO {
    id: u32,
    matrix: u32,
    material: u32,
    _padd: u32,
    sphere: Sphere<f32>
}

pub struct ModelInfoSSBOBuffer {
    ptr_model_info: *mut ModelInfoSSBO,
    model_info: GLuint,
    size: uint
}

impl ModelInfoSSBOBuffer {
    pub fn new(cfg: &Config) -> ModelInfoSSBOBuffer {
        let buffer = &mut [0];

        unsafe {
            gl::GenBuffers(buffer.len() as i32, buffer.unsafe_mut_ref(0));
        }

        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, buffer[0]);
        unsafe {
            gl::BufferData(gl::SHADER_STORAGE_BUFFER,
                           (mem::size_of::<ModelInfoSSBO>()*cfg.max_size()) as GLsizeiptr,
                           ptr::null(), gl::DYNAMIC_DRAW);
        }
        assert!(0 == gl::GetError());

        ModelInfoSSBOBuffer {
            ptr_model_info: ptr::mut_null(),
            model_info: buffer[0],
            size: cfg.max_size()
        }
    }

    pub fn map(&mut self) {
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.model_info);
        self.ptr_model_info = gl::MapBufferRange(gl::SHADER_STORAGE_BUFFER, 0, 
                (mem::size_of::<ModelInfoSSBO>()*self.size) as GLsizeiptr,
                gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
        ) as *mut ModelInfoSSBO;
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
                    info[idx] = ModelInfoSSBO {
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

struct ModelInfoTexture {
    id: u32,
    matrix: u32,
    material: u32
}

pub struct ModelInfoTextureBuffer {
    ptr_model_info: *mut ModelInfoTexture,
    model_info: GLuint,
    texture_model_info: GLuint,
    size: uint
}

impl ModelInfoTextureBuffer {
    pub fn new(cfg: &Config) -> ModelInfoTextureBuffer {
        let buffer = &mut [0];
        let texture = &mut [0];

        unsafe {
            gl::GenBuffers(buffer.len() as i32, buffer.unsafe_mut_ref(0));
            gl::GenTextures(texture.len() as i32, texture.unsafe_mut_ref(0));
        }

        gl::BindBuffer(gl::TEXTURE_BUFFER, buffer[0]);
        gl::BindTexture(gl::TEXTURE_BUFFER, texture[0]);
        gl::TexBuffer(gl::TEXTURE_BUFFER, gl::RGB32UI, buffer[0]);
        unsafe {
            gl::BufferData(gl::TEXTURE_BUFFER,
                           (mem::size_of::<ModelInfoTexture>()*cfg.max_size()) as GLsizeiptr,
                           ptr::null(), gl::DYNAMIC_DRAW);
        }
        assert!(0 == gl::GetError());

        ModelInfoTextureBuffer {
            ptr_model_info: ptr::mut_null(),
            model_info: buffer[0],
            texture_model_info: texture[0],
            size: cfg.max_size()
        }
    }

    pub fn map(&mut self) {
        gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_info);
        self.ptr_model_info = gl::MapBufferRange(gl::TEXTURE_BUFFER, 0, 
                (mem::size_of::<ModelInfoTexture>()*self.size) as GLsizeiptr,
                gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
        ) as *mut ModelInfoTexture;
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
                    info[idx] = ModelInfoTexture {
                        id: id.clone(),
                        matrix: position.get_loc(*pos) as u32,
                        material: db.material_index(draw.material).unwrap() as u32
                    };
                }
            });
        }
    }

    pub fn id(&self) -> GLuint {self.texture_model_info}
}