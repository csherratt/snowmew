use std::mem;

use sync::mutex;

use cgmath::transform::Transform3D;
use cgmath::quaternion::Quat;
use cgmath::vector::Vec3;
use cgmath::matrix::{Mat4, ToMat4, Matrix};

use OpenCL::hl::{Device, Context, CommandQueue, Program, Kernel, EventList};
use OpenCL::mem::CLBuffer;
use OpenCL::CL::CL_MEM_READ_WRITE;

use time::precise_time_ns;

static mut delta_reuse_mutex: mutex::StaticMutex = mutex::MUTEX_INIT;
static mut delta_reuse: Option<~[~[Delta]]> = None;

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
    int parent;
    q4 rot;
    float scale;
    f3 pos;
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
transform_to_mat4(const Transform3D trans)
{
    Mat4 mat;

    float x2 = trans.rot.x + trans.rot.x;
    float y2 = trans.rot.y + trans.rot.y;
    float z2 = trans.rot.z + trans.rot.z;

    float xx2 = x2 * trans.rot.x;
    float xy2 = x2 * trans.rot.y;
    float xz2 = x2 * trans.rot.z;

    float yy2 = y2 * trans.rot.y;
    float yz2 = y2 * trans.rot.z;
    float zz2 = z2 * trans.rot.z;

    float sy2 = y2 * trans.rot.s;
    float sz2 = z2 * trans.rot.s;
    float sx2 = x2 * trans.rot.s;

    mat.x.x = (1. - yy2 - zz2) * trans.scale;
    mat.x.y = (xy2 + sz2) * trans.scale;
    mat.x.z = (xz2 - sy2) * trans.scale;
    mat.x.w = 0.;

    mat.y.x = (xy2 - sz2) * trans.scale;
    mat.y.y = (1. - xx2 - zz2) * trans.scale;
    mat.y.z = (yz2 + sx2) * trans.scale;
    mat.y.w = 0.;

    mat.z.x = (xy2 + sy2) * trans.scale;
    mat.z.y = (yz2 - sx2) * trans.scale;
    mat.z.z = (1. - xx2 - yy2) * trans.scale;
    mat.z.w = 0.;

    mat.w.x = trans.pos.x;
    mat.w.y = trans.pos.y;
    mat.w.z = trans.pos.z;
    mat.w.w = 1.;

    return mat;
}

kernel void
calc_gen(global Transform3D *t, global Mat4 *gen, int offset_last, int offset_this)
{
    int id = get_global_id(0);
    global Transform3D *trans = &t[offset_this+id];
    Mat4 mat = transform_to_mat4(trans[0]);
    gen[offset_this+id] = mult_m(gen[offset_last+trans->parent], mat);
}
";



fn delta_alloc() -> ~[Delta]
{
    unsafe {
        let guard = delta_reuse_mutex.lock();
        let new = match delta_reuse {
            Some(ref mut arr) => {
                match arr.pop() {
                    Some(new) => new,
                    None => ~[]
                }
            },
            None => {
                ~[]
            }
        };
        let _ = guard;
        new
    }
}

fn delta_free(old: ~[Delta])
{
    unsafe {
        let guard = delta_reuse_mutex.lock();
        if delta_reuse.is_none() {
            delta_reuse = Some(~[old]);
        } else {
            delta_reuse.as_mut().unwrap().push(old);
        }
        let _ = guard;
    }
}


static mut position_reuse_mutex: mutex::StaticMutex = mutex::MUTEX_INIT;
static mut position_reuse: Option<~[~[Mat4<f32>]]> = None;

fn position_alloc() -> ~[Mat4<f32>]
{
    unsafe {
        let guard = position_reuse_mutex.lock();
        let new = match position_reuse {
            Some(ref mut arr) => {
                match arr.pop() {
                    Some(new) => new,
                    None => ~[]
                }
            },
            None => ~[]
        };
        let _ = guard;
        new
    }
}

fn position_free(old: ~[Mat4<f32>])
{
    unsafe {
        let guard = position_reuse_mutex.lock();
        if position_reuse.is_none() {
            position_reuse = Some(~[old]);
        } else {
            position_reuse.as_mut().unwrap().push(old);
        }
        let _ = guard;
    }
}


