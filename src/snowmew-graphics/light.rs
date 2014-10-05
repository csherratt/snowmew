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

use std::default::Default;
use cgmath::Vector3;
use serialize::{Encodable, Decodable};


#[deriving(Clone, Encodable, Decodable)]
pub struct Point {
    color: Vector3<f32>,
    intensity: f32
}


impl Point {
    pub fn new(color: Vector3<f32>,
               intensity: f32) -> Point {
        
        Point {
            color: color,
            intensity: intensity,
        }
    }

    pub fn color(&self) -> Vector3<f32> {self.color.clone()}
    pub fn intensity(&self) -> f32 {self.intensity.clone()}
}

#[deriving(Clone, Encodable, Decodable)]
pub struct Directional {
    normal: Vector3<f32>,
    color: Vector3<f32>,
    intensity: f32
}

impl Directional {
    pub fn new(normal: Vector3<f32>,
               color: Vector3<f32>,
               intensity: f32) -> Directional {
        
        Directional {
            normal: normal,
            color: color,
            intensity: intensity,
        }
    }

    pub fn normal(&self) -> Vector3<f32> {self.normal.clone()}
    pub fn color(&self) -> Vector3<f32> {self.color.clone()}
    pub fn intensity(&self) -> f32 {self.intensity.clone()}
}

#[deriving(Clone, Encodable, Decodable)]
pub enum Light {
    DirectionalLight(Directional),
    PointLight(Point)
}

impl Default for Light {
    fn default() -> Light {
        PointLight(Point {
            color: Vector3::new(0f32, 0., 0.),
            intensity: 0.
        })
    }
}