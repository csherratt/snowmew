use cgmath::matrix::{Matrix, Matrix4};
use cgmath::vector::{Vector3, Vector4};
use cgmath::point::{Point, Point3};
use cgmath::projection::perspective;
use cgmath::angle::deg;

use ovr;
use ovr::HMDInfo;

pub struct Camera {
    transform: Matrix4<f32>
}

pub struct DrawMatrices {
    pub projection: Matrix4<f32>,
    pub view: Matrix4<f32>
}

impl Camera {
    pub fn new(transform: Matrix4<f32>) -> Camera {
        Camera {
            transform: transform
        }
    }

    pub fn projection_matrix(&self, aspect_ratio: f32) -> Matrix4<f32> {
        perspective(
            deg(80f32), aspect_ratio, 0.01f32, 10000f32
        )
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        let eye = self.transform.mul_v(&Vector4::new(0f32, 0f32, 0f32, 1f32));
        let target = self.transform.mul_v(&Vector4::new(0f32, 0f32, -1f32, 1f32));
        let up = self.transform.mul_v(&Vector4::new(0f32, 1f32, 0f32, 1f32));

        let up = Point3::from_homogeneous(&up).sub_p(&Point3::from_homogeneous(&eye));
        let eye = Point3::new(eye.x/eye.w, eye.y/eye.w, eye.z/eye.w);
        let target = Point3::new(target.x/target.w, target.y/target.w, target.z/target.w);

        Matrix4::look_at(&eye, &target, &up)
    }

    pub fn get_matrices(&self, size: (i32, i32)) -> DrawMatrices {
        let (w, h) = size;
        let (w, h) = (w as f32, h as f32);

        DrawMatrices {
            projection: self.projection_matrix(w/h),
            view: self.view_matrix()
        }
    }

    pub fn move(&self, v: &Vector3<f32>) -> Point3<f32> {
        let o = self.transform.mul_v(&Vector4::new(v.x, v.y, v.z, 1f32));
        Point3::new(o.x, o.y, o.z)
    }
}

impl DrawMatrices
{
    pub fn ovr(&self, hmd: &HMDInfo, scale: f32) -> (DrawMatrices, DrawMatrices) {
        let ((pl, pr), (vl, vr)) = ovr::create_reference_matrices(hmd, &self.view, scale);

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