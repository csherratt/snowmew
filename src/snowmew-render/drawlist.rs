use std::mem;
use std::ptr;
use std::cast;
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
    DrawElements(object_key, u32, i32, u32, i32, u32, i32)
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
    priv size_model: uint,
    priv bins: TreeMap<Drawable ,~[Mat4<f32>]>,
    priv cmds: ~[DrawCommand]
}

impl Drawlist
{
    pub fn new() -> Drawlist
    {
        let mut buffers = &mut [0];

        unsafe {
            gl::GenBuffers(1, buffers.unsafe_mut_ref(0));
        }

        Drawlist {
            model_matrix: buffers[0],
            size_model: 0,
            bins: TreeMap::new(),
            cmds: ~[]
        }
    }

    // This downloads the positions to the GPU and bins the objects
    pub fn setup_scene(&mut self, db: &Graphics, scene: object_key)
    {
        let num_drawable = db.current.drawable_count();

        // clear bins
        for (_, data) in self.bins.mut_iter() {
            unsafe {
                data.set_len(0);
            }
        }

        let mut list = join_maps(db.current.walk_scene(scene), db.current.walk_drawables());
        for (id, (mat, draw)) in list {
            let empty = match self.bins.find_mut(draw) {
                Some(dat) => {dat.push(mat); false},
                None => true
            };
            if empty {
                self.bins.insert(draw.clone(), ~[mat]);
            }
        }

        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.model_matrix);
            if self.size_model < num_drawable {
                gl::BufferData(gl::SHADER_STORAGE_BUFFER,
                               (mem::size_of::<Mat4<f32>>() * num_drawable) as GLsizeiptr,
                               ptr::null(),
                               gl::DYNAMIC_READ);
                self.size_model = num_drawable;
            }

            let mut idx = 0;
            for (_, items) in self.bins.iter() {
                let size = mem::size_of::<Mat4<f32>>() * items.len();
                gl::BufferSubData(gl::SHADER_STORAGE_BUFFER,
                                  idx,
                                  size as i64,
                                  cast::transmute(&items[0]));
                idx += size as i64;
            }
        }
    }

    pub fn generate<'a>(&'a mut self, db: &Graphics) -> &'a [DrawCommand]
    {
        let size = self.bins.len();   

        unsafe { self.cmds.set_len(0); }

        let mut idx = 0;
        let mut base = 0;
        for (draw, vals) in self.bins.iter() {
            let geo = db.current.geometry(draw.geometry).unwrap();
            self.cmds.push(BindMaterial(draw.material));
            self.cmds.push(DrawElements(geo.vb,
                                        self.model_matrix,
                                        geo.count as i32,
                                        geo.offset as u32,
                                        0,
                                        base as u32,
                                        vals.len() as i32));
            base += vals.len();
            idx += 1;
        }

        self.cmds.slice(0, self.cmds.len())
    }
}

impl Drop for Drawlist
{
    fn drop(&mut self)
    {
        unsafe {
            let buffers = &[self.model_matrix];
            gl::DeleteBuffers(1, buffers.unsafe_ref(0));
        }
    }
}