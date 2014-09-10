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

extern crate "snowmew-core" as snowmew;
extern crate cgmath;
extern crate opencl;
extern crate cow;
extern crate time;

use std::default::Default;

use cgmath::{Transform, Decomposed};
use cgmath::Quaternion;
use cgmath::{Vector3, Vector4};
use cgmath::{Matrix4, ToMatrix4, Matrix};

use opencl::hl::{Device, Context, CommandQueue, Kernel, Event};
use opencl::mem::CLBuffer;
use opencl::CL::{CL_MEM_READ_ONLY};

use cow::btree::{BTreeMap, BTreeMapIterator};

use snowmew::common::{ObjectKey, Common};

static opencl_program: &'static str = include_str!("position.c");

pub struct Delta {
    delta : Decomposed<f32, Vector3<f32>, Quaternion<f32>>,
    parent: u32,
}

impl Default for Delta {
    fn default() -> Delta {
        Delta {
            parent: 0,
            delta: Transform::identity()
        }
    }
}

impl Clone for Delta {
    fn clone(&self) -> Delta {
        Delta {
            parent: self.parent.clone(),
            delta: Decomposed{scale: self.delta.scale.clone(),
                              rot:   self.delta.rot.clone(),
                              disp:  self.delta.disp.clone()}
        }
    }
}

pub trait MatrixManager {
    fn set(&mut self, idx: uint, mat: Matrix4<f32>);
    fn get(&self, idx: uint) -> Matrix4<f32>;
}

impl<'r> MatrixManager for &'r mut [Matrix4<f32>] {
    fn set(&mut self, idx: uint, m: Matrix4<f32>) { self[idx] = m; }
    fn get(&self, idx: uint) -> Matrix4<f32> { self[idx] }
}

#[deriving(Clone, Default)]
pub struct Deltas {
    gen: Vec<(u32, u32)>,
    delta: BTreeMap<u32, BTreeMap<u32, Delta>>,
}

#[deriving(Clone, Default, Eq, PartialOrd, PartialEq, Ord)]
pub struct Id(u32, u32);

impl Deltas {
    pub fn new() -> Deltas {
        let mut b = BTreeMap::new();
        b.insert(0u32, BTreeMap::new());
        b.find_mut(&0).unwrap().insert(0u32, Default::default());

        Deltas {
            gen: vec!((0, 1)),
            delta: b,
        }
    }

    pub fn root() -> Id { Id(0, 0) }

    pub fn get_loc(&self, id: Id) -> uint {
        let Id(gen, offset) = id;
        let (gen_offset, _) = self.gen[gen as uint];

        (gen_offset + offset) as uint
    }

    fn add_location(&mut self, gen: u32) -> u32 {
        // create a new generation if this is the first in it
        if gen as uint == self.gen.len() {
            let (s, len) = self.gen[(gen-1) as uint];
            self.gen.push((s+len, 1));

            self.delta.insert(gen, BTreeMap::new());

            0
        } else {
            // increment the starting index of each generation before ours
            for t in self.gen.mut_slice_from((gen+1) as uint).mut_iter() {
                let (off, len) = *t;
                *t = (off+1, len);
            }

            let (off, len) = self.gen[gen as uint];
            *self.gen.get_mut(gen as uint) = (off, len+1);

            len
        }
    }

    pub fn insert(&mut self, parent: Id, delta: Decomposed<f32, Vector3<f32>, Quaternion<f32>>) -> Id {
        let Id(gen, pid) = parent;

        assert!((gen as uint) < self.gen.len());

        let id = self.add_location(gen+1);
        match self.delta.find_mut(&(gen+1)) {
            Some(d) => {
                d.insert(id, Delta {
                    parent: pid,
                    delta: delta,
                })
            }
            None => fail!("there was no delta! {}", gen)
        };

        Id(gen+1, id)
    }

