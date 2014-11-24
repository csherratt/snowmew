//   Copyright 2014 Colin Sherratt
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

#![crate_name = "snowmew-position"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "A position manager for snowmew"]
#![feature(macro_rules)]
#![feature(tuple_indexing)]

extern crate "snowmew-core" as snowmew;
extern crate cgmath;
extern crate opencl;
extern crate cow;
extern crate time;
extern crate serialize;

use std::default::Default;
use serialize::Encodable;

use cgmath::{Transform, Decomposed};
use cgmath::Quaternion;
use cgmath::{Vector3, Vector4};
use cgmath::{Matrix4, ToMatrix4, Matrix};

use opencl::hl::{Device, Context, CommandQueue, Kernel, Event};
use opencl::mem::CLBuffer;
use opencl::cl::{CL_MEM_READ_ONLY};

use snowmew::common::{Entity, Common, Duplicate, Delete};
use snowmew::input_integrator::InputIntegratorGameData;
use snowmew::debugger::DebuggerGameData;
use snowmew::table::{Dynamic, DynamicIterator};

const OPENCL_PROGRAM: &'static str = include_str!("position.c");

pub trait MatrixManager {
    fn size(&mut self, size: uint);
    fn set(&mut self, idx: uint, mat: Matrix4<f32>);
    fn get(&self, idx: uint) -> Matrix4<f32>;
}

impl<'r> MatrixManager for &'r mut Vec<Matrix4<f32>> {
    fn size(&mut self, size: uint) {
        if self.len() < size {
            let amount = self.len() - size;
            self.grow(amount, Matrix4::identity())
        }
    }
    fn set(&mut self, idx: uint, m: Matrix4<f32>) { self[idx] = m; }
    fn get(&self, idx: uint) -> Matrix4<f32> { self[idx] }
}


#[deriving(Encodable, Decodable)]
pub struct Delta {
    pub parent: Option<Entity>,
    pub delta: Decomposed<f32, Vector3<f32>, Quaternion<f32>>
}

impl Clone for Delta {
    fn clone(&self) -> Delta {
        Delta {
            parent: self.parent,
            delta: self.delta
        }
    }
}

impl Default for Delta {
    fn default() -> Delta {
        Delta {
            parent: None,
            delta: Transform::identity()
        }
    }
}

#[deriving(Clone, Encodable, Decodable)]
pub struct PositionData {
    delta: Dynamic<Delta>,
}

impl PositionData {
    pub fn new() -> PositionData {
        PositionData {
            delta: Dynamic::new()
        }
    }
}

