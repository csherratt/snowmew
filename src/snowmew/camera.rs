use cgmath::matrix::{Matrix, Mat4};
use cgmath::vector::{Vec3, Vec4};
use cgmath::point::{Point3, Point};

pub struct Camera;


impl Camera {
    pub fn new() -> Camera
    {
        Camera
    }

    pub fn view_matrix(&self, transform: &Mat4<f32>) -> Mat4<f32>
    {
        let eye = transform.mul_v(&Vec4::new(0f32, 0f32, 0f32, 1f32));
        let target = transform.mul_v(&Vec4::new(0f32, 0f32, 1f32, 1f32));

        let eye = Point3::new(eye.x/eye.w, eye.y/eye.w, eye.z/eye.w);
        let target = Point3::new(target.x/target.w, target.y/target.w, target.z/target.w);

        println!("{:?} {:?}", eye, target);

        Mat4::look_at(&eye, &target, &Vec3::new(0f32, 1f32, 0f32))
    }


}