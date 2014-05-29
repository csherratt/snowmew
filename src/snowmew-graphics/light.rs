
use std::default::Default;
use cgmath::vector::Vector3;


#[deriving(Clone)]
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

#[deriving(Clone)]
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

#[deriving(Clone)]
pub enum Light {
    Directional(Directional),
    Point(Point)
}

impl Default for Light {
    fn default() -> Light {
        Point(Point {
            color: Vector3::new(0f32, 0., 0.),
            intensity: 0.
        })
    }
}