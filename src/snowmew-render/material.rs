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
use std::slice::raw::mut_buf_as_slice;

use gl;

use snowmew::ObjectKey;
use graphics::{Material, Graphics};

#[repr(packed)]
struct MaterialStd140 {
    ka: [f32, ..4],
    kd: [f32, ..4],
    ks: [f32, ..4],
    ka_texture: (i32, i32),
    kd_texture: (i32, i32),
    ks_texture: (i32, i32),
    ns: f32,
    ni: f32
}

fn get_mat(ka: Option<ObjectKey>, rd: &Graphics) -> (i32, i32) {
    match ka {
        Some(ref ka) => {
            let (a, b) = *rd.get_texture_atlas_index(*ka).expect("Could not find index");
            (a as i32, b as i32)
        }
        None => (-1i32, 0i32)
    }
}

impl MaterialStd140 {
    pub fn from(mat: &Material, rd: &Graphics) -> MaterialStd140 {
        let ka = mat.ka();
        let kd = mat.kd();
        let ks = mat.ks();

        MaterialStd140 {
            ka: [ka[0], ka[1], ka[2], 1.],
            kd: [kd[0], kd[1], kd[2], 1.],
            ks: [ks[0], ks[1], ks[2], 1.],
            ka_texture: get_mat(mat.map_ka(), rd),
            kd_texture: get_mat(mat.map_kd(), rd),
            ks_texture: get_mat(mat.map_ks(), rd),
            ns: mat.ns(),
            ni: mat.ni()
        }
    }
}

pub struct MaterialBuffer {
    buffer: u32,
    size: uint,
    ptr: *mut MaterialStd140,
}

impl MaterialBuffer {
    pub fn new(max: uint) -> MaterialBuffer {
        let ub = &mut [0];

        unsafe {
            gl::GenBuffers(1, ub.unsafe_mut(0));
            gl::BindBuffer(gl::UNIFORM_BUFFER, ub[0]);
            gl::BufferData(gl::UNIFORM_BUFFER,
                           (max * mem::size_of::<MaterialStd140>()) as i64,
                           ptr::null(),
                           gl::DYNAMIC_DRAW);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }

        MaterialBuffer {
            buffer: ub[0],
            size: max,
            ptr: ptr::null_mut(),
        }
    }

    pub fn map(&mut self) {
        gl::BindBuffer(gl::UNIFORM_BUFFER, self.buffer);
        self.ptr = gl::MapBufferRange(gl::UNIFORM_BUFFER, 0,
                                      (self.size * mem::size_of::<MaterialStd140>()) as i64,
                                      gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
                                      ) as *mut MaterialStd140;
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
    }

    pub fn unmap(&mut self) {
        self.ptr = ptr::null_mut();
        gl::BindBuffer(gl::UNIFORM_BUFFER, self.buffer);
        gl::UnmapBuffer(gl::UNIFORM_BUFFER);
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
    }

    pub fn build(&mut self, graphics: &Graphics) {
        unsafe {
            mut_buf_as_slice(self.ptr, self.size, |b| {
                for (id, (_, mat)) in graphics.material_iter().enumerate() {
                    b[id] = MaterialStd140::from(mat, graphics);
                } 
            });
        }
    }

    pub fn id(&self) -> u32 {self.buffer}
}