use cgmath::matrix::{Matrix, Mat4};
use cgmath::vector::{Vec3, Vec4};
use cgmath::point::{Point3};
use cgmath::quaternion::{Quat};

pub struct Camera
{
    rotation: Quat<f32>,
    transform: Mat4<f32>
}


impl Camera {
    pub fn new(rot: Quat<f32>, transform: Mat4<f32>) -> Camera
    {
        Camera {
            rotation: rot,
            transform: transform
        }
    }

    pub fn view_matrix(&self)  -> Mat4<f32>
    {
        let eye = self.transform.mul_v(&Vec4::new(0f32, 0f32, 0f32, 1f32));
        let target = self.transform.mul_v(&Vec4::new(0f32, 0f32, 1f32, 1f32));
        let up = self.rotation.mul_v(&Vec3::new(0f32, 1f32, 0f32));

        let eye = Point3::new(eye.x/eye.w, eye.y/eye.w, eye.z/eye.w);
        let target = Point3::new(target.x/target.w, target.y/target.w, target.z/target.w);

        Mat4::look_at(&eye, &target, &up)
    }

    pub fn move(&self, v: &Vec3<f32>) -> Point3<f32>
    {
        let o = self.transform.mul_v(&Vec4::new(v.x, v.y, v.z, 1f32));
        Point3::new(o.x, o.y, o.z)
    }
}