pub trait Positions {
    fn get_position<'a>(&'a self) -> &'a PositionData;
    fn get_position_mut<'a>(&'a mut self) -> &'a mut PositionData;

    fn set_delta(&mut self,
                 key: Entity,
                 parent: Option<Entity>,
                 transform: Decomposed<f32, Vector3<f32>, Quaternion<f32>>) {
        self.get_position_mut().delta.insert(key,
            Delta {
                parent: parent,
                delta: transform
            }
        );
    }

    fn set_to_identity(&mut self, key: Entity) {
        self.set_delta(key, None, Transform::identity())
    }

    fn init_position(&mut self, key: Entity) {
        if self.get_position().delta.get(key).is_none() {
            self.set_to_identity(key);
        }
    }

    fn set_scale(&mut self, key: Entity, scale: f32) {
        self.init_position(key);
        self.get_position_mut()
            .delta.get_mut(key)
            .map(|d| d.delta.scale = scale);
    }

    fn set_displacement(&mut self, key: Entity, disp: Vector3<f32>) {
        self.init_position(key);
        self.get_position_mut()
            .delta.get_mut(key)
            .map(|d| d.delta.disp = disp);
    }

    fn set_rotation(&mut self, key: Entity, rot: Quaternion<f32>) {
        self.init_position(key);
        self.get_position_mut()
            .delta.get_mut(key)
            .map(|d| d.delta.rot = rot);
    }

    fn get_scale(&mut self, key: Entity) -> Option<&f32> {
        self.get_position()
            .delta.get(key)
            .map(|d| &d.delta.scale)
    }

    fn get_displacement(&mut self, key: Entity) -> Option<&Vector3<f32>> {
        self.get_position()
            .delta.get(key)
            .map(|d| &d.delta.disp)
    }

    fn get_rotation(&mut self, key: Entity) -> Option<&Quaternion<f32>> {
        self.get_position()
            .delta.get(key)
            .map(|d| &d.delta.rot)
    }

    fn get_mut_scale(&mut self, key: Entity) -> Option<&mut f32> {
        self.get_position_mut()
            .delta.get_mut(key)
            .map(|d| &mut d.delta.scale)
    }

    fn get_mut_displacement(&mut self, key: Entity) -> Option<&mut Vector3<f32>> {
        self.get_position_mut()
            .delta.get_mut(key)
            .map(|d| &mut d.delta.disp)
    }

    fn get_mut_rotation(&mut self, key: Entity) -> Option<&mut Quaternion<f32>> {
        self.get_position_mut()
            .delta.get_mut(key)
            .map(|d| &mut d.delta.rot)
    }

    fn get_transform(&self, key: Entity) -> Option<Decomposed<f32, Vector3<f32>, Quaternion<f32>>> {
        self.get_position()
            .delta.get(key)
            .map(|d| d.delta)
    }

    fn get_parent(&self, key: Entity) -> Option<&Option<Entity>> {
        self.get_position()
            .delta.get(key)
            .map(|d| &d.parent)
    }

    fn position(&self, key: Entity) -> Matrix4<f32> {
        self.get_position()
            .delta.get(key)
            .map(|d| {
                let matrix = d.delta.to_matrix4();
                match d.parent {
                    Some(p) => {
                        let parent = self.position(p);
                        parent.mul_m(&matrix)
                    }
                    None => {
                        matrix
                    }
                }
            })
            .unwrap_or_else(|| Matrix4::identity())
    }

    fn write_positions<MM: MatrixManager>(&self, mm: &mut MM) {
        mm.size(self.get_position().delta.highest_entity() as uint);
        for (key, _) in self.get_position().delta.iter() {
            mm.set(key as uint, self.position(key));
        }
    }

    fn position_iter(&self) -> PositionIter {
        let pos = self.get_position();
        PositionIter {
            pos: pos,
            iter: pos.delta.iter()
        }
    }

    /*fn write_positions_cl_vec4x4(&self, cq: &CommandQueue,
                        ctx: &mut CalcPositionsCl, out: &[CLBuffer<Vector4<f32>>, ..4]) -> Event {
        self.get_position().position.write_positions_cl_vec4x4(cq, ctx, out)
    }

    fn write_positions_cl_mat4(&self, cq: &CommandQueue,
                        ctx: &mut CalcPositionsCl, out: &[CLBuffer<Matrix4<f32>>]) -> Event {
        self.get_position().position.write_positions_cl_mat4(cq, ctx, out)
    }*/
}

pub struct PositionIter<'a> {
    pos: &'a PositionData,
    iter: DynamicIterator<'a, Delta>
}

impl<'a> Iterator<(Entity, Matrix4<f32>)> for PositionIter<'a> {
    fn next(&mut self) -> Option<(Entity, Matrix4<f32>)> {
        self.iter.next().map(|(id, _)| (id, self.pos.position(id)) )
    }
}

impl Duplicate for PositionData {
    fn duplicate(&mut self, src: Entity, dst: Entity) {
        let delta = self.delta.get(src).map(|&x| x);
        delta.map(|delta| self.delta.insert(dst, delta));
    }
}

impl Delete for PositionData {
    fn delete(&mut self, key: Entity) -> bool {
        self.delta.remove(key)
    }
}

impl Positions for PositionData {
    fn get_position<'a>(&'a self) -> &'a PositionData { self }
    fn get_position_mut<'a>(&'a mut self) -> &'a mut PositionData { self }
}

impl<T: Positions> Positions for InputIntegratorGameData<T> {
    fn get_position<'a>(&'a self) -> &'a PositionData { self.inner.get_position() }
    fn get_position_mut<'a>(&'a mut self) -> &'a mut PositionData { self.inner.get_position_mut() }
}

impl<T: Positions> Positions for DebuggerGameData<T> {
    fn get_position<'a>(&'a self) -> &'a PositionData { self.inner.get_position() }
    fn get_position_mut<'a>(&'a mut self) -> &'a mut PositionData { self.inner.get_position_mut() }
}
