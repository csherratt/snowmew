use std::vec;

use cgmath::matrix::{Matrix, Mat4};

use OpenCL::CL::{CL_MEM_WRITE_ONLY, CL_MEM_READ_ONLY};
use OpenCL::mem::{CLBuffer, Buffer};
use OpenCL::hl::{Context, CommandQueue, Program, Kernel, Device, KernelArg};

use db::Graphics;
use snowmew::core::{object_key, Drawable};


static source: &'static str = &"

struct f4 {
    float x, y, z, w;
};

typedef struct f4 f4;

struct mat4 {
    f4 x, y, z, w; 
};

typedef struct mat4 Mat4;

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

f4
mult_v(Mat4 mat, f4 vec) 
{
    f4 out = {
        mat.x.x * vec.x + mat.y.x * vec.y + mat.z.x * vec.z + mat.w.x * vec.w,
        mat.x.y * vec.x + mat.y.y * vec.y + mat.z.y * vec.z + mat.w.y * vec.w,
        mat.x.z * vec.x + mat.y.z * vec.y + mat.z.z * vec.z + mat.w.z * vec.w,
        mat.x.w * vec.x + mat.y.w * vec.y + mat.z.w * vec.z + mat.w.w * vec.w,
    };

    return out;
}

constant f4 corner[8] = {
    {1.,  1.,  1., 1.}, {-1.,  1.,  1., 1.},
    {1., -1.,  1., 1.}, {-1., -1.,  1., 1.},
    {1.,  1., -1., 1.}, {-1.,  1., -1., 1.},
    {1., -1., -1., 1.}, {-1., -1., -1., 1.}
};

kernel void
bounds_check(global Mat4 *in, global char *out, global Mat4 *proj)
{
    int i;
    bool visible = true;
    f4 point[8];
    int index = get_global_id(0);

    if (index < get_global_size(0)) {
        Mat4 mat = mult_m(proj[0], in[index]);

        for (i=0; i<8; i++) {
            point[i] = mult_v(mat, corner[i]);
            float inv = 1. / point[i].w;
            point[i].x *= inv;
            point[i].y *= inv;
            point[i].z *= inv;
        }

        bool behind_camera = true;
        bool right_of_camera = true;
        bool left_of_camera = true;
        bool above_camera = true;
        bool below_camera = true;
        for (i=0; i<8; i++) {
            behind_camera &= point[i].z > 1.;
            right_of_camera &= point[i].x > 1.;
            left_of_camera &= point[i].x < -1.;
            above_camera &= point[i].y > 1.;
            below_camera &= point[i].y < -1.;
        }

        out[index] = (behind_camera|
                      right_of_camera|left_of_camera|
                      above_camera|below_camera) ? 0 : 1;
    }
}
";

static size: uint = 16*1024;

pub struct ObjectCullOffloadContext
{
    priv input: ~[Mat4<f32>],
    priv input_buffer: CLBuffer<Mat4<f32>>,
    priv output: ~[i8],
    priv output_buffer: CLBuffer<i8>,
    priv camera: CLBuffer<Mat4<f32>>,

    priv program: Program,
    priv kernel: Kernel,
    priv queue: CommandQueue,
}

pub struct ObjectCullOffloadContextIter<'a, IN>
{
    priv parent: &'a mut ObjectCullOffloadContext,

    priv input: IN,
    priv keys: ~[object_key],
    priv output_idx: uint
}

impl ObjectCullOffloadContext
{
    pub fn new(ctx: &Context, device: &Device, queue: CommandQueue) -> ObjectCullOffloadContext
    {
        let program = ctx.create_program_from_source(source.clone());
        match program.build(device) {
            Err(s) => fail!("could not compile: {:?}", s),
            _ => ()
        }

        let kernel = program.create_kernel("bounds_check");

        ObjectCullOffloadContext {
            input: vec::with_capacity(size),
            input_buffer: ctx.create_buffer(size, CL_MEM_READ_ONLY),
            output: vec::with_capacity(size),
            output_buffer: ctx.create_buffer(size, CL_MEM_WRITE_ONLY),
            camera: ctx.create_buffer(1, CL_MEM_READ_ONLY),

            program: program,
            kernel: kernel,
            queue: queue,
        }
    }

    pub fn iter<'a, IN: Iterator<(object_key, Mat4<f32>)>>(&'a mut self, input: IN, camera: Mat4<f32>) -> ObjectCullOffloadContextIter<'a, IN>
    {
        unsafe {
            self.input.set_len(0);
            self.output.set_len(0);
        }

        let camera = &[camera];
        self.queue.write(&self.camera, &camera, ());

        ObjectCullOffloadContextIter {
            input: input,
            keys: vec::with_capacity(size),
            parent: self,
            output_idx: 0
        }
    }
}

impl<'a, IN: Iterator<(object_key, Mat4<f32>)>>
     Iterator<(object_key, Mat4<f32>)> for ObjectCullOffloadContextIter<'a, IN>
{
    #[inline(never)]
    fn next(&mut self) -> Option<(object_key, Mat4<f32>)>
    {
        loop {
            while self.parent.output.len() > self.output_idx {
                let idx = self.output_idx;
                self.output_idx += 1;

                if self.parent.output[idx] == 1 {
                    return Some((self.keys[idx], self.parent.input[idx]));
                }
            }

            unsafe {
                self.parent.input.set_len(0);
                self.parent.output.set_len(0);
                self.output_idx = 0;
                self.keys.set_len(0);
            }


            // read input
            while self.keys.len() < size {
                match self.input.next() {
                    Some((oid, mat)) => {
                        self.keys.push(oid);
                        self.parent.input.push(mat);
                    },
                    None => break
                }
            }

            if self.keys.len() != 0 {
                self.parent.queue.write(&self.parent.input_buffer, &self.parent.input.as_slice(), ());

                self.parent.kernel.set_arg(0, &self.parent.input_buffer);
                self.parent.kernel.set_arg(1, &self.parent.output_buffer);
                self.parent.kernel.set_arg(2, &self.parent.camera);

                unsafe {
                    self.parent.output.set_len(self.keys.len());
                }

                let event = self.parent.queue.enqueue_async_kernel(&self.parent.kernel, self.keys.len(), None, ());
                self.parent.queue.read(&self.parent.output_buffer, &mut self.parent.output.as_mut_slice(), &event);
            } else {
                return None;
            }
        }
    }
}