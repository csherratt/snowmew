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
use std::mem;
use std::slice::raw::mut_buf_as_slice;
use render_data::Renderable;

use cow::join::{join_set_to_map, join_maps};

use gl;
use gl::types::{GLsizeiptr, GLuint};

use collision::sphere::Sphere;

use Config;

use snowmew::Entity;

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
            gl::GenBuffers(buffer.len() as i32, buffer.unsafe_mut(0));
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, buffer[0]);
            gl::BufferData(gl::SHADER_STORAGE_BUFFER,
                           (mem::size_of::<ModelInfoSSBO>()*cfg.max_size()) as GLsizeiptr,
                           ptr::null(), gl::DYNAMIC_DRAW);
        }

        ModelInfoSSBOBuffer {
            ptr_model_info: ptr::null_mut(),
            model_info: buffer[0],
            size: cfg.max_size()
        }
    }

    pub fn map(&mut self) {
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.model_info);
            self.ptr_model_info = gl::MapBufferRange(gl::SHADER_STORAGE_BUFFER, 0,
                    (mem::size_of::<ModelInfoSSBO>()*self.size) as GLsizeiptr,
                    gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
            ) as *mut ModelInfoSSBO;
        }
    }

    pub fn unmap(&mut self) {
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.model_info);
            gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
            self.ptr_model_info = ptr::null_mut();
        }
    }

    pub fn build(&mut self, db: &Renderable, scene: Entity) {
        let position = db.compute_positions();
        unsafe {
            mut_buf_as_slice(self.ptr_model_info, self.size, |info| {
                for (idx, (id, (draw, pos))) in join_set_to_map(db.scene_iter(scene),
                                                join_maps(db.drawable_iter(), db.location_iter())).enumerate() {
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
            gl::GenBuffers(buffer.len() as i32, buffer.unsafe_mut(0));
            gl::GenTextures(texture.len() as i32, texture.unsafe_mut(0));

            gl::BindBuffer(gl::TEXTURE_BUFFER, buffer[0]);
            gl::BindTexture(gl::TEXTURE_BUFFER, texture[0]);
            gl::TexBuffer(gl::TEXTURE_BUFFER, gl::RGB32UI, buffer[0]);
            gl::BufferData(gl::TEXTURE_BUFFER,
                           (mem::size_of::<ModelInfoTexture>()*cfg.max_size()) as GLsizeiptr,
                           ptr::null(), gl::DYNAMIC_DRAW);
        }

        ModelInfoTextureBuffer {
            ptr_model_info: ptr::null_mut(),
            model_info: buffer[0],
            texture_model_info: texture[0],
            size: cfg.max_size()
        }
    }

    pub fn map(&mut self) {
        unsafe {
            gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_info);
            self.ptr_model_info = gl::MapBufferRange(gl::TEXTURE_BUFFER, 0,
                    (mem::size_of::<ModelInfoTexture>()*self.size) as GLsizeiptr,
                    gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
            ) as *mut ModelInfoTexture;
        }
    }

    pub fn unmap(&mut self) {
        unsafe {
            gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_info);
            gl::UnmapBuffer(gl::TEXTURE_BUFFER);
            self.ptr_model_info = ptr::null_mut();
        }
    }

    pub fn build(&mut self, db: &Renderable, scene: Entity) {
        let position = db.compute_positions();
        unsafe {
            mut_buf_as_slice(self.ptr_model_info, self.size, |info| {
                for (idx, (id, (draw, pos))) in join_set_to_map(db.scene_iter(scene),
                                                join_maps(db.drawable_iter(), db.location_iter())).enumerate() {
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