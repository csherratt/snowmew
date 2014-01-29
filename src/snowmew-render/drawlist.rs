use cgmath::matrix::{Mat4, Matrix};
use cgmath::vector::{Vec4, Vector};
use db::Graphics;
use snowmew::core::{object_key};

use snowmew::geometry::Geometry;
use snowmew::core::Drawable;

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
    priv shader_id: object_key,
    priv vb_id: object_key,
    priv last_shader_id: object_key,
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

impl<'a, IN: Iterator<(object_key, (Mat4<f32>, &'a Drawable))>> Iterator<DrawCommand> for Expand<'a, IN>
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
                Some((oid, (mat, draw))) => {
                    self.mat = Some(mat);
                    self.shader_id = draw.shader;
                    self.geometry = self.db.current.geometry(draw.geometry);
                },
                None => return None,
            }
        }
    }
}