    pub fn get_mut<'a>(&'a mut self, id: Id) -> &'a mut Decomposed<f32, Vector3<f32>, Quaternion<f32>> {
        let Id(gen, id) = id;
        &mut self.delta.find_mut(&gen).unwrap().find_mut(&id).unwrap().delta
    }

    pub fn update(&mut self, id: Id, delta: Decomposed<f32, Vector3<f32>, Quaternion<f32>>) {
        *self.get_mut(id) = delta;
    }

    pub fn get_delta(&self, id :Id) -> Decomposed<f32, Vector3<f32>, Quaternion<f32>> {
        let Id(gen, id) = id;
        self.delta.find(&gen).unwrap().find(&id).unwrap().delta
    }

    pub fn get_mat(&self, id :Id) -> Matrix4<f32> {
        match id {
            Id(0, key) => {
                self.delta.find(&0).unwrap().find(&key).unwrap().delta.to_matrix4()
            },
            Id(gen, key) => {
                let cell = self.delta.find(&gen).unwrap().find(&key).unwrap();
                let mat = cell.delta.to_matrix4();
                let parent = Id(gen-1, cell.parent);

                self.get_mat(parent).mul_m(&mat)
            }
        }
    }

    #[inline(never)]
    pub fn write_positions<MM: MatrixManager>(&self, mm: &mut MM) {
        let mut last_gen_off = 0;
        mm.set(0, Matrix4::identity());

        for (&(gen_off, _), (_, gen)) in self.gen.iter().zip(self.delta.iter()) {
            for (off, delta) in gen.iter() {
                let ploc = last_gen_off + delta.parent;
                let nmat = mm.get(ploc as uint).mul_m(&delta.delta.to_matrix4());
                mm.set((off + gen_off) as uint, nmat);
            }
            last_gen_off = gen_off;
        }
    }

    pub fn write_positions_cl_vec4x4(&self, cq: &CommandQueue, ctx: &mut CalcPositionsCl,
                           out: &[CLBuffer<Vector4<f32>>, ..4]) -> Event {

        let last = self.gen.len();
        let (s, l) = self.gen[last-1];
        let size = s + l;

        unsafe {
            ctx.parent_buffer.reserve(size as uint);
            ctx.parent_buffer.set_len(size as uint);
            ctx.input_buffer.reserve(size as uint);
            ctx.input_buffer.set_len(size as uint);
        }

        for (&(gen_off, _), (_, gen)) in self.gen.iter().zip(self.delta.iter()) {
            for (off, delta) in gen.iter() {
                *ctx.input_buffer.get_mut((off + gen_off) as uint) = delta.delta;
                *ctx.parent_buffer.get_mut((off + gen_off) as uint) = delta.parent.clone();
            }
        }

        let events = &[cq.write_async(&ctx.input, &ctx.input_buffer.as_slice(), ()),
                       cq.write_async(&ctx.parent, &ctx.parent_buffer.as_slice(), ())];

        // write init value
        ctx.init_kernel_vec4.set_arg(0, &out[0]);
        ctx.init_kernel_vec4.set_arg(1, &out[1]);
        ctx.init_kernel_vec4.set_arg(2, &out[2]);
        ctx.init_kernel_vec4.set_arg(3, &out[3]);
        let mut event = cq.enqueue_async_kernel(&ctx.init_kernel_vec4, 1i, None, events.as_slice());

        // run the kernel across the deltas 
        ctx.kernel_vec4.set_arg(0, &ctx.input);
        ctx.kernel_vec4.set_arg(1, &ctx.parent);
        ctx.kernel_vec4.set_arg(2, &out[0]);
        ctx.kernel_vec4.set_arg(3, &out[1]);
        ctx.kernel_vec4.set_arg(4, &out[2]);
        ctx.kernel_vec4.set_arg(5, &out[3]);
        for idx in range(1, self.gen.len()) {
            let (off, _) = self.gen[idx-1];
            ctx.kernel_vec4.set_arg(6, &off);
            let (off2, len) = self.gen[idx];
            ctx.kernel_vec4.set_arg(7, &off2);
            event = cq.enqueue_async_kernel(&ctx.kernel_vec4, len as uint, None, event);
        }

        event
    }

    pub fn write_positions_cl_mat4(&self, cq: &CommandQueue, ctx: &mut CalcPositionsCl,
                           out: &[CLBuffer<Matrix4<f32>>]) -> Event {

        let last = self.gen.len();
        let (s, l) = self.gen[last-1];
        let size = s + l;

        unsafe {
            ctx.parent_buffer.reserve(size as uint);
            ctx.parent_buffer.set_len(size as uint);
            ctx.input_buffer.reserve(size as uint);
            ctx.input_buffer.set_len(size as uint);
        }

        for (&(gen_off, _), (_, gen)) in self.gen.iter().zip(self.delta.iter()) {
            for (off, delta) in gen.iter() {
                *ctx.input_buffer.get_mut((off + gen_off) as uint) = delta.delta;
                *ctx.parent_buffer.get_mut((off + gen_off) as uint) = delta.parent.clone();
            }
        }

        let events = &[cq.write_async(&ctx.input, &ctx.input_buffer.as_slice(), ()),
                       cq.write_async(&ctx.parent, &ctx.parent_buffer.as_slice(), ())];

        // write init value
        ctx.init_kernel_mat.set_arg(0, &out[0]);
        let mut event = cq.enqueue_async_kernel(&ctx.init_kernel_mat, 1i, None, events.as_slice());

        // run the kernel across the deltas 
        ctx.kernel_mat.set_arg(0, &ctx.input);
        ctx.kernel_mat.set_arg(1, &ctx.parent);
        ctx.kernel_mat.set_arg(2, &out[0]);
        for idx in range(1, self.gen.len()) {
            let (off, _) = self.gen[idx-1];
            ctx.kernel_mat.set_arg(3, &off);
            let (off2, len) = self.gen[idx];
            ctx.kernel_mat.set_arg(4, &off2);
            event = cq.enqueue_async_kernel(&ctx.kernel_mat, len as uint, None, event);
        }

        event
    }

    pub fn to_positions_gl(&self, out_delta: &mut [Delta]) -> ComputedPositionGL {
        for (&(gen_off, _), (_, gen)) in self.gen.iter().zip(self.delta.iter()) {
            for (off, delta) in gen.iter() {
                out_delta[(off + gen_off) as uint] = delta.clone();
            }
        }

        ComputedPositionGL {
            gen: self.gen.clone()
        }
    }

    pub fn compute_positions(&self) -> ComputedPosition {
        ComputedPosition {
            gen: self.gen.clone()
        }
    }
}

