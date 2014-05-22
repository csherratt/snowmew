
use std::default::Default;
use cgmath::vector::Vector3;

impl Default for PointLight {
    fn default() -> PointLight {
        PointLight {
            center: Vector3::new(0f32, 0., 0.),
            color: Vector3::new(0f32, 0., 0.),
            intensity: 0.
        }
    }
}

#[deriving(Clone)]
pub struct PointLight {
    center: Vector3<f32>,
    color: Vector3<f32>,
    intensity: f32
}

impl PointLight {
    pub fn new(pos: Vector3<f32>, color: Vector3<f32>, intensity: f32) -> PointLight {
        PointLight {
            center: pos,
            color: color,
            intensity: intensity
        }
    }

    pub fn center(&self) -> Vector3<f32> {self.center.clone()}
    pub fn color(&self) -> Vector3<f32> {self.color.clone()}
    pub fn intensity(&self) -> f32 {self.intensity.clone()}
}