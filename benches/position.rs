
extern crate "snowmew-core" as snowmew;
extern crate cgmath;
extern crate opencl;
extern crate test;
extern crate "snowmew-position" as position;

use test::Bencher;
use position::{PositionData, Positions};
use position::cl::Accelerator;

use cgmath::{Matrix4, Matrix, Decomposed, Quaternion, Vector3, Vector4};

use opencl::hl::EventList;


#[bench]
fn calc_positions_opencl_mat_gpu(bench: &mut Bencher) {
    let (device, context, queue) = opencl::util::create_compute_context_prefer(opencl::util::GPUPrefered).unwrap();
    let mut ctx = Accelerator::new(&context, &device);

    let buffers: opencl::mem::CLBuffer<Matrix4<f32>> 
                = context.create_buffer(128*1024, opencl::cl::CL_MEM_WRITE_ONLY);

    let mut pos = PositionData::new();
    for i in range(0u32, 128*1024) {
        if i % 16 == 0 {
            pos.set_delta(i, None, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
        } else {
            pos.set_delta(i, Some(i-1), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
        }
    }

    bench.iter(|| {
        ctx.compute_mat(&pos, &queue, &buffers).wait();
    });
}

#[bench]
fn calc_positions_opencl_mat_cpu(bench: &mut Bencher) {
    let (device, context, queue) = opencl::util::create_compute_context_prefer(opencl::util::CPUPrefered).unwrap();
    let mut ctx = Accelerator::new(&context, &device);

    let buffers: opencl::mem::CLBuffer<Matrix4<f32>> 
                = context.create_buffer(128*1024, opencl::cl::CL_MEM_WRITE_ONLY);

    let mut pos = PositionData::new();
    for i in range(0u32, 128*1024) {
        if i % 16 == 0 {
            pos.set_delta(i, None, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
        } else {
            pos.set_delta(i, Some(i-1), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
        }
    }

    bench.iter(|| {
        ctx.compute_mat(&pos, &queue, &buffers).wait();
    });
}