pub struct ComputedPositionGL {
    pub gen: Vec<(u32, u32)>
}

impl ComputedPositionGL {
    pub fn get_loc(&self, id: Id) -> uint {
        let Id(gen, offset) = id;
        let (gen_offset, _) = self.gen[gen as uint];

        (gen_offset + offset) as uint
    }
}

pub struct ComputedPosition {
    gen: Vec<(u32, u32)>
}

impl Clone for ComputedPosition {
    fn clone(&self) -> ComputedPosition {
        ComputedPosition {
            gen: self.gen.clone(),
        }
    }
}

impl ComputedPosition {
    pub fn root() -> Id { Id(0, 0) }

    pub fn get_loc(&self, id: Id) -> uint {
        let Id(gen, offset) = id;
        let (gen_offset, _) = self.gen[gen as uint];

        (gen_offset + offset) as uint
    }
}

pub struct CalcPositionsCl {
    kernel_vec4: Kernel,
    init_kernel_vec4: Kernel,
    kernel_mat: Kernel,
    init_kernel_mat: Kernel,
    input_buffer: Vec<Decomposed<f32, Vector3<f32>, Quaternion<f32>>>,
    input: CLBuffer<Decomposed<f32, Vector3<f32>, Quaternion<f32>>>,
    parent_buffer: Vec<u32>,
    parent: CLBuffer<u32>,
}

impl CalcPositionsCl {
    pub fn new(ctx: &Context, device: &Device) -> CalcPositionsCl {
        let program = ctx.create_program_from_source(opencl_program);
    
        match program.build(device) {
            Ok(_) => (),
            Err(build_log) => {
                println!("Error building program:");
                println!("{:s}", build_log);
                fail!("");
            }
        }


        let kernel_mat = program.create_kernel("calc_gen_mat");
        let init_kernel_mat = program.create_kernel("set_idenity_mat");
        let kernel_vec4 = program.create_kernel("calc_gen_vec4");
        let init_kernel_vec4 = program.create_kernel("set_idenity_vec4");
        let delta_mem = ctx.create_buffer(1024*1024, CL_MEM_READ_ONLY);
        let parent = ctx.create_buffer(1024*1024, CL_MEM_READ_ONLY);

        CalcPositionsCl {
            kernel_vec4: kernel_vec4,
            init_kernel_vec4: init_kernel_vec4,
            kernel_mat: kernel_mat,
            init_kernel_mat: init_kernel_mat,
            input: delta_mem,
            input_buffer: Vec::new(),
            parent: parent,
            parent_buffer: Vec::new()
        }
    }
}

