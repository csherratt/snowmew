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

//
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
    priv geo_id: object_key,
    priv geo: Option<&'a Geometry>,
    priv vb_id: object_key,
    priv vb: Option<&'a VertexBuffer>,
    priv shader_id: object_key,
    priv shader: Option<&'a Shader>,
    priv camera: Mat4<f32>,
    priv db: &'a Graphics
}

impl<'a, IN: Iterator<(object_key, Mat4<f32>, &'a Drawable)>> Expand<'a,IN>
{
    pub fn new(input: IN, camera: Mat4<f32>, db: &'a Graphics) -> Expand<'a,IN>
    {
        Expand {
            input: input,
            geo: None,
            vb: None,
            shader: None,
            geo_id: 0,
            vb_id: 0,
            shader_id: 0,
            camera: camera,
            db: db
        }
    }
}

impl<'a, IN: Iterator<(object_key, Mat4<f32>, &'a Drawable)>>
     Iterator<(object_key, Mat4<f32>, &'a Geometry, &'a VertexBuffer, &'a Shader)> for Expand<'a, IN>
{
    fn next(&mut self) -> Option<(object_key, Mat4<f32>, &'a Geometry, &'a VertexBuffer, &'a Shader)>
    {
            match self.input.next() {
                Some((oid, mat, draw)) => {
                    if draw.geometry != self.geo_id {
                        self.geo = self.db.current.geometry(draw.geometry);
                        self.geo_id = draw.geometry;
                    }
                    let geo = self.geo.unwrap();

                    if geo.vb != self.vb_id {
                        self.vb = self.db.vertex.find(&geo.vb);
                        self.vb_id = geo.vb;
                        self.vb.unwrap().bind();
                    }
                    let vb = self.vb.unwrap();

                    if draw.shader != self.shader_id {
                        self.shader = self.db.shaders.find(&draw.shader);
                        self.shader_id = draw.shader;
                        self.shader.unwrap().bind();
                        self.shader.unwrap().set_projection(&self.camera)
                    }
                    let shader = self.shader.unwrap();

                    return Some((oid, mat, geo, vb, shader));
                },
                None => return None
            }
    }
}