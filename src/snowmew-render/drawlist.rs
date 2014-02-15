use std::mem;
use std::ptr;
use std::cast;
use std::libc::c_void;
use std::vec::raw::mut_buf_as_slice;

use cgmath::matrix::{Mat4, Matrix};
use cgmath::vector::{Vec4, Vector};
use db::Graphics;

use snowmew::core::{object_key};
use snowmew::geometry::Geometry;
use snowmew::core::Drawable;

use gl;
use gl::types::{GLuint, GLsizeiptr};
use cow::join::join_maps;

use collections::treemap::TreeMap;

pub struct ObjectCull<IN>
{
    priv input: IN,
    priv camera: Mat4<f32>
}

impl<'a, IN: Iterator<(object_key, Mat4<f32>)>> ObjectCull<IN>
{
    pub fn new(input: IN, camera: Mat4<f32>) -> ObjectCull<IN>
    {
        ObjectCull {
            input: input,
            camera: camera
        }
    }
}

impl<'a, IN: Iterator<(object_key, Mat4<f32>)>>
     Iterator<(object_key, Mat4<f32>)> for ObjectCull<IN>
{
    #[inline]
    fn next(&mut self) -> Option<(object_key, Mat4<f32>)>
    {
        static cube_points: &'static [Vec4<f32>] = &'static [
            Vec4{x:1., y:  1., z:  1., w: 1.}, Vec4{x:-1., y:  1., z:  1., w: 1.},
            Vec4{x:1., y: -1., z:  1., w: 1.}, Vec4{x:-1., y: -1., z:  1., w: 1.},
            Vec4{x:1., y:  1., z: -1., w: 1.}, Vec4{x:-1., y:  1., z: -1., w: 1.},
            Vec4{x:1., y: -1., z: -1., w: 1.}, Vec4{x:-1., y: -1., z: -1., w: 1.},
        ];

        loop {
            match self.input.next() {
                Some((oid, mat)) => {
                    let proj = self.camera.mul_m(&mat);

                    let mut behind_camera = true;
                    let mut right_of_camera = true;
                    let mut left_of_camera = true;
                    let mut above_camera = true;
                    let mut below_camera = true;

                    for p in cube_points.iter() {
                        let point = proj.mul_v(p);
                        let point = point.mul_s(1./point.w);

                        behind_camera &= point.z > 1.;
                        right_of_camera &= point.x > 1.;
                        left_of_camera &= point.x < -1.;
                        above_camera &= point.y > 1.;
                        below_camera &= point.y < -1.;
                    }

                    if !(behind_camera|right_of_camera|left_of_camera|above_camera|below_camera) {
                        return Some((oid, mat));
                    }
                },
                None => return None
            }
        }
    }
}

pub struct Expand<'a, IN>
{
    priv input: IN,
    priv material_id: object_key,
    priv vb_id: object_key,
    priv last_material_id: object_key,
    priv last_vb_id: object_key,
    priv mat: Option<Mat4<f32>>,
    priv geometry: Option<&'a Geometry>,
    priv db: &'a Graphics
}

impl<'a, IN: Iterator<(object_key, (Mat4<f32>, &'a Drawable))>> Expand<'a, IN>
{
    pub fn new(input: IN, db: &'a Graphics) -> Expand<'a, IN>
    {
        Expand {
            input: input,
            material_id: 0,
            vb_id: 0,
            last_material_id: 0,
            last_vb_id: 0,
            mat: None,
            geometry: None,
            db: db
        }
    }
}

pub enum DrawCommand
{
    Draw(Geometry),
    BindMaterial(object_key),
    BindVertexBuffer(object_key),
    SetModelMatrix(Mat4<f32>),
    MultiDraw(object_key, u32, u32, u32, u32),
    DrawElements(object_key, u32, i32, u32, i32, u32, i32),
    DrawElements2(object_key, Geometry, u32, ~[u32])
}

impl<'a, IN: Iterator<(object_key, (Mat4<f32>, &'a Drawable))>> Iterator<DrawCommand> for Expand<'a, IN>
{
    #[inline]
    fn next(&mut self) -> Option<DrawCommand>
    {
        loop {
            if self.material_id != self.last_material_id {
                self.last_material_id = self.material_id;
                return Some(BindMaterial(self.material_id));
            }

            if self.vb_id != self.last_vb_id {
                self.last_vb_id = self.vb_id;
                return Some(BindVertexBuffer(self.vb_id));
            }

            match self.mat {
                Some(mat) => {
                    let out = SetModelMatrix(mat);
                    self.mat = None;
                    return Some(out);
                },
                None => ()
            }

            match self.geometry {
                Some(geometry) => {
                    let out = Draw(geometry.clone());
                    self.geometry = None;
                    return Some(out);
                },
                None => ()
            }

            match self.input.next() {
                Some((_, (mat, draw))) => {
                    self.mat = Some(mat);
                    self.material_id = draw.material;
                    self.geometry = self.db.current.geometry(draw.geometry);
                    self.vb_id = self.geometry.unwrap().vb;
                },
                None => return None,
            }
        }
    }
}