#[deriving(Clone)]
pub struct PositionData {
    location: BTreeMap<ObjectKey, Id>,
    position: Deltas
}

impl PositionData {
    pub fn new() -> PositionData {
        PositionData {
            location: BTreeMap::new(),
            position: Deltas::new()
        }
    }
}

pub trait Positions: Common {
    fn get_position<'a>(&'a self) -> &'a PositionData;
    fn get_position_mut<'a>(&'a mut self) -> &'a mut PositionData;

    fn position_id(&mut self, key: ObjectKey) -> Id {
        if key == 0 {
            Deltas::root()
        } else {
            match self.get_position().location.find(&key) {
                Some(id) =>  return *id,
                None => ()
            }

            let poid = self.object(key).unwrap().parent;
            let pid = self.position_id(poid);
            let id = self.get_position_mut().position.insert(pid, Transform::identity());
            self.get_position_mut().location.insert(key, id);
            id
        }
    }

    fn update_location(&mut self, key: ObjectKey, location: Decomposed<f32, Vector3<f32>, Quaternion<f32>>) {
        let id = self.position_id(key);
        self.get_position_mut().position.update(id, location);
    }

    fn set_to_identity(&mut self, key: ObjectKey) {
        let id = self.position_id(key);
        self.get_position_mut().position.update(id, Transform::identity());
    }

    fn set_scale(&mut self, key: ObjectKey, scale: f32) {
        let id = self.position_id(key);
        self.get_position_mut().position.get_mut(id).scale = scale;
    }

    fn set_displacement(&mut self, key: ObjectKey, disp: Vector3<f32>) {
        let id = self.position_id(key);
        self.get_position_mut().position.get_mut(id).disp = disp;
    }

    fn set_rotation(&mut self, key: ObjectKey, rot: Quaternion<f32>) {
        let id = self.position_id(key);
        self.get_position_mut().position.get_mut(id).rot = rot;
    }

    fn location(&self, key: ObjectKey) -> Option<Decomposed<f32, Vector3<f32>, Quaternion<f32>>> {
        match self.get_position().location.find(&key) {
            Some(id) => Some(self.get_position().position.get_delta(*id)),
            None => None
        }
    }

    fn position(&self, oid: ObjectKey) -> Matrix4<f32> {
        let obj = self.object(oid);
        let p_mat = match obj {
            Some(obj) => self.position(obj.parent),
            None => Matrix4::identity()
        };

        let loc = match self.location(oid) {
            Some(t) => {t.to_matrix4()},
            None => Matrix4::identity()
        };
        p_mat.mul_m(&loc)
    }

    fn write_positions<MM: MatrixManager>(&self, mm: &mut MM) {
        self.get_position().position.write_positions(mm)
    }

    fn write_positions_cl_vec4x4(&self, cq: &CommandQueue,
                        ctx: &mut CalcPositionsCl, out: &[CLBuffer<Vector4<f32>>, ..4]) -> Event {
        self.get_position().position.write_positions_cl_vec4x4(cq, ctx, out)
    }

    fn write_positions_cl_mat4(&self, cq: &CommandQueue,
                        ctx: &mut CalcPositionsCl, out: &[CLBuffer<Matrix4<f32>>]) -> Event {
        self.get_position().position.write_positions_cl_mat4(cq, ctx, out)
    }

    fn compute_positions(&self) -> ComputedPosition {
        self.get_position().position.compute_positions()
    }

    fn location_iter<'a>(&'a self) -> BTreeMapIterator<'a, ObjectKey, Id> {
        self.get_position().location.iter()
    }

    fn position_count(&self) -> uint {
        let last = self.get_position().position.gen.len();
        let (s, l) = self.get_position().position.gen[last-1];
        (s + l) as uint
    }
}
