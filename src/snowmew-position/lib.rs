#![crate_id = "github.com/csherratt/snowmew#snowmew-position:0.1"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "A position manager for snowmew"]

extern crate snowmew;
extern crate cgmath;
extern crate OpenCL;
extern crate cow;

use std::default::Default;

use cgmath::transform::Transform3D;
use cgmath::quaternion::Quaternion;
use cgmath::vector::{Vector3, Vector4};
use cgmath::matrix::{Matrix4, ToMatrix4, Matrix};

use OpenCL::hl::{Device, Context, CommandQueue, Program, Kernel, Event};
use OpenCL::mem::CLBuffer;
use OpenCL::CL::{CL_MEM_READ_ONLY};

use cow::btree::{BTreeMap, BTreeMapIterator};

use snowmew::common::{ObjectKey, Common};

static opencl_program: &'static str = include_str!("position.c");

pub struct Delta {
    delta : Transform3D<f32>,
    parent: u32,
}

impl Default for Delta {
    fn default() -> Delta {
        Delta {
            parent: 0,
            delta: Transform3D::new(1f32, Quaternion::zero(), Vector3::zero()),
        }
    }
}

impl Clone for Delta {
    fn clone(&self) -> Delta {
        let tras = self.delta.get();
        Delta {
            parent: self.parent.clone(),
            delta: Transform3D::new(tras.scale.clone(),
                                    tras.rot.clone(),
                                    tras.disp.clone()),
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

#[deriving(Clone, Default, Eq, TotalOrd, TotalEq, Ord)]
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
        let (gen_offset, _) = *self.gen.get(gen as uint);

        (gen_offset + offset) as uint
    }

    fn add_location(&mut self, gen: u32) -> u32 {
        // create a new generation if this is the first in it
        if gen as uint == self.gen.len() {
            let (s, len) = *self.gen.get((gen-1) as uint);
            self.gen.push((s+len, 1));

            self.delta.insert(gen, BTreeMap::new());

            0
        } else {
            // increment the starting index of each generation before ours
            for t in self.gen.mut_slice_from((gen+1) as uint).mut_iter() {
                let (off, len) = *t;
                *t = (off+1, len);
            }

            let (off, len) = *self.gen.get(gen as uint);
            *self.gen.get_mut(gen as uint) = (off, len+1);

            len
        }
    }

    pub fn insert(&mut self, parent: Id, delta: Transform3D<f32>) -> Id {
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
            None => fail!("there was no delta! {:?}", gen)
        };

        Id(gen+1, id)
    }

    pub fn update(&mut self, id: Id, delta: Transform3D<f32>) {
        let Id(gen, id) = id;
        self.delta.find_mut(&gen).unwrap().find_mut(&id).unwrap().delta = delta;
    }

    pub fn get_delta(&self, id :Id) -> Transform3D<f32> {
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

    pub fn write_positions_cl(&self, cq: &CommandQueue, ctx: &mut CalcPositionsCl,
                           out: &[CLBuffer<Vector4<f32>>, ..4]) -> Event {

        cq.map_mut(&ctx.input, (), |out_delta| {
        cq.map_mut(&ctx.parent, (), |out_parent| {
            for (&(gen_off, _), (_, gen)) in self.gen.iter().zip(self.delta.iter()) {
                for (off, delta) in gen.iter() {
                    out_delta[(off + gen_off) as uint] = delta.delta;
                    out_parent[(off + gen_off) as uint] = delta.parent.clone();
                }
            }
        })});

        // write init value
        ctx.init_kernel.set_arg(0, &out[0]);
        ctx.init_kernel.set_arg(1, &out[1]);
        ctx.init_kernel.set_arg(2, &out[2]);
        ctx.init_kernel.set_arg(3, &out[3]);
        let mut event = cq.enqueue_async_kernel(&ctx.init_kernel, 1, None, ());

        // run the kernel across the deltas 
        ctx.kernel.set_arg(0, &ctx.input);
        ctx.kernel.set_arg(1, &ctx.parent);
        ctx.kernel.set_arg(2, &out[0]);
        ctx.kernel.set_arg(3, &out[1]);
        ctx.kernel.set_arg(4, &out[2]);
        ctx.kernel.set_arg(5, &out[3]);
        for idx in range(1, self.gen.len()) {
            let (off, _) = *self.gen.get(idx-1);
            ctx.kernel.set_arg(6, &off);
            let (off2, len) = *self.gen.get(idx);
            ctx.kernel.set_arg(7, &off2);
            event = cq.enqueue_async_kernel(&ctx.kernel, len as uint, None, event);
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
        let (gen_offset, _) = *self.gen.get(gen as uint);

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
        let (gen_offset, _) = *self.gen.get(gen as uint);

        (gen_offset + offset) as uint
    }
}

pub struct CalcPositionsCl {
    program: Program,
    kernel: Kernel,
    init_kernel: Kernel,
    input: CLBuffer<Transform3D<f32>>,
    parent: CLBuffer<u32>,
}

impl CalcPositionsCl {
    pub fn new(ctx: &Context, device: &Device) -> CalcPositionsCl {
        let program = ctx.create_program_from_source(opencl_program);
    
        match program.build(device) {
            Ok(()) => (),
            Err(build_log) => {
                println!("Error building program:");
                println!("{:s}", build_log);
                fail!("");
            }
        }

        let kernel = program.create_kernel("calc_gen");
        let init_kernel = program.create_kernel("set_idenity");
        let delta_mem = ctx.create_buffer(1024*1024, CL_MEM_READ_ONLY);
        let parent = ctx.create_buffer(1024*1024, CL_MEM_READ_ONLY);

        CalcPositionsCl {
            program: program,
            kernel: kernel,
            init_kernel: init_kernel,
            input: delta_mem,
            parent: parent
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
            let id = self.get_position_mut().position.insert(pid,
                Transform3D::new(1f32, Quaternion::identity(), Vector3::new(0f32, 0f32, 0f32))
            );
            self.get_position_mut().location.insert(key, id);
            id
        }
    }

    fn update_location(&mut self, key: ObjectKey, location: Transform3D<f32>) {
        let id = self.position_id(key);
        self.get_position_mut().position.update(id, location);
    }

    fn location(&self, key: ObjectKey) -> Option<Transform3D<f32>> {
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
            Some(t) => {t.get().to_matrix4()},
            None => Matrix4::identity()
        };
        p_mat.mul_m(&loc)
    }

    fn write_positions<MM: MatrixManager>(&self, mm: &mut MM) {
        self.get_position().position.write_positions(mm)
    }

    fn write_positions_cl(&self, cq: &CommandQueue,
                        ctx: &mut CalcPositionsCl, out: &[CLBuffer<Vector4<f32>>, ..4]) -> Event {
        self.get_position().position.write_positions_cl(cq, ctx, out)
    }

    fn compute_positions(&self) -> ComputedPosition {
        self.get_position().position.compute_positions()
    }

    fn location_iter<'a>(&'a self) -> BTreeMapIterator<'a, ObjectKey, Id> {
        self.get_position().location.iter()
    }

    fn position_count(&self) -> uint {
        let last = self.get_position().position.gen.len();
        let (s, l) = *self.get_position().position.gen.get(last-1);
        (s + l) as uint
    }
}