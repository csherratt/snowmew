use std::default::Default;

use cgmath::vector::Vec3;

#[deriving(Clone)]
pub struct PhongMat
{
    ks: Vec3<f32>,
    kd: Vec3<f32>,
    ka: Vec3<f32>,
    alpha: f32
}

#[deriving(Clone)]
pub enum Material
{
    NoMaterial,
    Flat(Vec3<f32>),
    Phong(PhongMat)
}

impl Default for Material
{
    fn default() -> Material
    {
        NoMaterial
    }
}

impl Material
{
    pub fn flat(color: Vec3<f32>) -> Material
    {
        Flat(color)
    }

    pub fn phong(ks: Vec3<f32>, kd: Vec3<f32>, ka: Vec3<f32>, alpha: f32) -> Material
    {
        Phong(
            PhongMat {
                ks: ks,
                kd: kd,
                ka: ka,
                alpha: alpha
            }
        )
    }
}