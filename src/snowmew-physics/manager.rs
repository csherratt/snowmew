
use std::vec::Vec;

use cgmath::point::Point3;
use cgmath::vector::{Vector, Vector4, Vector3};
use cgmath::matrix::{Matrix, Matrix4};
use cgmath::transform::Transform3D;

use cow::join::join_maps;

use snowmew::common::{ObjectKey, CommonData, Common};
use collision::bvh::{BvhBuilder, Bvh};
use collision::aabb::{Aabb3};
use collision::Merge;

use position::{Positions, ComputedPosition, PositionData};

use {Physics, Velocity, Collider, PhysicsData};

#[deriving(Clone)]
struct PhysicsTemp {
    common: CommonData,
    position: PositionData,
    physics: PhysicsData
}

impl PhysicsTemp {
    fn new<T: Physics>(t: &T) -> PhysicsTemp {
        PhysicsTemp {
            common: t.get_common().clone(),
            position: t.get_position().clone(),
            physics: t.get_physics().clone()
        }
    }
}

impl Common for PhysicsTemp {
    fn get_common<'a>(&'a self) -> &'a CommonData { &self.common }
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData { &mut self.common }
}

impl Positions for PhysicsTemp {
    fn get_position<'a>(&'a self) -> &'a PositionData { &self.position }
    fn get_position_mut<'a>(&'a mut self) -> &'a mut PositionData { &mut self.position }
}

impl Physics for PhysicsTemp {
    fn get_physics<'a>(&'a self) -> &'a PhysicsData { &self.physics }
    fn get_physics_mut<'a>(&'a mut self) -> &'a mut PhysicsData { &mut self.physics }
}

pub struct PhysicsManager {
    static_builder: Option<BvhBuilder<ObjectKey, Aabb3<f32>, Point3<f32>>>,
    static_bvh: Option<Bvh<ObjectKey, Aabb3<f32>>>,
    matrix: Vec<Matrix4<f32>>,
    version: uint
}

impl PhysicsManager {
    pub fn new() -> PhysicsManager { 
        PhysicsManager {
            static_builder: Some(BvhBuilder::new()),
            static_bvh: None,
            matrix: Vec::new(),
            version: 0
        }
    }

    fn build_static_bvh<P: Physics>(&mut self, pos: &ComputedPosition, data: &P) {
        if self.static_bvh.is_some() && data.get_physics().static_version == self.version {
            return;
        }

        let mut bvh = match self.static_builder.take() {
            Some(bvh) => bvh,
            None => match self.static_bvh.take() {
                Some(bvh_builder) => bvh_builder.to_builder(),
                None => fail!("no bvh or builder")
            }
        };

        for (key, (loc, &Collider(ref coll))) in join_maps(data.location_iter(), data.get_physics().static_colliders.iter()) {
            let aabb = recalc_aabb(coll, self.matrix.get(pos.get_loc(*loc)));
            bvh.add(aabb, *key);
        }

        self.static_bvh = Some(bvh.build());
        self.version = data.get_physics().static_version;
    }

    pub fn step<P: Physics>(&mut self, data: &mut P, time: f32) {
        let old = PhysicsTemp::new(data);
        unsafe { 
            self.matrix.reserve(old.position_count());
            self.matrix.set_len(old.position_count());
        }
        let pos = old.to_positions(&mut self.matrix.as_mut_slice());

        self.build_static_bvh(&pos, &old);
        match self.static_bvh {
            None => fail!("Could not unwrap bvh"),
            Some(ref bvh) => {
                for (key, ((loc, &Velocity(ref vel)), &Collider(ref coll))) in 
                        join_maps(join_maps(old.location_iter(), old.get_physics().velocity.iter()),
                                            old.get_physics().colliders.iter()) {
                    let vel = vel.mul_s(time);
                    let aabb = recalc_aabb_with_vec(coll, self.matrix.get(pos.get_loc(*loc)), &vel);
                    let mut collided = false;
                    for _ in bvh.collision_iter(&aabb) {
                        collided = true;
                        break;
                    }
                    if !collided {
                        let t = data.location(*key).expect("unxpeced missing location");
                        let disp = t.get().disp.add_v(&vel);
                        let scale = t.get().scale;
                        let rot = t.get().rot;
                        data.update_location(*key, Transform3D::new(scale, rot, disp));
                    }
                }                
            }
        }

    }
}

fn aabb_point(idx: uint, aabb: &Aabb3<f32>, mat: &Matrix4<f32>) -> Point3<f32> {
    let v = Vector4::new(if idx & 0x1 == 0x1 {aabb.min.x} else {aabb.max.x},
                         if idx & 0x2 == 0x2 {aabb.min.y} else {aabb.max.y},
                         if idx & 0x4 == 0x4 {aabb.min.z} else {aabb.max.z},
                         1.);
    let v = mat.mul_v(&v);
    Point3::from_homogeneous(&v)
}

fn recalc_aabb(aabb: &Aabb3<f32>, mat: &Matrix4<f32>) -> Aabb3<f32> {
    range(0u, 8).map(|i| aabb_point(i, aabb, mat)).collect()
}

fn recalc_aabb_with_vec(aabb: &Aabb3<f32>, mat: &Matrix4<f32>, vec: &Vector3<f32>) -> Aabb3<f32> {
    let start = recalc_aabb(aabb, mat);
    let end = recalc_aabb(aabb, &mat.mul_m(&Matrix4::translate(vec)));
    start.merge(&end)
}
