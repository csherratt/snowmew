use cgmath::matrix::{Mat4, ToMat4, Matrix};
use cgmath::vector::{Vec4, Vector};
use db::Graphics;
use snowmew::core::{object_key, Drawable};

use std::cmp::TotalOrd;
use std::vec::VecIterator;
use std::vec;

use cow::btree::BTreeMap;
use cow::btree::BTreeMapIterator;

use snowmew::geometry::Geometry;
use vertex_buffer::VertexBuffer;
use shader::Shader;

use OpenCL::CL::CL_MEM_READ_WRITE;
use OpenCL::mem::CLBuffer;
use OpenCL::hl::{Context, CommandQueue};

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
        loop {
            match self.input.next() {
                Some((oid, mat)) => {
                    let proj = self.camera.mul_m(&mat);
                    let points = &mut [
                        Vec4::new(1f32,  1f32,  1f32, 1f32), Vec4::new(-1f32,  1f32,  1f32, 1f32),
                        Vec4::new(1f32, -1f32,  1f32, 1f32), Vec4::new(-1f32, -1f32,  1f32, 1f32),
                        Vec4::new(1f32,  1f32, -1f32, 1f32), Vec4::new(-1f32,  1f32, -1f32, 1f32),
                        Vec4::new(1f32, -1f32, -1f32, 1f32), Vec4::new(-1f32, -1f32, -1f32, 1f32),
                    ];

                    for i in range(0, points.len()) {
                        let new = proj.mul_v(&points[i]);
                        points[i] = new.mul_s(1./new.w);
                    }

                    let mut infront = false;
                    for point in points.iter() {
                        infront |= (point.x < 1.) && (point.x > -1.) &&
                                   (point.y < 1.) && (point.y > -1.) &&
                                   (point.z < 1.);
                    }

                    if infront {
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
    priv shader_id: object_key,
    priv vb_id: object_key,
    priv last_shader_id: object_key,
    priv last_vb_id: object_key,
    priv mat: Option<Mat4<f32>>,
    priv geometry: Option<&'a Geometry>,
    priv db: &'a Graphics
}

impl<'a, IN: Iterator<(object_key, Mat4<f32>)>> Expand<'a,IN>
{
    pub fn new(input: IN, db: &'a Graphics) -> Expand<'a,IN>
    {
        Expand {
            input: input,
            shader_id: 0,
            vb_id: 0,
            last_shader_id: 0,
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
    BindShader(object_key),
    BindVertexBuffer(object_key),
    SetMatrix(Mat4<f32>),
}

impl<'a, IN: Iterator<(object_key, Mat4<f32>)>> Iterator<DrawCommand> for Expand<'a, IN>
{
    #[inline]
    fn next(&mut self) -> Option<DrawCommand>
    {
        loop {
            if self.shader_id != self.last_shader_id {
                self.last_shader_id = self.shader_id;
                return Some(BindShader(self.shader_id));
            }

            if self.vb_id != self.last_vb_id {
                self.last_vb_id = self.vb_id;
                return Some(BindVertexBuffer(self.vb_id));
            }

            match self.mat {
                Some(mat) => {
                    let out = SetMatrix(mat);
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
                Some((oid, mat)) => {
                    self.mat = Some(mat);
                    match self.db.current.drawable(oid) {
                        Some(draw) => {
                            self.shader_id = draw.shader;
                            self.geometry = self.db.current.geometry(draw.geometry);
                        },
                        None => ()
                    }
                },
                None => return None,
            }
        }
    }
}