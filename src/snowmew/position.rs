use std::default::Default;

use cgmath::transform::Transform3D;
use cgmath::quaternion::Quat;
use cgmath::vector::{Vec3, Vec4};
use cgmath::matrix::{Mat4, ToMat4, Matrix};

use OpenCL::hl::{Device, Context, CommandQueue, Program, Kernel, EventList, Event};
use OpenCL::mem::CLBuffer;
use OpenCL::CL::{CL_MEM_READ_ONLY};

use cow::btree::{BTreeMap, BTreeMapIterator};

use core::{ObjectKey, Common};


static opencl_program: &'static str = "
struct q4 {
    float s, x, y, z;
};

struct f4 {
    float x, y, z, w;
};

struct f3 {
    float x, y, z;
};

typedef struct q4 q4;
typedef struct f4 f4;
typedef struct f3 f3;

struct mat4 {
    f4 x, y, z, w;
};

struct transform
{
    q4 rot;
    f3 pos;
    float scale;
    int parent;
    int padd[3];
};

typedef struct mat4 Mat4;
typedef struct transform Transform3D;

#define DOT(OUT, A, B, i, j) \
    OUT.j.i = A.x.i * B.j.x + \
    A.y.i * B.j.y + \
    A.z.i * B.j.z + \
    A.w.i * B.j.w

Mat4
mult_m(const Mat4 a, const Mat4 b)
{
    Mat4 out;

    DOT(out, a, b, x, x);
    DOT(out, a, b, x, y);
    DOT(out, a, b, x, z);
    DOT(out, a, b, x, w);

    DOT(out, a, b, y, x);
    DOT(out, a, b, y, y);
    DOT(out, a, b, y, z);
    DOT(out, a, b, y, w);

    DOT(out, a, b, z, x);
    DOT(out, a, b, z, y);
    DOT(out, a, b, z, z);
    DOT(out, a, b, z, w);

    DOT(out, a, b, w, x);
    DOT(out, a, b, w, y);
    DOT(out, a, b, w, z);
    DOT(out, a, b, w, w);

    return out;
}

Mat4
transform_to_mat4(global Transform3D *trans)
{
    Mat4 mat;

    float x2 = trans->rot.x + trans->rot.x;
    float y2 = trans->rot.y + trans->rot.y;
    float z2 = trans->rot.z + trans->rot.z;

    float xx2 = x2 * trans->rot.x;
    float xy2 = x2 * trans->rot.y;
    float xz2 = x2 * trans->rot.z;

    float yy2 = y2 * trans->rot.y;
    float yz2 = y2 * trans->rot.z;
    float zz2 = z2 * trans->rot.z;

    float sy2 = y2 * trans->rot.s;
    float sz2 = z2 * trans->rot.s;
    float sx2 = x2 * trans->rot.s;

    mat.x.x = (1. - yy2 - zz2) * trans->scale;
    mat.x.y = (xy2 + sz2) * trans->scale;
    mat.x.z = (xz2 - sy2) * trans->scale;
    mat.x.w = 0.;

    mat.y.x = (xy2 - sz2) * trans->scale;
    mat.y.y = (1. - xx2 - zz2) * trans->scale;
    mat.y.z = (yz2 + sx2) * trans->scale;
    mat.y.w = 0.;

    mat.z.x = (xz2 + sy2) * trans->scale;
    mat.z.y = (yz2 - sx2) * trans->scale;
    mat.z.z = (1. - xx2 - yy2) * trans->scale;
    mat.z.w = 0.;

    mat.w.x = trans->pos.x;
    mat.w.y = trans->pos.y;
    mat.w.z = trans->pos.z;
    mat.w.w = 1.;

    return mat;
}

Mat4
get_mat4(global f4* x, global f4* y, global f4* z, global f4* w, uint idx)
{
    Mat4 mat;
    mat.x = x[idx];
    mat.y = y[idx];
    mat.z = z[idx];
    mat.w = w[idx];
    return mat;
}

void
set_mat4(global f4* x, global f4* y, global f4* z, global f4* w, uint idx, Mat4 mat)
{
    x[idx] = mat.x;
    y[idx] = mat.y;
    z[idx] = mat.z;
    w[idx] = mat.w;
}

kernel void
calc_gen(global Transform3D *t,
         global f4* x, global f4* y, global f4* z, global f4* w,
         int offset_last, int offset_this)
{
    int id = get_global_id(0);
    global Transform3D *trans = &t[offset_this + id];
    Mat4 mat = transform_to_mat4(trans);
    Mat4 parent = get_mat4(x, y, z, w, offset_last+trans->parent);
    Mat4 result = mult_m(parent, mat);
    set_mat4(x, y, z, w, offset_this + id, result);
}

kernel void
set_idenity(global f4* x, global f4* y, global f4* z, global f4* w) {
    x[0].x = (float)1; x[0].y = (float)0; x[0].z = (float)0; x[0].w = (float)0;
    y[0].x = (float)0; y[0].y = (float)1; y[0].z = (float)0; y[0].w = (float)0;
    z[0].x = (float)0; z[0].y = (float)0; z[0].z = (float)1; z[0].w = (float)0;
    w[0].x = (float)0; w[0].y = (float)0; w[0].z = (float)0; w[0].w = (float)1;
}
";

