use std::default::Default;

use cgmath::vector::Vector3;

use snowmew::ObjectKey;

#[deriving(Clone)]
pub struct Material {
    ka: Vector3<f32>,
    kd: Vector3<f32>,
    ks: Vector3<f32>,
    ke: Vector3<f32>,
    tf: Vector3<f32>,
    ns: f32,
    ni: f32,
    tr: f32,
    d: f32,
    illum: int,

    map_ka:   Option<ObjectKey>,
    map_kd:   Option<ObjectKey>,
    map_ks:   Option<ObjectKey>,
    map_ke:   Option<ObjectKey>,
    map_ns:   Option<ObjectKey>,
    map_d:    Option<ObjectKey>,
    map_bump: Option<ObjectKey>,
    map_refl: Option<ObjectKey>,
}

impl Default for Material {
    fn default() -> Material {
        Material::new()
    }
}

impl Material {
    pub fn new() -> Material {
        Material {
            ka: Vector3::new(0f32, 0., 0.),
            kd: Vector3::new(0f32, 0., 0.),
            ks: Vector3::new(0f32, 0., 0.),
            ke: Vector3::new(0f32, 0., 0.),
            tf: Vector3::new(0f32, 0., 0.),
            ns: 0.,
            ni: 0.,
            tr: 0.,
            d: 0.,
            illum: 2,
            map_ka:   None,
            map_kd:   None,
            map_ks:   None,
            map_ke:   None,
            map_ns:   None,
            map_d:    None,
            map_bump: None,
            map_refl: None,
        }      
    }

    pub fn simple(color: Vector3<f32>) -> Material {
        let mut mat = Material::new();
        mat.ka = color.clone();
        mat.kd = color.clone();
        mat.ks = color.clone();
        mat
    }

    pub fn Ka(&self) -> Vector3<f32> {self.ka}
    pub fn set_Ka(&mut self, c: Vector3<f32>) {self.ka = c;}

    pub fn Kd(&self) -> Vector3<f32> {self.kd}
    pub fn set_Kd(&mut self, c: Vector3<f32>) {self.kd = c;}

    pub fn Ks(&self) -> Vector3<f32> {self.ks}
    pub fn set_Ks(&mut self, c: Vector3<f32>) {self.ks = c;}

    pub fn Ke(&self) -> Vector3<f32> {self.ks}
    pub fn set_Ke(&mut self, c: Vector3<f32>) {self.ke = c;}

    pub fn Tf(&self) -> Vector3<f32> {self.tf}
    pub fn set_Tf(&mut self, c: Vector3<f32>) {self.tf = c;}
}