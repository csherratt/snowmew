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
#![crate_type = "lib"]


extern crate "snowmew-core" as snowmew;
extern crate cgmath;
#[cfg(feature="use_opencl")]
extern crate opencl;
extern crate collect;
extern crate "rustc-serialize" as rustc_serialize;

use std::default::Default;
use std::cmp::max;

use cgmath::{Transform, Decomposed, Vector3, Matrix4, ToMatrix4, Matrix, Quaternion};
use collect::iter::OrderedMapIterator;

use snowmew::common::{Entity, Duplicate, Delete};
use snowmew::table::{Static, StaticIterator};

pub trait MatrixManager {
    fn size(&mut self, size: usize);
    fn set(&mut self, idx: usize, mat: Matrix4<f32>);
    fn get(&self, idx: usize) -> Matrix4<f32>;
}

impl<'r> MatrixManager for Vec<Matrix4<f32>> {
    fn size(&mut self, size: usize) {
        use std::iter::repeat;

        if self.len() < size {
            let amount = self.len() - size;
            self.extend(repeat(Matrix4::identity()).take(amount));
        }
    }
    fn set(&mut self, idx: usize, m: Matrix4<f32>) { self[idx] = m; }
    fn get(&self, idx: usize) -> Matrix4<f32> { self[idx] }
}


#[derive(RustcEncodable, RustcDecodable, Copy)]
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

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct PositionData {
    max: Entity,
    delta: Static<Delta>,
}

impl PositionData {
    pub fn new() -> PositionData {
        PositionData {
            max: 0,
            delta: Static::new()
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
        self.get_position_mut().max = max(key, self.get_position().max);
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

    fn set_parent(&mut self, key: Entity, parent: Option<Entity>) {
        self.init_position(key);
        self.get_position_mut()
            .delta.get_mut(key)
            .map(|d| d.parent = parent);
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

    fn write_positions(&self, mm: &mut MatrixManager) {
        mm.size(self.position_max());
        for (key, _) in self.get_position().delta.iter() {
            mm.set(key as usize, self.position(key));
        }
    }

    fn delta_iter(&self) -> StaticIterator<Delta> {
        self.get_position().delta.iter()
    }

    fn position_iter(&self) -> PositionIter {
        let pos = self.get_position();
        PositionIter {
            pos: pos,
            iter: pos.delta.iter()
        }
    }

    fn position_max(&self) -> usize { self.get_position().max as usize + 1 }
}

pub struct PositionIter<'a> {
    pos: &'a PositionData,
    iter: StaticIterator<'a, Delta>
}

impl<'a> Iterator for PositionIter<'a> {
    type Item = (Entity, Matrix4<f32>);

    fn next(&mut self) -> Option<(Entity, Matrix4<f32>)> {
        self.iter.next().map(|(id, _)| (id, self.pos.position(id)) )
    }
}

impl<'a> OrderedMapIterator<Entity, Matrix4<f32>> for PositionIter<'a> {}


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

#[cfg(feature="use_opencl")]
pub mod cl {
    use cgmath::{Transform, Decomposed, Vector3, Vector4, Matrix4, Quaternion};

    use opencl::hl::{Device, Context, CommandQueue, Kernel, Event};
    use opencl::mem::CLBuffer;
    use opencl::cl::{CL_MEM_READ_ONLY};

    use super::Positions;

    const OPENCL_PROGRAM: &'static str = include_str!("position.c");

    #[derive(Copy)]
    pub struct Delta {
        delta: Decomposed<f32, Vector3<f32>, Quaternion<f32>>
    }

    impl Clone for Delta {
        fn clone(&self) -> Delta {
            Delta {
                delta: self.delta
            }
        }
    }

    pub struct Accelerator {
        kernel_vec4: Kernel,
        kernel_mat: Kernel,
        input: CLBuffer<Delta>,
        input_buf: Vec<Delta>,
        parent: CLBuffer<u32>,
        parent_buf: Vec<u32>
    }

    impl Accelerator {
        pub fn new(ctx: &Context, device: &Device) -> Accelerator {
            use std::iter::repeat;

            let program = ctx.create_program_from_source(OPENCL_PROGRAM);
            match program.build(device) {
                Ok(_) => (),
                Err(build_log) => {
                    println!("Error building program:");
                    println!("{}", build_log);
                    panic!("");
                }
            }

            let kernel_mat = program.create_kernel("calc_mat");
            let kernel_vec4 = program.create_kernel("calc_vec4");
            let delta_mem = ctx.create_buffer(1024*1024, CL_MEM_READ_ONLY);
            let delta_buf: Vec<Delta> =
                repeat(Delta {delta: Transform::identity()}).take(1024*1024).collect();
            let parent_buf: Vec<u32> =
                repeat(0u32).take(1024*1024).collect();
            let parent_mem = ctx.create_buffer(1024*1024, CL_MEM_READ_ONLY);

            Accelerator {
                kernel_vec4: kernel_vec4,
                kernel_mat: kernel_mat,
                input: delta_mem,
                input_buf: delta_buf,
                parent: parent_mem,
                parent_buf: parent_buf
            }
        }

        fn write<P: Positions>(&mut self, pos: &P) -> usize {
            let mut top = 0;
            for i in (0..pos.position_max()) {
                self.parent_buf[i] = !0;
            }
            for (idx, &p) in pos.delta_iter() {
                top = idx;
                self.parent_buf[idx as usize] = p.parent.unwrap_or(!0);
                self.input_buf[idx as usize] = Delta {
                    delta: p.delta
                };
            }
            top as usize
        }

        pub fn compute_mat<P: Positions>(&mut self,
                                      pos: &P,
                                      queue: &CommandQueue,
                                      buf: &CLBuffer<Matrix4<f32>>) -> Event {
            self.write(pos);
            let max = pos.position_max();
            let event = [queue.write_async(&self.input, &&self.input_buf[0..max], ()),
                         queue.write_async(&self.parent, &&self.parent_buf[0..max], ())];
            self.kernel_mat.set_arg(0, &self.input);
            self.kernel_mat.set_arg(1, &self.parent);
            self.kernel_mat.set_arg(2, buf);
            self.kernel_mat.set_arg(3, &(pos.position_max() as u32));
            queue.enqueue_async_kernel(&self.kernel_mat, pos.position_max(), None, &event)
        }

        pub fn compute_vec4x4<P: Positions>(&mut self,
                                        pos: &P,
                                        queue: &CommandQueue,
                                        buf: &[CLBuffer<Vector4<f32>>; 4]) -> Event {
            self.write(pos);
            let max = pos.position_max();
            let event = [queue.write_async(&self.input, &&self.input_buf[0..max], ()),
                         queue.write_async(&self.parent, &&self.parent_buf[0..max], ())];
            self.kernel_vec4.set_arg(0, &self.input);
            self.kernel_vec4.set_arg(1, &self.parent);
            self.kernel_vec4.set_arg(2, &buf[0]);
            self.kernel_vec4.set_arg(3, &buf[1]);
            self.kernel_vec4.set_arg(4, &buf[2]);
            self.kernel_vec4.set_arg(5, &buf[3]);
            self.kernel_vec4.set_arg(6, &(max as u32));
            queue.enqueue_async_kernel(&self.kernel_vec4, pos.position_max(), None, &event)
        }
    }
}