pub struct Delta {
    delta : Transform3D<f32>,
    parent: u32,
    padd: [u32, ..3]
}

impl Default for Delta {
    fn default() -> Delta {
        Delta {
            parent: 0,
            delta: Transform3D::new(1f32, Quat::zero(), Vec3::zero()),
            padd: [0, 0, 0]
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
            padd: [0, 0, 0]
        }
    }
}

pub trait MatrixManager {
    fn set(&mut self, idx: uint, mat: Mat4<f32>);
    fn get(&self, idx: uint) -> Mat4<f32>;
}

impl<'r> MatrixManager for &'r mut [Mat4<f32>] {
    fn set(&mut self, idx: uint, m: Mat4<f32>) { self[idx] = m; }
    fn get(&self, idx: uint) -> Mat4<f32> { self[idx] }
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

    pub fn root() -> Id {Id(0, 0)}

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
                    padd: [0, 0, 0]
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

    pub fn get_mat(&self, id :Id) -> Mat4<f32> {
        match id {
            Id(0, key) => {
                self.delta.find(&0).unwrap().find(&key).unwrap().delta.to_mat4()
            },
            Id(gen, key) => {
                let cell = self.delta.find(&gen).unwrap().find(&key).unwrap();
                let mat = cell.delta.to_mat4();
                let parent = Id(gen-1, cell.parent);

                self.get_mat(parent).mul_m(&mat)
            }
        }
    }

    #[inline(never)]
    pub fn to_positions<MM: MatrixManager>(&self, mm: &mut MM) -> ComputedPosition {
        let mut last_gen_off = 0;
        mm.set(0, Mat4::identity());

        for (&(gen_off, _), (_, gen)) in self.gen.iter().zip(self.delta.iter()) {
            for (off, delta) in gen.iter() {
                let ploc = last_gen_off + delta.parent;
                let nmat = mm.get(ploc as uint).mul_m(&delta.delta.to_mat4());
                mm.set((off + gen_off) as uint, nmat);
            }
            last_gen_off = gen_off;
        }

        ComputedPosition {
            gen: self.gen.clone()
        }
    }

    pub fn to_positions_cl(&self, cq: &CommandQueue, ctx: &mut CalcPositionsCl,
                           out: &[CLBuffer<Vec4<f32>>, ..4]) -> (Event, ComputedPosition) {

        cq.map_mut(&ctx.input, (), |out_delta| {
            for (&(gen_off, _), (_, gen)) in self.gen.iter().zip(self.delta.iter()) {
                for (off, delta) in gen.iter() {
                    out_delta[(off + gen_off) as uint] = delta.clone();
                }
            }            
        });

        // write init value
        ctx.init_kernel.set_arg(0, &out[0]);
        ctx.init_kernel.set_arg(1, &out[1]);
        ctx.init_kernel.set_arg(2, &out[2]);
        ctx.init_kernel.set_arg(3, &out[3]);
        let mut event = cq.enqueue_async_kernel(&ctx.init_kernel, 1, None, ());

        // run the kernel across the deltas 
        ctx.kernel.set_arg(0, &ctx.input);
        ctx.kernel.set_arg(1, &out[0]);
        ctx.kernel.set_arg(2, &out[1]);
        ctx.kernel.set_arg(3, &out[2]);
        ctx.kernel.set_arg(4, &out[3]);
        for idx in range(1, self.gen.len()) {
            let (off, _) = *self.gen.get(idx-1);
            ctx.kernel.set_arg(5, &off);
            let (off2, len) = *self.gen.get(idx);
            ctx.kernel.set_arg(6, &off2);
            event = cq.enqueue_async_kernel(&ctx.kernel, len as uint, None, event);
        }

        event.wait();

        (event, ComputedPosition {
            gen: self.gen.clone()
        })
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
    pub fn root() -> Id {Id(0, 0)}

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
    input: CLBuffer<Delta>
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

        CalcPositionsCl {
            program: program,
            kernel: kernel,
            init_kernel: init_kernel,
            input: delta_mem
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
                Transform3D::new(1f32, Quat::identity(), Vec3::new(0f32, 0f32, 0f32))
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

    fn position(&self, oid: ObjectKey) -> Mat4<f32> {
        let obj = self.object(oid);
        let p_mat = match obj {
            Some(obj) => self.position(obj.parent),
            None => Mat4::identity()
        };

        let loc = match self.location(oid) {
            Some(t) => {t.get().to_mat4()},
            None => Mat4::identity()
        };
        p_mat.mul_m(&loc)
    }

    fn to_positions<MM: MatrixManager>(&self, mm: &mut MM) -> ComputedPosition {
        self.get_position().position.to_positions(mm)
    }

    fn to_positions_cl(&self, cq: &CommandQueue,
                        ctx: &mut CalcPositionsCl, out: &[CLBuffer<Vec4<f32>>, ..4]) -> (Event, ComputedPosition) {
        self.get_position().position.to_positions_cl(cq, ctx, out)
    }

    fn to_positions_gl(&self, out_delta: &mut [Delta]) -> ComputedPositionGL {
        self.get_position().position.to_positions_gl(out_delta)
    }

    fn location_iter<'a>(&'a self) -> BTreeMapIterator<'a, ObjectKey, Id> {
        self.get_position().location.iter()
    }
}