pub struct Delta
{
    priv delta : Transform3D<f32>,
    priv parent: u32,
    priv padd: [u32, ..3]

}

impl Default for Delta
{
    fn default() -> Delta
    {
        Delta {
            parent: 0,
            delta: Transform3D::new(1f32, Quat::zero(), Vec3::zero()),
            padd: [0, 0, 0]
        }
    }
}

impl Clone for Delta
{
    fn clone(&self) -> Delta
    {
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

pub struct Deltas
{
    priv gen: ~[(u32, u32)],
    priv delta: ~[Delta],
}

impl Clone for Deltas
{
    fn clone(&self) -> Deltas
    {
        let mut vec = delta_alloc();
        vec.reserve(self.delta.len());
        unsafe {
            vec.set_len(self.delta.len());
            vec.copy_memory(self.delta.as_slice())
        }
        Deltas {
            gen: self.gen.clone(),
            delta: vec
        }
    }
}

impl Drop for Deltas
{
    fn drop(&mut self)
    {
        let mut vec = ~[];
        mem::swap(&mut vec, &mut self.delta);
        delta_free(vec);
    }
}

#[deriving(Clone, Default, Eq, TotalOrd, TotalEq)]
pub struct Id(u32, u32);

impl Deltas
{
    pub fn new() -> Deltas
    {
        Deltas {
            gen: ~[(0, 1)],
            delta: ~[Default::default()],
        }
    }

    pub fn root() -> Id {Id(0, 0)}

    pub fn get_loc(&self, id: Id) -> uint
    {
        let Id(gen, offset) = id;
        let (gen_offset, _) = self.gen[gen];

        (gen_offset + offset) as uint
    }

    fn add_loc(&mut self, gen: u32) -> (uint, u32)
    {
        if gen as uint == self.gen.len() {
            let (s, len) = self.gen[gen-1];
            self.gen.push((s+len, 1));

            ((s+len) as uint, 0)
        } else {
            for t in self.gen.mut_slice_from((gen+1) as uint).mut_iter() {
                let (off, len) = *t;
                *t = (off+1, len);
            }

            let (off, len) = self.gen[gen];
            self.gen[gen] = (off, len+1);

            ((off+len) as uint, len)
        }
    }

    pub fn insert(&mut self, parent: Id, delta: Transform3D<f32>) -> Id
    {
        let Id(gen, pid) = parent;

        assert!((gen as uint) < self.gen.len());

        let (loc, id) = self.add_loc(gen+1);
        self.delta.insert(loc, Delta {
            parent: pid,
            delta: delta,
            padd: [0, 0, 0]
        });

        Id(gen+1, id)
    }

    pub fn update(&mut self, id: Id, delta: Transform3D<f32>)
    {
        let loc = self.get_loc(id);
        self.delta[loc].delta = delta;
    }

    pub fn get_delta(&self, id :Id) -> Transform3D<f32>
    {
        let loc = self.get_loc(id);
        self.delta[loc].delta
    }

    pub fn get_mat(&self, id :Id) -> Mat4<f32>
    {
        let loc = self.get_loc(id);
        match id {
            Id(0, _) => {
                self.delta[loc].delta.to_mat4()
            },
            Id(gen, _) => {
                let mat = self.delta[loc].delta.to_mat4();
                let parent = Id(gen-1, self.delta[loc].parent);

                self.get_mat(parent).mul_m(&mat)
            }
        }
    }

    pub fn to_positions(&self) -> Positions
    {
        let mut mat = position_alloc();
        mat.reserve(self.delta.len());
        unsafe {mat.set_len(1);}
        mat[0] = Mat4::identity();

        let mut last_gen_off = 0;
        for &(gen_off, len) in self.gen.slice_from(1).iter() {
            for off in range(gen_off, gen_off+len) {
                let ploc = last_gen_off + self.delta[off].parent;
                let nmat = mat[ploc].mul_m(&self.delta[off].delta.to_mat4());
                mat.push(nmat);
            }
            last_gen_off = gen_off;
        }

        Positions {
            gen: self.gen.clone(),
            pos: mat
        }
    }

    pub fn to_positions_cl(&self, cq: &CommandQueue, ctx: &mut CalcPositionsCl) -> Positions
    {
        let default: &[Mat4<f32>] = &[Mat4::identity()];

        if self.gen.len() == 1 {
            return Positions {
                gen: self.gen.clone(),
                pos: ~[Mat4::identity()]
            };
        }

        let start = precise_time_ns();

        cq.write(&ctx.input, &self.delta.as_slice(), ());
        cq.write(&ctx.output, &default, ());

        let write_end = precise_time_ns();

        ctx.kernel.set_arg(0, &ctx.input);
        ctx.kernel.set_arg(1, &ctx.output);

        let (off, _) = self.gen[0];
        ctx.kernel.set_arg(2, &off);
        let (off, len) = self.gen[1];
        ctx.kernel.set_arg(3, &off);

        let mut event = cq.enqueue_async_kernel(&ctx.kernel, len as uint, None, ());

        for idx in range(2, self.gen.len()) {
            let (off, _) = self.gen[idx-1];
            ctx.kernel.set_arg(2, &off);
            let (off, len) = self.gen[idx];
            ctx.kernel.set_arg(3, &off);
            event = cq.enqueue_async_kernel(&ctx.kernel, len as uint, None, event);
        }

        let mut mat = position_alloc();
        mat.reserve(self.delta.len());
        unsafe {mat.set_len(self.delta.len());}

        let compute_done = precise_time_ns();

        event.wait();

        cq.read(&ctx.output, &mut mat.as_mut_slice(), ());

        let download = precise_time_ns();

        println!("{} {} {}",
            write_end - start,
            compute_done - write_end,
            download - compute_done);

        Positions {
            gen: self.gen.clone(),
            pos: mat
        }
    }

    pub fn to_positions_gl(&self, out_delta: &mut [Delta]) -> PositionsGL
    {
        for (idx, delta) in self.delta.iter().enumerate() {
            out_delta[idx] = delta.clone();
        }

        PositionsGL {
            gen: self.gen.clone()
        }
    }
}

pub struct PositionsGL {
    gen: ~[(u32, u32)]
}

impl PositionsGL
{
    pub fn get_loc(&self, id: Id) -> uint
    {
        let Id(gen, offset) = id;
        let (gen_offset, _) = self.gen[gen];

        (gen_offset + offset) as uint
    }
}

pub struct Positions
{
    priv gen: ~[(u32, u32)],
    priv pos: ~[Mat4<f32>],
}

impl Clone for Positions
{
    #[inline(never)]
    fn clone(&self) -> Positions
    {
        let mut vec = position_alloc();
        vec.reserve(self.pos.len());
        unsafe {
            vec.set_len(self.pos.len());
            vec.copy_memory(self.pos.as_slice())
        }
        Positions {
            gen: self.gen.clone(),
            pos: vec
        }
    }
}

impl Drop for Positions
{
    #[inline(never)]
    fn drop(&mut self)
    {
        let mut vec = ~[];
        mem::swap(&mut vec, &mut self.pos);
        position_free(vec);
    }
}

impl Positions
{
    pub fn root() -> Id {Id(0, 0)}

    fn get_loc(&self, id: Id) -> uint
    {
        let Id(gen, offset) = id;
        let (gen_offset, _) = self.gen[gen];

        (gen_offset + offset) as uint
    }

    pub fn get_mat(&self, id :Id) -> Mat4<f32>
    {
        self.pos[self.get_loc(id)].clone()
    }
}

pub struct CalcPositionsCl
{
    program: Program,
    kernel: Kernel,
    input: CLBuffer<Delta>,
    output: CLBuffer<Mat4<f32>>
}

impl CalcPositionsCl
{
    pub fn new(ctx: &Context, device: &Device) -> CalcPositionsCl
    {
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

        let mat_mem: CLBuffer<Mat4<f32>> = ctx.create_buffer(1024*1024, CL_MEM_READ_WRITE);
        let delta_mem: CLBuffer<Delta> = ctx.create_buffer(1024*1024, CL_MEM_READ_WRITE);

        CalcPositionsCl {
            program: program,
            kernel: kernel,
            input: delta_mem,
            output: mat_mem
        }
    }
}

