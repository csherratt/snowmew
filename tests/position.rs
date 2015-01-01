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

#![feature(macro_rules)]
#![feature(globs)]
#![feature(phase)]

extern crate "snowmew-core" as snowmew;
extern crate cgmath;
extern crate opencl;
extern crate cow;
extern crate "snowmew-position" as position;

use position::{PositionData, Positions};
use position::cl::Accelerator;

use cgmath::{Matrix4, Matrix, Decomposed, Quaternion, Vector3, Vector4};

use opencl::hl::EventList;

#[test]
fn children() {
    let mut pos = PositionData::new();
    pos.set_delta(0, None, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(1, Some(0), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(2, Some(1), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(3, Some(2), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(4, Some(3), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});

    let mat0 = pos.position(0);
    let mat1 = pos.position(1);
    let mat2 = pos.position(2);
    let mat3 = pos.position(3);
    let mat4 = pos.position(4);

    let vec = Vector4::new(0f32, 0f32, 0f32, 1f32);
    assert!(mat0.mul_v(&vec) == Vector4::new(1f32, 1f32, 1f32, 1f32));
    assert!(mat1.mul_v(&vec) == Vector4::new(2f32, 2f32, 2f32, 1f32));
    assert!(mat2.mul_v(&vec) == Vector4::new(3f32, 3f32, 3f32, 1f32));
    assert!(mat3.mul_v(&vec) == Vector4::new(4f32, 4f32, 4f32, 1f32));
    assert!(mat4.mul_v(&vec) == Vector4::new(5f32, 5f32, 5f32, 1f32));
}

#[test]
fn children_tree() {
    let mut pos = PositionData::new();
    pos.set_delta(0, None, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(1, None, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});
    pos.set_delta(2, Some(0), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(3, Some(0), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});
    pos.set_delta(4, Some(1), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(5, Some(1), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});

    let mat0 = pos.position(2);
    let mat1 = pos.position(3);
    let mat2 = pos.position(4);
    let mat3 = pos.position(5);

    let vec = Vector4::new(0f32, 0f32, 0f32, 1f32);
    assert!(mat0.mul_v(&vec) == Vector4::new(2f32, 2f32, 2f32, 1f32));
    assert!(mat1.mul_v(&vec) == Vector4::new(0f32, 0f32, 0f32, 1f32));
    assert!(mat2.mul_v(&vec) == Vector4::new(0f32, 0f32, 0f32, 1f32));
    assert!(mat3.mul_v(&vec) == Vector4::new(-2f32, -2f32, -2f32, 1f32));
}

#[test]
fn write_positions() {
    let mut pos = PositionData::new();
    let mut vec: Vec<Matrix4<f32>> = vec![Matrix4::identity(), Matrix4::identity(), Matrix4::identity(), Matrix4::identity(),
                                          Matrix4::identity(), Matrix4::identity(), Matrix4::identity(), Matrix4::identity()];

    pos.set_delta(0, None, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(1, Some(0), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(2, Some(1), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(3, Some(2), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(4, Some(3), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});

    pos.write_positions(&mut vec);

    let mat0 = vec[0];
    let mat1 = vec[1];
    let mat2 = vec[2];
    let mat3 = vec[3];
    let mat4 = vec[4];

    let vec = Vector4::new(0f32, 0f32, 0f32, 1f32);

    assert!(mat0.mul_v(&vec) == Vector4::new(1f32, 1f32, 1f32, 1f32));
    assert!(mat1.mul_v(&vec) == Vector4::new(2f32, 2f32, 2f32, 1f32));
    assert!(mat2.mul_v(&vec) == Vector4::new(3f32, 3f32, 3f32, 1f32));
    assert!(mat3.mul_v(&vec) == Vector4::new(4f32, 4f32, 4f32, 1f32));
    assert!(mat4.mul_v(&vec) == Vector4::new(5f32, 5f32, 5f32, 1f32));
}

#[test]
fn write_positions_tree() {
    let mut vec: Vec<Matrix4<f32>> =  vec![Matrix4::identity(), Matrix4::identity(), Matrix4::identity(), Matrix4::identity(),
                                           Matrix4::identity(), Matrix4::identity(), Matrix4::identity(), Matrix4::identity()];

    let mut pos = PositionData::new();
    pos.set_delta(0, None, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(1, None, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});
    pos.set_delta(2, Some(0), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(3, Some(0), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});
    pos.set_delta(4, Some(1), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(5, Some(1), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});

    pos.write_positions(&mut vec);

    let mat0 = vec[2];
    let mat1 = vec[3];
    let mat2 = vec[4];
    let mat3 = vec[5];

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
fn calc_positions_opencl_vec4x4() {
    let (device, context, queue) = opencl::util::create_compute_context_prefer(opencl::util::PreferedType::GPUPrefered).unwrap();
    let mut ctx = Accelerator::new(&context, &device);

    let buffers: [opencl::mem::CLBuffer<Vector4<f32>>, ..4]
                = [context.create_buffer(16, opencl::cl::CL_MEM_READ_WRITE),
                   context.create_buffer(16, opencl::cl::CL_MEM_READ_WRITE),
                   context.create_buffer(16, opencl::cl::CL_MEM_READ_WRITE),
                   context.create_buffer(16, opencl::cl::CL_MEM_READ_WRITE)];

    let mut pos = PositionData::new();
    pos.set_delta(0, None, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(1, Some(0), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(2, Some(1), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(3, Some(2), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(4, Some(3), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});

    ctx.compute_vec4x4(&pos, &queue, &buffers).wait();
    let vec = fetch_matrixs(&queue, &buffers);

    let mat0 = vec[0];
    let mat1 = vec[1];
    let mat2 = vec[2];
    let mat3 = vec[3];
    let mat4 = vec[4];

    let vec = Vector4::new(0f32, 0f32, 0f32, 1f32);
    assert_eq!(mat0.mul_v(&vec), Vector4::new(1f32, 1f32, 1f32, 1f32));
    assert_eq!(mat1.mul_v(&vec), Vector4::new(2f32, 2f32, 2f32, 1f32));
    assert_eq!(mat2.mul_v(&vec), Vector4::new(3f32, 3f32, 3f32, 1f32));
    assert_eq!(mat3.mul_v(&vec), Vector4::new(4f32, 4f32, 4f32, 1f32));
    assert_eq!(mat4.mul_v(&vec), Vector4::new(5f32, 5f32, 5f32, 1f32));
}

#[test]
fn calc_positions_opencl_vec4x4_tree() {
    let (device, context, queue) = opencl::util::create_compute_context_prefer(opencl::util::PreferedType::GPUPrefered).unwrap();
    let mut ctx = Accelerator::new(&context, &device);

    let buffers: [opencl::mem::CLBuffer<Vector4<f32>>, ..4]
            = [context.create_buffer(16, opencl::cl::CL_MEM_READ_WRITE),
               context.create_buffer(16, opencl::cl::CL_MEM_READ_WRITE),
               context.create_buffer(16, opencl::cl::CL_MEM_READ_WRITE),
               context.create_buffer(16, opencl::cl::CL_MEM_READ_WRITE)];

    let mut pos = PositionData::new();
    pos.set_delta(0, None, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(1, None, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});
    pos.set_delta(2, Some(0), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(3, Some(0), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});
    pos.set_delta(4, Some(1), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(5, Some(1), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});

    ctx.compute_vec4x4(&pos, &queue, &buffers).wait();
    let vec = fetch_matrixs(&queue, &buffers);

    let mat0 = vec[2];
    let mat1 = vec[3];
    let mat2 = vec[4];
    let mat3 = vec[5];

    let vec = Vector4::new(0f32, 0f32, 0f32, 1f32);
    assert!(mat0.mul_v(&vec) == Vector4::new(2f32, 2f32, 2f32, 1f32));
    assert!(mat1.mul_v(&vec) == Vector4::new(0f32, 0f32, 0f32, 1f32));
    assert!(mat2.mul_v(&vec) == Vector4::new(0f32, 0f32, 0f32, 1f32));
    assert!(mat3.mul_v(&vec) == Vector4::new(-2f32, -2f32, -2f32, 1f32));
}

#[test]
fn calc_positions_opencl_mat() {
    let (device, context, queue) = opencl::util::create_compute_context_prefer(opencl::util::PreferedType::GPUPrefered).unwrap();
    let mut ctx = Accelerator::new(&context, &device);

    let buffers: opencl::mem::CLBuffer<Matrix4<f32>>
                = context.create_buffer(16, opencl::cl::CL_MEM_READ_WRITE);

    let mut pos = PositionData::new();
    pos.set_delta(0, None, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(1, Some(0), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(2, Some(1), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(3, Some(2), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(4, Some(3), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});

    let evt = ctx.compute_mat(&pos, &queue, &buffers).wait();
    let vec: Vec<Matrix4<f32>> = queue.get(&buffers, evt);

    let mat0 = vec[0];
    let mat1 = vec[1];
    let mat2 = vec[2];
    let mat3 = vec[3];
    let mat4 = vec[4];

    let vec = Vector4::new(0f32, 0f32, 0f32, 1f32);
    assert_eq!(mat0.mul_v(&vec), Vector4::new(1f32, 1f32, 1f32, 1f32));
    assert_eq!(mat1.mul_v(&vec), Vector4::new(2f32, 2f32, 2f32, 1f32));
    assert_eq!(mat2.mul_v(&vec), Vector4::new(3f32, 3f32, 3f32, 1f32));
    assert_eq!(mat3.mul_v(&vec), Vector4::new(4f32, 4f32, 4f32, 1f32));
    assert_eq!(mat4.mul_v(&vec), Vector4::new(5f32, 5f32, 5f32, 1f32));
}

#[test]
fn calc_positions_opencl_mat_tree() {
    let (device, context, queue) = opencl::util::create_compute_context_prefer(opencl::util::PreferedType::GPUPrefered).unwrap();
    let mut ctx = Accelerator::new(&context, &device);

    let buffers: opencl::mem::CLBuffer<Matrix4<f32>>
                = context.create_buffer(16, opencl::cl::CL_MEM_READ_WRITE);

    let mut pos = PositionData::new();
    pos.set_delta(0, None, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(1, None, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});
    pos.set_delta(2, Some(0), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(3, Some(0), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});
    pos.set_delta(4, Some(1), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(5, Some(1), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(-1f32, -1f32, -1f32)});

    let evt = ctx.compute_mat(&pos, &queue, &buffers).wait();
    let vec: Vec<Matrix4<f32>> = queue.get(&buffers, evt);

    let mat0 = vec[2];
    let mat1 = vec[3];
    let mat2 = vec[4];
    let mat3 = vec[5];

    let vec = Vector4::new(0f32, 0f32, 0f32, 1f32);
    assert!(mat0.mul_v(&vec) == Vector4::new(2f32, 2f32, 2f32, 1f32));
    assert!(mat1.mul_v(&vec) == Vector4::new(0f32, 0f32, 0f32, 1f32));
    assert!(mat2.mul_v(&vec) == Vector4::new(0f32, 0f32, 0f32, 1f32));
    assert!(mat3.mul_v(&vec) == Vector4::new(-2f32, -2f32, -2f32, 1f32));
}


#[test]
fn calc_positions_opencl_gap() {
    let (device, context, queue) = opencl::util::create_compute_context_prefer(opencl::util::PreferedType::GPUPrefered).unwrap();
    let mut ctx = Accelerator::new(&context, &device);

    let buffers: opencl::mem::CLBuffer<Matrix4<f32>>
                = context.create_buffer(16, opencl::cl::CL_MEM_READ_WRITE);

    let mut pos = PositionData::new();
    pos.set_delta(0, None, Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(11, Some(0), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(12, Some(11), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(13, Some(12), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});
    pos.set_delta(14, Some(13), Decomposed{scale: 1f32, rot: Quaternion::identity(), disp: Vector3::new(1f32, 1f32, 1f32)});

    let evt = ctx.compute_mat(&pos, &queue, &buffers).wait();
    let vec: Vec<Matrix4<f32>> = queue.get(&buffers, evt);

    let mat0 = vec[0];
    let mat1 = vec[11];
    let mat2 = vec[12];
    let mat3 = vec[13];
    let mat4 = vec[14];

    let vec = Vector4::new(0f32, 0f32, 0f32, 1f32);
    assert_eq!(mat0.mul_v(&vec), Vector4::new(1f32, 1f32, 1f32, 1f32));
    assert_eq!(mat1.mul_v(&vec), Vector4::new(2f32, 2f32, 2f32, 1f32));
    assert_eq!(mat2.mul_v(&vec), Vector4::new(3f32, 3f32, 3f32, 1f32));
    assert_eq!(mat3.mul_v(&vec), Vector4::new(4f32, 4f32, 4f32, 1f32));
    assert_eq!(mat4.mul_v(&vec), Vector4::new(5f32, 5f32, 5f32, 1f32));
}
