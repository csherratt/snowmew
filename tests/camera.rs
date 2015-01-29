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

extern crate "snowmew-render" as render;
extern crate cgmath;

use render::camera::Camera;
use cgmath::{Matrix4};
use cgmath::{Vector3};
use cgmath::{Point3, Point};

#[test]
fn origin() {
    let v = Vector3::new(1., 1., 1.);
    let m = Matrix4::from_translation(&v);
    let camera = Camera::new(10, 10, m);

    let origin = camera.origin();
    assert_eq!(Point3::new(1., 1., 1.), origin);

    let camera = Camera::new(10, 10, m*m);
    let origin = camera.origin();
    assert_eq!(Point3::new(2., 2., 2.), origin);
}

#[test]
fn test_ray() {
    let v = Vector3::new(1., 1., 1.);
    let m = Matrix4::from_translation(&v);
    let camera = Camera::new(10, 10, m);

    let o = Point3::new(1., 1., 1.);
    let ray = camera.pixel_ray(5, 5);
    assert_eq!(ray.origin, o);
    assert_eq!(ray.direction, Vector3::new(0., 0., -1.));
}