struct Indirect {
    vertex_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: u32,
    base_instance: u32
}

pub struct Drawlist
{
    priv model_matrix: GLuint,
    priv model_matrix_ptr: *mut Mat4<f32>,
    priv max_size: uint,
    priv bins: TreeMap<Drawable ,~[u32]>,
    priv cmds: ~[DrawCommand]
}

impl Drawlist
{
    pub fn new(max_size: uint) -> Drawlist
    {
        let buffers = &mut [0];

        let model_matrix_ptr = unsafe {
            let size = (mem::size_of::<Mat4<f32>>() * max_size) as GLsizeiptr;
            let flags = gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT;
            gl::GenBuffers(1, buffers.unsafe_mut_ref(0));
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, buffers[0]);
            gl::BufferStorage(gl::SHADER_STORAGE_BUFFER, size, ptr::null(), flags);
            gl::MapBufferRange(gl::SHADER_STORAGE_BUFFER, 0, size, flags) as *mut Mat4<f32>
        };

        println!("model_matrix {:?}", model_matrix_ptr)

        Drawlist {
            model_matrix: buffers[0],
            model_matrix_ptr: model_matrix_ptr,
            max_size: 0,
            bins: TreeMap::new(),
            cmds: ~[]
        }
    }

    // This downloads the positions to the GPU and bins the objects
    pub fn setup_scene(&mut self, db: &Graphics, scene: object_key)
    {
        let num_drawable = db.current.drawable_count();
        assert!(self.max_size < num_drawable);

        // clear bins
        for (_, data) in self.bins.mut_iter() {
            unsafe {
                data.set_len(0);
            }
        }

        unsafe {
            mut_buf_as_slice(self.model_matrix_ptr, num_drawable, |write| {
                let list = join_maps(db.current.walk_scene(scene), db.current.walk_drawables());
                for (idx, (_, (mat, draw))) in list.enumerate() {
                    write[idx] = mat;
                    let empty = match self.bins.find_mut(draw) {
                        Some(dat) => {dat.push(idx as u32); false},
                        None => true
                    };
                    if empty {
                        self.bins.insert(draw.clone(), ~[idx as u32]);
                    }
                }
            });
        }
    }

    pub fn render<'a>(&'a mut self, db: &Graphics, camera: Mat4<f32>)
    {
        let shader = db.flat_instanced_shader.unwrap();
        shader.bind();
        shader.set_projection(&camera);

        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, self.model_matrix);

        for (draw, vals) in self.bins.iter() {
            let geo = db.current.geometry(draw.geometry).unwrap();
            let mat = db.current.material(draw.material).unwrap();

            let vbo = db.vertex.find(&geo.vb);
            vbo.unwrap().bind();

            shader.set_material(mat);

            for v in vals.chunks(512) {
                unsafe {
                    gl::Uniform1iv(1, v.len() as i32, cast::transmute(&v[0]));
                    gl::DrawElementsInstancedBaseInstance(gl::TRIANGLES,
                                                          geo.count as i32,
                                                          gl::UNSIGNED_INT,
                                                          (geo.offset * 32) as *c_void,
                                                          v.len() as i32, 0);
                }

            }
        }
    }
}

impl Drop for Drawlist
{
    fn drop(&mut self)
    {
        /* TODO, is dropped dies on a none-gl task bad things happen */
        //let buffers = &[self.model_matrix];
        //gl::UnmapBuffer(self.model_matrix);
        //gl::DeleteBuffers(1, buffers.unsafe_ref(0));
    }
}