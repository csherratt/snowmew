
use std::default::Default;
use cgmath::vector::Vector3;

impl Default for PointLight {
    fn default() -> PointLight {
        PointLight {
            color: Vector3::new(0f32, 0., 0.),
            intensity: 0.
        }
    }
}

#[deriving(Clone)]
pub struct PointLight {
    color: Vector3<f32>,
    intensity: f32
}

impl PointLight {
    pub fn new(color: Vector3<f32>,
               intensity: f32) -> PointLight {
        
        PointLight {
            color: color,
            intensity: intensity,
        }
    }

    pub fn color(&self) -> Vector3<f32> {self.color.clone()}
    pub fn intensity(&self) -> f32 {self.intensity.clone()}
}