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

    pub fn map_Ka(&self) -> Option<ObjectKey> {self.map_ka}
    pub fn set_map_Ka(&mut self, oid: ObjectKey) {self.map_ka = Some(oid);}

    pub fn map_Kd(&self) -> Option<ObjectKey> {self.map_kd}
    pub fn set_map_Kd(&mut self, oid: ObjectKey) {self.map_kd = Some(oid);}

    pub fn map_Ks(&self) -> Option<ObjectKey> {self.map_ks}
    pub fn set_map_Ks(&mut self, oid: ObjectKey) {self.map_ks = Some(oid);}

    pub fn map_Ke(&self) -> Option<ObjectKey> {self.map_ke}
    pub fn set_map_Ke(&mut self, oid: ObjectKey) {self.map_ke = Some(oid);}

    pub fn map_ns(&self) -> Option<ObjectKey> {self.map_ns}
    pub fn set_map_ns(&mut self, oid: ObjectKey) {self.map_ns = Some(oid);}

    pub fn map_d(&self) -> Option<ObjectKey> {self.map_d}
    pub fn set_map_d(&mut self, oid: ObjectKey) {self.map_d = Some(oid);}

    pub fn map_bump(&self) -> Option<ObjectKey> {self.map_bump}
    pub fn set_map_bump(&mut self, oid: ObjectKey) {self.map_bump = Some(oid);}

    pub fn map_refl(&self) -> Option<ObjectKey> {self.map_refl}
    pub fn set_map_refl(&mut self, oid: ObjectKey) {self.map_refl = Some(oid);}

    pub fn ns(&self) -> f32 {self.ns}
    pub fn set_ns(&mut self, v: f32) {self.ns = v}

    pub fn ni(&self) -> f32 {self.ni}
    pub fn set_ni(&mut self, v: f32) {self.ni = v}

}