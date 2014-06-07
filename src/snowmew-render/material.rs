
use std::mem;
use std::ptr;
use std::slice::raw::mut_buf_as_slice;

use cgmath::vector::Vector4;
use gl;

use snowmew::ObjectKey;
use graphics::{Material, Graphics};

#[packed]
struct MaterialStd140 {
    ka: Vector4<f32>,
    kd: Vector4<f32>,
    ks: Vector4<f32>,
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
        let ka = mat.Ka();
        let kd = mat.Kd();
        let ks = mat.Ks();

        MaterialStd140 {
            ka: Vector4::new(ka.x, ka.y, ka.z, 1.),
            kd: Vector4::new(kd.x, kd.y, kd.z, 1.),
            ks: Vector4::new(ks.x, ks.y, ks.z, 1.),
            ka_texture: get_mat(mat.map_Ka(), rd),
            kd_texture: get_mat(mat.map_Kd(), rd),
            ks_texture: get_mat(mat.map_Ks(), rd),
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
            gl::GenBuffers(1, ub.unsafe_mut_ref(0));
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
            ptr: ptr::mut_null(),
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
        self.ptr = ptr::mut_null();
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