use cgmath::matrix::{Matrix, Mat4};
use cgmath::vector::{Vec3, Vec4};
use cgmath::point::Point3;
use cgmath::quaternion::Quat;
use cgmath::projection::perspective;
use cgmath::angle::deg;

use ovr;
use ovr::HMDInfo;

pub struct Camera
{
    rotation: Quat<f32>,
    transform: Mat4<f32>
}

pub struct DrawMatrices
{
    projection: Mat4<f32>,
    view: Mat4<f32>
}

impl Camera {
    pub fn new(rot: Quat<f32>, transform: Mat4<f32>) -> Camera
    {
        Camera {
            rotation: rot,
            transform: transform
        }
    }

    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4<f32>
    {
        perspective(
            deg(80f32), aspect_ratio, 1f32, 10000f32
        )
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

    pub fn get_matrices(&self, size: (uint, uint)) -> DrawMatrices
    {
        let (w, h) = size;
        let (w, h) = (w as f32, h as f32);

        DrawMatrices {
            projection: self.projection_matrix(w/h),
            view: self.view_matrix()
        }
    }

    pub fn move(&self, v: &Vec3<f32>) -> Point3<f32>
    {
        let o = self.transform.mul_v(&Vec4::new(v.x, v.y, v.z, 1f32));
        Point3::new(o.x, o.y, o.z)
    }
}

impl DrawMatrices
{
    pub fn ovr(&self, hmd: &HMDInfo) -> (DrawMatrices, DrawMatrices)
    {
        let ((pl, pr), (vl, vr)) = ovr::create_reference_matrices(hmd, &self.view, 1.7);

        (DrawMatrices {
            projection: pl,
            view: vl
         },
         DrawMatrices {
            projection: pr,
            view: vr
         })
    }
}