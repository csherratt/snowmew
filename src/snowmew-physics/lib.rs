#![crate_id = "github.com/csherratt/snowmew#snowmew-physics:0.1"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "A collison detection manager for snowmew"]

extern crate snowmew;
extern crate cow;
extern crate cgmath;
extern crate position = "snowmew-position";
extern crate collision;

use snowmew::common::{ObjectKey, Common};
use position::Positions;

use collision::bvh::BvhBuilder;
use collision::aabb::{Aabb, Aabb3};

use cgmath::point::Point3;
use cgmath::matrix::{Matrix, Matrix4};
use cgmath::vector::{Vector3, Vector4};

use cow::btree::BTreeMap;
use cow::join::join_maps;

pub mod manager;

#[deriving(Clone)]
struct Collider(Aabb3<f32>);

impl std::default::Default for Collider {
    fn default() -> Collider {
        Collider(Aabb3::new(Point3::new(0f32, 0., 0.),
                            Point3::new(0f32, 0., 0.)))
    }
}

#[deriving(Clone)]
struct Velocity(Vector3<f32>);

impl std::default::Default for Velocity {
    fn default() -> Velocity {
        Velocity(Vector3::new(0f32, 0., 0.))
    }
}

#[deriving(Clone)]
pub struct PhysicsData {
    static_colliders: BTreeMap<ObjectKey, Collider>,
    colliders: BTreeMap<ObjectKey, Collider>,
    velocity: BTreeMap<ObjectKey, Velocity>
}

impl PhysicsData {
    pub fn new() -> PhysicsData {
        PhysicsData {
            static_colliders: BTreeMap::new(),
            colliders: BTreeMap::new(),
            velocity: BTreeMap::new()
        }
    }
}

fn aabb_point(idx: uint, aabb: &Aabb3<f32>, mat: &Matrix4<f32>) -> Point3<f32> {
    let v = Vector4::new(if idx & 0x1 == 0x1 {aabb.min.x} else {aabb.max.x},
                         if idx & 0x2 == 0x2 {aabb.min.y} else {aabb.max.y},
                         if idx & 0x4 == 0x4 {aabb.min.z} else {aabb.max.z},
                         1.);
    let v = mat.mul_v(&v);
    Point3::new(v.x/v.w, v.y/v.w, v.z/v.w)
}

fn recalc_aabb(aabb: &Aabb3<f32>, mat: &Matrix4<f32>) -> Aabb3<f32> {
    let mut new_aabb = Aabb3::new(aabb_point(0, aabb, mat),
                                  aabb_point(1, aabb, mat));
    for i in range(2u, 8) {
        new_aabb = new_aabb.grow(&aabb_point(i, aabb, mat));
    }

    new_aabb
}

pub trait Physics: Common + Positions {
    fn get_physics<'a>(&'a self) -> &'a PhysicsData;
    fn get_physics_mut<'a>(&'a mut self) -> &'a mut PhysicsData;

    fn add_static_collider(&mut self, key: ObjectKey, collider: Aabb3<f32>) {
        self.get_physics_mut().static_colliders.insert(key, Collider(collider));   
    }

    fn get_static_collider<'a>(&'a self, key: ObjectKey) -> Option<&'a Aabb3<f32>> {
        match self.get_physics().static_colliders.find(&key) {
            Some(&Collider(ref c)) => Some(c),
            None => None
        }
    }

    fn add_collider(&mut self, key: ObjectKey, collider: Aabb3<f32>) {
        self.get_physics_mut().colliders.insert(key, Collider(collider));   
    }

    fn get_collider<'a>(&'a self, key: ObjectKey) -> Option<&'a Aabb3<f32>> {
        match self.get_physics().colliders.find(&key) {
            Some(&Collider(ref c)) => Some(c),
            None => None
        }
    }

    fn set_velocity(&mut self, key: ObjectKey, v: Vector3<f32>) {
        self.get_physics_mut().velocity.insert(key, Velocity(v));
    }

    fn get_velocity(&self, key: ObjectKey) -> Option<Vector3<f32>> {
        match self.get_physics().velocity.find(&key) {
            Some(&Velocity(ref dat)) => Some(dat.clone()),
            None => None
        }
    }

    fn check_collision(&self) {
        let mut mats: Vec<Matrix4<f32>> = Vec::with_capacity(self.position_count());
        unsafe { mats.set_len(self.position_count()) };
        let pos = self.to_positions(&mut mats.as_mut_slice());

        let mut bvh_builder = BvhBuilder::new();

        for (key, (loc, &Collider(ref aabb))) in join_maps(self.location_iter(), self.get_physics().static_colliders.iter()) {
            let mat = mats.get(pos.get_loc(*loc));
            bvh_builder.add(recalc_aabb(aabb, mat), *key);
        }

        let bvh = bvh_builder.build();

        for (_, (loc, &Collider(ref aabb))) in join_maps(self.location_iter(), self.get_physics().colliders.iter()) {
            let mat = mats.get(pos.get_loc(*loc));
            let aabb = recalc_aabb(aabb, mat);
            for i in bvh.collision_iter(&aabb) {
                println!("{:?}", i);
            }
        }
    }
}

