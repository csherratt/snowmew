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



pub struct ObjectCull<IN>
{
    priv input: IN,
    priv camera: Mat4<f32>
}

impl<'a, IN: Iterator<(object_key, Mat4<f32>, &'a Drawable)>> ObjectCull<IN>
{
    pub fn new(input: IN, camera: Mat4<f32>) -> ObjectCull<IN>
    {
        ObjectCull {
            input: input,
            camera: camera
        }
    }
}

impl<'a, IN: Iterator<(object_key, Mat4<f32>, &'a Drawable)>>
     Iterator<(object_key, Mat4<f32>, &'a Drawable)> for ObjectCull<IN>
{
    fn next(&mut self) -> Option<(object_key, Mat4<f32>, &'a Drawable)>
    {
        loop {
            match self.input.next() {
                Some((oid, mat, draw)) => {
                    let a = self.camera.mul_m(&mat);
                    let a = a.mul_v(&Vec4::new(0f32, 0f32, 0f32, 1f32));
                    let a = a.mul_s(1./a.w);

                    // this check is CRAP
                    if a.z > 0. && a.x > -1.2 && a.x < 1.2 && a.y > -1.2 && a.y < 1.2 {
                        return Some((oid, mat, draw));
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

impl<'a, IN: Iterator<(object_key, Mat4<f32>, &'a Drawable)>> Expand<'a,IN>
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
    Done,
    Draw(Geometry),
    BindShader(object_key),
    BindVertexBuffer(object_key),
    SetMatrix(Mat4<f32>),
}

impl<'a, IN: Iterator<(object_key, Mat4<f32>, &'a Drawable)>> Iterator<DrawCommand> for Expand<'a, IN>
{
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
                Some((oid, mat, draw)) => {
                    self.mat = Some(mat);
                    self.shader_id = draw.shader;
                    self.geometry = self.db.current.geometry(draw.geometry);
                },
                None => return None,
            }
        }
    }
}