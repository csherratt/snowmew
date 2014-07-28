#![feature(macro_rules)]
#![feature(globs)]
#![feature(phase)]

extern crate snowmew;
extern crate cgmath;
extern crate opencl;
extern crate cow;
extern crate position = "snowmew-position";

use position::Deltas;
use position::CalcPositionsCl;

use cgmath::matrix::{Matrix4, Matrix};
use cgmath::transform::Decomposed;
use cgmath::quaternion::Quaternion;
use cgmath::vector::{Vector3, Vector4};

use opencl::hl::EventList;

#[test]
fn insert_children() {
    let mut pos = Deltas::new();

    let id0 = pos.insert(Deltas::root(), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id1 = pos.insert(id0, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id2 = pos.insert(id1, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id3 = pos.insert(id2, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id4 = pos.insert(id3, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});

    let mat0 = pos.get_mat(id0);
    let mat1 = pos.get_mat(id1);
    let mat2 = pos.get_mat(id2);
    let mat3 = pos.get_mat(id3);
    let mat4 = pos.get_mat(id4);

    let vec = Vector4::new(0f32, 0f32, 0f32, 1f32);

    assert!(mat0.mul_v(&vec) == Vector4::new(1f32, 1f32, 1f32, 1f32));
    assert!(mat1.mul_v(&vec) == Vector4::new(2f32, 2f32, 2f32, 1f32));
    assert!(mat2.mul_v(&vec) == Vector4::new(3f32, 3f32, 3f32, 1f32));
    assert!(mat3.mul_v(&vec) == Vector4::new(4f32, 4f32, 4f32, 1f32));
    assert!(mat4.mul_v(&vec) == Vector4::new(5f32, 5f32, 5f32, 1f32));
}

#[test]
fn insert_children_tree() {
    let mut pos = Deltas::new();

    let id0 = pos.insert(Deltas::root(), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id1 = pos.insert(Deltas::root(), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});

    let id0_0 = pos.insert(id0, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id0_1 = pos.insert(id0, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});
    let id1_0 = pos.insert(id1, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id1_1 = pos.insert(id1, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});

    let mat0 = pos.get_mat(id0_0);
    let mat1 = pos.get_mat(id0_1);
    let mat2 = pos.get_mat(id1_0);
    let mat3 = pos.get_mat(id1_1);

    let vec = Vector4::new(0f32, 0f32, 0f32, 1f32);

    assert!(mat0.mul_v(&vec) == Vector4::new(2f32, 2f32, 2f32, 1f32));
    assert!(mat1.mul_v(&vec) == Vector4::new(0f32, 0f32, 0f32, 1f32));
    assert!(mat2.mul_v(&vec) == Vector4::new(0f32, 0f32, 0f32, 1f32));
    assert!(mat3.mul_v(&vec) == Vector4::new(-2f32, -2f32, -2f32, 1f32));
}

#[test]
fn to_positions() {
    let mut pos = Deltas::new();
    let mut vec: &mut [Matrix4<f32>] = &mut [Matrix4::identity(), Matrix4::identity(), Matrix4::identity(), Matrix4::identity(),
                                             Matrix4::identity(), Matrix4::identity(), Matrix4::identity(), Matrix4::identity()];

    let id0 = pos.insert(Deltas::root(), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id1 = pos.insert(id0, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id2 = pos.insert(id1, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id3 = pos.insert(id2, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id4 = pos.insert(id3, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});

    pos.write_positions(&mut vec);
    let pos = pos.compute_positions();

    let mat0 = vec[pos.get_loc(id0)];
    let mat1 = vec[pos.get_loc(id1)];
    let mat2 = vec[pos.get_loc(id2)];
    let mat3 = vec[pos.get_loc(id3)];
    let mat4 = vec[pos.get_loc(id4)];

    let vec = Vector4::new(0f32, 0f32, 0f32, 1f32);

    assert!(mat0.mul_v(&vec) == Vector4::new(1f32, 1f32, 1f32, 1f32));
    assert!(mat1.mul_v(&vec) == Vector4::new(2f32, 2f32, 2f32, 1f32));
    assert!(mat2.mul_v(&vec) == Vector4::new(3f32, 3f32, 3f32, 1f32));
    assert!(mat3.mul_v(&vec) == Vector4::new(4f32, 4f32, 4f32, 1f32));
    assert!(mat4.mul_v(&vec) == Vector4::new(5f32, 5f32, 5f32, 1f32));
}


#[test]
fn to_positions_tree() {
    let mut pos = Deltas::new();
    let mut vec: &mut [Matrix4<f32>] = &mut [Matrix4::identity(), Matrix4::identity(), Matrix4::identity(), Matrix4::identity(),
                                             Matrix4::identity(), Matrix4::identity(), Matrix4::identity(), Matrix4::identity()];

    let id0 = pos.insert(Deltas::root(), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id1 = pos.insert(Deltas::root(), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});

    let id0_0 = pos.insert(id0, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id0_1 = pos.insert(id0, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});
    let id1_0 = pos.insert(id1, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id1_1 = pos.insert(id1, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});

    pos.write_positions(&mut vec);
    let pos = pos.compute_positions();

    let mat0 = vec[pos.get_loc(id0_0)];
    let mat1 = vec[pos.get_loc(id0_1)];
    let mat2 = vec[pos.get_loc(id1_0)];
    let mat3 = vec[pos.get_loc(id1_1)];

    let vec = Vector4::new(0f32, 0f32, 0f32, 1f32);

    assert!(mat0.mul_v(&vec) == Vector4::new(2f32, 2f32, 2f32, 1f32));
    assert!(mat1.mul_v(&vec) == Vector4::new(0f32, 0f32, 0f32, 1f32));
    assert!(mat2.mul_v(&vec) == Vector4::new(0f32, 0f32, 0f32, 1f32));
    assert!(mat3.mul_v(&vec) == Vector4::new(-2f32, -2f32, -2f32, 1f32));
}

fn fetch_matrixs(queue: &opencl::hl::CommandQueue,
                 buffers: &[opencl::mem::CLBuffer<Vector4<f32>>, ..4]) -> Vec<Matrix4<f32>> {

    let vec0: Vec<Vector4<f32>> = queue.get(&buffers[0], ());
    let vec1: Vec<Vector4<f32>> = queue.get(&buffers[1], ());
    let vec2: Vec<Vector4<f32>> = queue.get(&buffers[2], ());
    let vec3: Vec<Vector4<f32>> = queue.get(&buffers[3], ());

    vec0.iter().zip(
    vec1.iter().zip(
    vec2.iter().zip(
    vec3.iter()))).map(|(a, (b, (c, d)))| {
        Matrix4::from_cols(*a, *b, *c, *d)
    }).collect()

}

#[test]
fn calc_positions_opencl() {
    let (device, context, queue) = opencl::util::create_compute_context_prefer(opencl::util::GPUPrefered).unwrap();
    let mut ctx = CalcPositionsCl::new(&context, &device);

    let mut pos_old = Deltas::new();
    let buffers: [opencl::mem::CLBuffer<Vector4<f32>>, ..4] 
                = [context.create_buffer(16, opencl::CL::CL_MEM_READ_WRITE),
                   context.create_buffer(16, opencl::CL::CL_MEM_READ_WRITE),
                   context.create_buffer(16, opencl::CL::CL_MEM_READ_WRITE),
                   context.create_buffer(16, opencl::CL::CL_MEM_READ_WRITE)];

    let id0 = pos_old.insert(Deltas::root(), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id1 = pos_old.insert(id0, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id2 = pos_old.insert(id1, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id3 = pos_old.insert(id2, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id4 = pos_old.insert(id3, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});

    pos_old.write_positions_cl_vec4x4(&queue, &mut ctx, &buffers).wait();
    let pos = pos_old.compute_positions();
    let vec = fetch_matrixs(&queue, &buffers);

    let mat0 = vec[pos.get_loc(id0)];
    let mat1 = vec[pos.get_loc(id1)];
    let mat2 = vec[pos.get_loc(id2)];
    let mat3 = vec[pos.get_loc(id3)];
    let mat4 = vec[pos.get_loc(id4)];

    let vec = Vector4::new(0f32, 0f32, 0f32, 1f32);

    assert!(mat0.mul_v(&vec) == Vector4::new(1f32, 1f32, 1f32, 1f32));
    assert!(mat1.mul_v(&vec) == Vector4::new(2f32, 2f32, 2f32, 1f32));
    assert!(mat2.mul_v(&vec) == Vector4::new(3f32, 3f32, 3f32, 1f32));
    assert!(mat3.mul_v(&vec) == Vector4::new(4f32, 4f32, 4f32, 1f32));
    assert!(mat4.mul_v(&vec) == Vector4::new(5f32, 5f32, 5f32, 1f32));
}

#[test]
fn calc_positions_opencl_tree() {
    let (device, context, queue) = opencl::util::create_compute_context_prefer(opencl::util::GPUPrefered).unwrap();
    let mut ctx = CalcPositionsCl::new(&context, &device);

    let mut pos = Deltas::new();
    let buffers: [opencl::mem::CLBuffer<Vector4<f32>>, ..4] 
            = [context.create_buffer(16, opencl::CL::CL_MEM_READ_WRITE),
                   context.create_buffer(16, opencl::CL::CL_MEM_READ_WRITE),
                   context.create_buffer(16, opencl::CL::CL_MEM_READ_WRITE),
                   context.create_buffer(16, opencl::CL::CL_MEM_READ_WRITE)];

    let id0 = pos.insert(Deltas::root(), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id1 = pos.insert(Deltas::root(), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});

    let id0_0 = pos.insert(id0, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id0_1 = pos.insert(id0, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});
    let id1_0 = pos.insert(id1, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    let id1_1 = pos.insert(id1, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});

    pos.write_positions_cl_vec4x4(&queue, &mut ctx, &buffers).wait();
    let pos = pos.compute_positions();
    let vec = fetch_matrixs(&queue, &buffers);

    let mat0 = vec[pos.get_loc(id0_0)];
    let mat1 = vec[pos.get_loc(id0_1)];
    let mat2 = vec[pos.get_loc(id1_0)];
    let mat3 = vec[pos.get_loc(id1_1)];

    let vec = Vector4::new(0f32, 0f32, 0f32, 1f32);

    assert!(mat0.mul_v(&vec) == Vector4::new(2f32, 2f32, 2f32, 1f32));
    assert!(mat1.mul_v(&vec) == Vector4::new(0f32, 0f32, 0f32, 1f32));
    assert!(mat2.mul_v(&vec) == Vector4::new(0f32, 0f32, 0f32, 1f32));
    assert!(mat3.mul_v(&vec) == Vector4::new(-2f32, -2f32, -2f32, 1f32));
}
