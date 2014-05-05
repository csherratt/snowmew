use std::default::Default;

use cgmath::vector::Vector3;

#[deriving(Clone)]
pub struct PhongMat
{
    ks: Vector3<f32>,
    kd: Vector3<f32>,
    ka: Vector3<f32>,
    alpha: f32
}

#[deriving(Clone)]
pub enum Material
{
    NoMaterial,
    Flat(Vector3<f32>),
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
    pub fn flat(color: Vector3<f32>) -> Material
    {
        Flat(color)
    }

    pub fn phong(ks: Vector3<f32>, kd: Vector3<f32>, ka: Vector3<f32>, alpha: f32) -> Material
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