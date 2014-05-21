
use std::mem;
use std::ptr;
use std::slice::raw::mut_buf_as_slice;
use collections::treemap::TreeMap;

use cgmath::vector::Vector4;
use gl;

use snowmew::ObjectKey;
use graphics::{Graphics, Material};

use db::GlState;

#[packed]
struct MaterialStd140 {
    kd: Vector4<f32>,
    kd_texture: (i32, i32),
    padd_end: (i32, i32),
}

fn get_mat(ka: Option<ObjectKey>, gl: &GlState) -> (i32, i32) {
    match ka {
        Some(ref ka) => gl.texture.get_index(*ka).unwrap(),
        None => (-1, 0)
    }
}

impl MaterialStd140 {
    pub fn from(mat: &Material, gl: &GlState) -> MaterialStd140 {
        let kd = mat.Ka();
        MaterialStd140 {
            kd: Vector4::new(kd.x, kd.y, kd.z, 1.),
            kd_texture: get_mat(mat.map_Ka(), gl),
            padd_end: (0, 0)
        }
    }
}

pub struct MaterialBuffer {
    buffer: u32,
    size: uint,
    ptr: *mut MaterialStd140,
    material_to_id: TreeMap<ObjectKey, u32>,
    id_to_material: TreeMap<u32, ObjectKey>,
}

impl MaterialBuffer {
    pub fn new(max: uint) -> MaterialBuffer {
        println!("sizeof {}", mem::size_of::<MaterialStd140>());
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
            material_to_id: TreeMap::new(),
            id_to_material: TreeMap::new()
        }
    }

    pub fn map(&mut self) {
        gl::BindBuffer(gl::UNIFORM_BUFFER, self.buffer);
        self.ptr = gl::MapBufferRange(gl::UNIFORM_BUFFER, 0,
                                      (self.size * mem::size_of::<MaterialStd140>()) as i64,
                                      gl::MAP_WRITE_BIT) as *mut MaterialStd140;
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
    }

    pub fn unmap(&mut self) {
        self.ptr = ptr::mut_null();
        gl::BindBuffer(gl::UNIFORM_BUFFER, self.buffer);
        gl::UnmapBuffer(gl::UNIFORM_BUFFER);
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
    }

    pub fn build(&mut self, graphics: &Graphics, gl: &GlState) {
        self.material_to_id.clear();
        self.id_to_material.clear();
        unsafe {
            mut_buf_as_slice(self.ptr, self.size, |b| {
                for (id, (key, mat)) in graphics.material_iter().enumerate() {
                    b[id] = MaterialStd140::from(mat, gl);
                    self.material_to_id.insert(*key, (id+1) as u32);
                    self.id_to_material.insert((id+1) as u32, *key);
                } 
            });
        }
    }

    pub fn id(&self) -> u32 {self.buffer}
}