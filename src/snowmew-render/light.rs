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

use std::ptr;
use std::mem;

use gl;
use gl::types::GLuint;

use cgmath::Matrix;
use cgmath::{EuclideanVector, Vector, Vector3, Vector4};

use position::Positions;
use graphics;
use graphics::Graphics;

const POINT_LIGHT_MAX: uint = 480;
const DIRECTIONAL_MAX: uint = 8;

#[repr(packed)]
struct PointLight {
    color: Vector4<f32>,
    position: Vector4<f32>
}

#[repr(packed)]
struct DirectionLight {
    color: Vector4<f32>,
    normal: Vector4<f32>
}

#[repr(packed)]
struct LightsStd140 {
    point_count: u32,
    direction_count: u32,
    padd: (i32, i32),
    point_lights: [PointLight, ..POINT_LIGHT_MAX],
    direction_lights: [DirectionLight, ..DIRECTIONAL_MAX]
}

pub struct LightsBuffer {
    buffer: GLuint,
    ptr: *mut LightsStd140,
}

impl LightsBuffer {
    pub fn new() -> LightsBuffer {
        let ub = &mut [0];
        unsafe {
            gl::GenBuffers(1, ub.unsafe_mut(0));
            gl::BindBuffer(gl::UNIFORM_BUFFER, ub[0]);
            gl::BufferData(gl::UNIFORM_BUFFER,
                           mem::size_of::<LightsStd140>() as i64,
                           ptr::null(),
                           gl::DYNAMIC_DRAW);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }

        LightsBuffer {
            buffer: ub[0],
            ptr: ptr::null_mut()
        }
    }

    pub fn map(&mut self) {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.buffer);
            self.ptr = gl::MapBufferRange(
                gl::UNIFORM_BUFFER, 0,
                mem::size_of::<LightsStd140>() as i64,
                gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
            ) as *mut LightsStd140;
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
    }

    pub fn unmap(&mut self) {
        self.ptr = ptr::null_mut();
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.buffer);
            gl::UnmapBuffer(gl::UNIFORM_BUFFER);
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
    }

    pub fn build<G: Graphics + Positions>(&mut self, graphics: &G) {
        let ptr: &mut LightsStd140 = unsafe { mem::transmute(self.ptr) };
        let base = Vector4::new(0f32, 0., 0., 1.);
        let mut point_light_count = 0u;
        let mut direction_light_count = 0u;

        fn color(color: Vector3<f32>, intensity: f32) -> Vector4<f32> {
            let c = color.mul_s(intensity);
            Vector4::new(c.x, c.y, c.z, 1.)
        };

        for (key, light) in graphics.light_iter() {
            match light {
                &graphics::PointLight(p) => {
                    if point_light_count == POINT_LIGHT_MAX {
                        println!("Dropping point light, overflow dropping light");
                    } else {
                        ptr.point_lights[point_light_count] =
                            PointLight {
                                color: color(p.color(), p.intensity()),
                                position: graphics.position(key).mul_v(&base)
                            };
                        point_light_count += 1;
                    }
                }
                &graphics::DirectionalLight(d) => {
                    if direction_light_count == DIRECTIONAL_MAX {
                        println!("Dropping directional light, overflow dropping light");
                    } else {
                        let n = d.normal();
                        let n = Vector4::new(n.x, n.y, n.z, 0.);
                        let n = graphics.position(key).mul_v(&n).normalize();
                        ptr.direction_lights[direction_light_count] =
                            DirectionLight {
                                color: color(d.color(), d.intensity()),
                                normal: n
                            };
                        direction_light_count += 1;
                    }
                }
            }
        }

        ptr.point_count = point_light_count as u32;
        ptr.direction_count = direction_light_count as u32;
    }

    pub fn id(&self) -> u32 {
        self.buffer
    }
}