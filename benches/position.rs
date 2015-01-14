
extern crate "snowmew-core" as snowmew;
extern crate cgmath;
extern crate opencl;
extern crate test;
extern crate "snowmew-position" as position;

use test::{Bencher, black_box};
use position::{PositionData, Positions};
use position::cl::Accelerator;

use cgmath::{Matrix4, Decomposed, Quaternion, Vector3, Vector4};

use opencl::hl::{EventList, Context};

const SIZE: usize = 1024*64;

fn create_positon_data() -> PositionData {
    let mut pos = PositionData::new();
    for i in range(0u32, SIZE as u32) {
        if i % 16 == 0 {
            pos.set_delta(i, None, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
        } else {
            pos.set_delta(i, Some(i-1), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
        }
    }
    pos
}

fn create_buffers_vec4(ctx: &Context) -> [opencl::mem::CLBuffer<Vector4<f32>>, ..4] {
    [
        ctx.create_buffer(SIZE, opencl::cl::CL_MEM_WRITE_ONLY),
        ctx.create_buffer(SIZE, opencl::cl::CL_MEM_WRITE_ONLY),
        ctx.create_buffer(SIZE, opencl::cl::CL_MEM_WRITE_ONLY),
        ctx.create_buffer(SIZE, opencl::cl::CL_MEM_WRITE_ONLY)
    ]
}

fn create_buffers_mat(ctx: &Context) -> opencl::mem::CLBuffer<Matrix4<f32>> {
    ctx.create_buffer(SIZE, opencl::cl::CL_MEM_WRITE_ONLY)
}

#[bench]
fn calc_positions_opencl_mat_gpu(bench: &mut Bencher) {
    let (device, context, queue) = opencl::util::create_compute_context_prefer(opencl::util::PreferedType::GPUPrefered).unwrap();
    let mut ctx = Accelerator::new(&context, &device);

    let buffers = create_buffers_mat(&context);

    let pos = create_positon_data();

    bench.iter(|| {
        ctx.compute_mat(&pos, &queue, &buffers).wait();
    });
}

#[bench]
fn calc_positions_opencl_mat_cpu(bench: &mut Bencher) {
    let (device, context, queue) = opencl::util::create_compute_context_prefer(opencl::util::PreferedType::CPUPrefered).unwrap();
    let mut ctx = Accelerator::new(&context, &device);

    let buffers = create_buffers_mat(&context);
    let pos = create_positon_data();

    bench.iter(|| {
        ctx.compute_mat(&pos, &queue, &buffers).wait();
    });
}

#[bench]
fn calc_positions_opencl_vec4_gpu(bench: &mut Bencher) {
    let (device, context, queue) = opencl::util::create_compute_context_prefer(opencl::util::PreferedType::GPUPrefered).unwrap();
    let mut ctx = Accelerator::new(&context, &device);

    let buffers = create_buffers_vec4(&context);
    let pos = create_positon_data();

    bench.iter(|| {
        ctx.compute_vec4x4(&pos, &queue, &buffers).wait();
    });
}

#[bench]
fn calc_positions_opencl_vec4_cpu(bench: &mut Bencher) {
    let (device, context, queue) = opencl::util::create_compute_context_prefer(opencl::util::PreferedType::CPUPrefered).unwrap();
    let mut ctx = Accelerator::new(&context, &device);

    let buffers = create_buffers_vec4(&context);
    let pos = create_positon_data();

    bench.iter(|| {
        ctx.compute_vec4x4(&pos, &queue, &buffers).wait();
    });
}

#[bench]
fn calc_positions_iter(bench: &mut Bencher) {

    let pos = create_positon_data();

    bench.iter(|| {
        for (idx, mat) in pos.position_iter() {
            black_box(mat);
            black_box(idx);
        }
    });
}