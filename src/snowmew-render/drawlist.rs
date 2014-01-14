use cgmath::matrix::{Mat4, ToMat4, Matrix};
use cgmath::vector::{Vec4, Vector};
use db::Graphics;
use snowmew::core::{object_key, Drawable};

use std::cmp::TotalOrd;
use std::vec::VecIterator;
use std::vec;

use cow::btree::BTreeMap;
use cow::btree::BTreeMapIterator;

pub struct Drawlist<'a>
{
    list: BTreeMap<order_key, Draw<'a>>
}

#[deriving(Clone, Default)]
pub struct Draw<'a>
{
    oid: object_key,
    mat: Mat4<f32>,
    draw: Drawable
}


#[deriving(Clone, Default)]
struct order_key(f32);
 
impl TotalEq for order_key
{
    fn equals(&self, other: &order_key) -> bool
    {
        let order_key(s) = *self;
        let order_key(o) = *other;

        s == o
    }
}

impl TotalOrd for order_key
{
    fn cmp(&self, other: &order_key) -> Ordering
    {
        let order_key(s) = *self;
        let order_key(o) = *other;

        if s > o {
            Greater
        } else {
            Less 
        } 
    }
}

impl<'a> Drawlist<'a>
{
    pub fn create<'a>(db: &'a Graphics, scene: object_key, camera: Mat4<f32>) -> Drawlist<'a>
    {
        let mut l = BTreeMap::new();
        let point = Vec4::new(0f32, 0f32, 0f32, 1f32);

        for (uid, mat, draw) in db.current.walk_drawables(scene) {
            let a = camera.mul_m(&mat);
            let a = a.mul_v(&point);
            let a = a.mul_s(1./a.w);

            if a.z > 0. && a.x > -1.2 && a.x < 1.2 && a.y > -1.2 && a.y < 1.2 {
                l.insert(order_key(a.z), Draw {
                    oid: uid,
                    mat: mat,
                    draw: draw.clone()
                });
            }
        }

        Drawlist {
            list: l,
        }
    }

    /*pub fn sort(&mut self, camera: Mat4<f32>)
    {
        let point = Vec4::new(0f32, 0f32, 0f32, 1f32);

        self.list.sort_by(
            |&a, &b| {
                let a = camera.mul_m(&a.mat);
                let b = camera.mul_m(&b.mat);

                let a = a.mul_v(&point);
                let b = b.mul_v(&point);

                let a = a.mul_s(1./a.w);
                let b = b.mul_s(1./b.w);

                if a.z > b.z {
                    Less
                } else if b.z > a.z {
                    Greater
                } else {
                    Equal
                }
            }
        );
    }*/

    pub fn iter(&'a self) -> BTreeMapIterator<'a, order_key, Draw<'a>>
    {
        println!("len: {:}", self.list.len());
        self.list.iter()
    }
}