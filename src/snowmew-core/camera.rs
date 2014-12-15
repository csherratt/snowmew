//   Copyright 2014 Colin Sherratt
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

use cgmath::{Matrix, Matrix4, ToMatrix4};
use cgmath::{Vector3, Vector4};
use cgmath::{Point, Point3};
use cgmath::perspective;
use cgmath::deg;

// use ovr;
use ovr::{EyeRenderDescriptor, FovPort, Pose};

#[deriving(Copy)]
/// Camera can be used to do Camera like actions
pub struct Camera {
    transform: Matrix4<f32>
}

#[deriving(Copy)]
/// A set of matrices that can be used to render from the point
/// of view of the camera
pub struct DrawMatrices {
    /// The projection matrix
    pub projection: Matrix4<f32>,
    /// The view matrix
    pub view: Matrix4<f32>
}

fn view_matrix(t: &Matrix4<f32>) -> Matrix4<f32> {
    let eye = t.mul_v(&Vector4::new(0f32, 0f32, 0f32, 1f32));
    let target = t.mul_v(&Vector4::new(0f32, 0f32, -1f32, 1f32));
    let up = t.mul_v(&Vector4::new(0f32, 1f32, 0f32, 1f32));

    let up = Point3::from_homogeneous(&up).sub_p(&Point3::from_homogeneous(&eye));
    let eye = Point3::new(eye.x/eye.w, eye.y/eye.w, eye.z/eye.w);
    let target = Point3::new(target.x/target.w, target.y/target.w, target.z/target.w);

    Matrix4::look_at(&eye, &target, &up)
}

impl Camera {
    /// Create a new camera from a position matrix
    pub fn new(transform: Matrix4<f32>) -> Camera {
        Camera { transform: transform }
    }

    /// Create a perspective matrix for the Camera
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Matrix4<f32> {
        perspective(
            deg(80f32), aspect_ratio, 0.01f32, 10000f32
        )
    }

    /// Create a view matrix for the Camera
    pub fn view_matrix(&self) -> Matrix4<f32> {
        view_matrix(&self.transform)
    }

    /// Create a projection matrix and a view matrix for the camera.
    pub fn get_matrices(&self, size: (i32, i32)) -> DrawMatrices {
        let (w, h) = size;
        let (w, h) = (w as f32, h as f32);

        DrawMatrices {
            projection: self.projection_matrix(w/h),
            view: self.view_matrix()
        }
    }

    /// Create a set of draw matricies that are correct for OculusRift rendering
    /// This requires supplying the `fov`, `eye` and `pose` that is supplied pereye
    /// by `vr-rs`
    pub fn ovr(&self, fov: &FovPort, eye: &EyeRenderDescriptor, pose: &Pose) -> DrawMatrices {
        let projection = fov.projection(0.01, 10000., true);
        let view = self.transform.mul_m(&pose.orientation.to_matrix4());
        let view = view_matrix(&view).mul_m(&Matrix4::from_translation(&eye.view_adjust));

        DrawMatrices {
            projection: projection,
            view: view
        }
    }

    /// A utility function that will calculate a point relative to the camera
    /// This can be used to simulate moving of the character based on the current
    /// Camera's position.
    pub fn move_with_vector(&self, v: &Vector3<f32>) -> Point3<f32> {
        let o = self.transform.mul_v(&Vector4::new(v.x, v.y, v.z, 1f32));
        Point3::new(o.x, o.y, o.z)
    }
}