
use snowmew::common::{ObjectKey};
use collision::bvh::{BvhBuilder};
use collision::aabb::{Aabb3};

use position::Positions;

use cgmath::point::Point3;
use cgmath::matrix::{Matrix4};

use cow::join::join_maps;

use {Physics};

pub struct PhysicsManager {
    static_builder: Option<BvhBuilder<ObjectKey, Aabb3<f32>, Point3<f32>>>,
    matrix: Vec<Matrix4<f32>>
}

impl PhysicsManager {
    pub fn new() -> PhysicsManager { 
        PhysicsManager {
            static_builder: Some(BvhBuilder::new()),
            matrix: Vec::new()
        }
    }

    pub fn step<P: Physics>(&mut self, data: &mut P, _: f32) {
        let mut move = Vec::new();

        for (key, (loc, vel)) in join_maps(data.location_iter(), data.get_physics().velocity.iter()) {
            move.push((key, loc, vel));
        } 

        /*unsafe { 
            self.matrix.reserve(data.position_count());
            self.matrix.set_len(data.position_count());
        }
        let pos = data.to_positions(&mut self.matrix.as_mut_slice());

        let mut bvh_builder = self.static_builder.take().unwrap();

        for (key, (loc, &Collider(ref aabb))) in join_maps(data.location_iter(), data.get_physics().static_colliders.iter()) {
            let mat = self.matrix.get(pos.get_loc(*loc));
            bvh_builder.add(recalc_aabb(aabb, mat), *key);
        }

        let bvh = bvh_builder.build();

        for (_, (loc, &Collider(ref aabb))) in join_maps(data.location_iter(), data.get_physics().colliders.iter()) {
            let mat = self.matrix.get(pos.get_loc(*loc));
            let aabb = recalc_aabb(aabb, mat);
            for i in bvh.collision_iter(&aabb) {
                println!("{:?}", i);
            }
        }

        self.static_builder = Some(bvh.to_builder());
        */
    }
}

/*fn aabb_point(idx: uint, aabb: &Aabb3<f32>, mat: &Matrix4<f32>) -> Point3<f32> {
    let v = Vector4::new(if idx & 0x1 == 0x1 {aabb.min.x} else {aabb.max.x},
                         if idx & 0x2 == 0x2 {aabb.min.y} else {aabb.max.y},
                         if idx & 0x4 == 0x4 {aabb.min.z} else {aabb.max.z},
                         1.);
    let v = mat.mul_v(&v);
    Point3::new(v.x/v.w, v.y/v.w, v.w/v.w)
}

fn recalc_aabb(aabb: &Aabb3<f32>, mat: &Matrix4<f32>) -> Aabb3<f32> {
    range(0u, 8).map(|i| aabb_point(i, aabb, mat)).collect()
}*/
