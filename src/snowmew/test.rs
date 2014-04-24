#![feature(macro_rules)]
#![feature(globs)]
#![feature(phase)]

#[phase(syntax, link)] extern crate snowmew;
extern crate cgmath;
extern crate OpenCL;
extern crate cow;

mod core {
    use snowmew::core::{Database, Common};
    use snowmew::position::Positions;
    use cgmath::transform::*;
    use cgmath::vector::*;
    use cgmath::quaternion::*;
    use cgmath::matrix::*;

    #[test]
    fn db_new_object()
    {
        let mut db = Database::new();

        let id = db.new_object(None, "main");

        assert!(db.find("main").unwrap() == id);
    }

    #[test]
    fn db_set_location()
    {
        let mut db = Database::new();

        let id = db.new_object(None, "main");

        let trans = Transform3D::new(2f32,
                        Quat::new(1f32, 2f32, 3f32, 4f32),
                        Vec3::new(1f32, 2f32, 3f32));

        db.update_location(id, trans);

        assert!(db.location(id).unwrap().get().to_mat4() == trans.get().to_mat4());
    }
}

mod position {
    use OpenCL;
    use snowmew::{Deltas, CalcPositionsCl};
    use cgmath::matrix::Matrix;
    use cgmath::transform::Transform3D;
    use cgmath::quaternion::Quat;
    use cgmath::vector::{Vec3, Vec4};

    #[test]
    fn insert_children()
    {
        let mut pos = Deltas::new();

        let id0 = pos.insert(Deltas::root(), Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id1 = pos.insert(id0, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id2 = pos.insert(id1, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id3 = pos.insert(id2, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id4 = pos.insert(id3, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));

        let mat0 = pos.get_mat(id0);
        let mat1 = pos.get_mat(id1);
        let mat2 = pos.get_mat(id2);
        let mat3 = pos.get_mat(id3);
        let mat4 = pos.get_mat(id4);

        let vec = Vec4::new(0f32, 0f32, 0f32, 1f32);

        assert!(mat0.mul_v(&vec) == Vec4::new(1f32, 1f32, 1f32, 1f32));
        assert!(mat1.mul_v(&vec) == Vec4::new(2f32, 2f32, 2f32, 1f32));
        assert!(mat2.mul_v(&vec) == Vec4::new(3f32, 3f32, 3f32, 1f32));
        assert!(mat3.mul_v(&vec) == Vec4::new(4f32, 4f32, 4f32, 1f32));
        assert!(mat4.mul_v(&vec) == Vec4::new(5f32, 5f32, 5f32, 1f32));
    }

    #[test]
    fn insert_children_tree()
    {
        let mut pos = Deltas::new();

        let id0 = pos.insert(Deltas::root(), Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id1 = pos.insert(Deltas::root(), Transform3D::new(1f32, Quat::identity(), Vec3::new(-1f32, -1f32, -1f32)));

        let id0_0 = pos.insert(id0, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id0_1 = pos.insert(id0, Transform3D::new(1f32, Quat::identity(), Vec3::new(-1f32, -1f32, -1f32)));
        let id1_0 = pos.insert(id1, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id1_1 = pos.insert(id1, Transform3D::new(1f32, Quat::identity(), Vec3::new(-1f32, -1f32, -1f32)));
    
        let mat0 = pos.get_mat(id0_0);
        let mat1 = pos.get_mat(id0_1);
        let mat2 = pos.get_mat(id1_0);
        let mat3 = pos.get_mat(id1_1);

        let vec = Vec4::new(0f32, 0f32, 0f32, 1f32);

        assert!(mat0.mul_v(&vec) == Vec4::new(2f32, 2f32, 2f32, 1f32));
        assert!(mat1.mul_v(&vec) == Vec4::new(0f32, 0f32, 0f32, 1f32));
        assert!(mat2.mul_v(&vec) == Vec4::new(0f32, 0f32, 0f32, 1f32));
        assert!(mat3.mul_v(&vec) == Vec4::new(-2f32, -2f32, -2f32, 1f32));
    }

    #[test]
    fn to_positions()
    {
        let mut pos = Deltas::new();

        let id0 = pos.insert(Deltas::root(), Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id1 = pos.insert(id0, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id2 = pos.insert(id1, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id3 = pos.insert(id2, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id4 = pos.insert(id3, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));

        let pos = pos.to_positions();

        let mat0 = pos.get_mat(id0);
        let mat1 = pos.get_mat(id1);
        let mat2 = pos.get_mat(id2);
        let mat3 = pos.get_mat(id3);
        let mat4 = pos.get_mat(id4);

        let vec = Vec4::new(0f32, 0f32, 0f32, 1f32);

        assert!(mat0.mul_v(&vec) == Vec4::new(1f32, 1f32, 1f32, 1f32));
        assert!(mat1.mul_v(&vec) == Vec4::new(2f32, 2f32, 2f32, 1f32));
        assert!(mat2.mul_v(&vec) == Vec4::new(3f32, 3f32, 3f32, 1f32));
        assert!(mat3.mul_v(&vec) == Vec4::new(4f32, 4f32, 4f32, 1f32));
        assert!(mat4.mul_v(&vec) == Vec4::new(5f32, 5f32, 5f32, 1f32));
    }


    #[test]
    fn to_positions_tree()
    {
        let mut pos = Deltas::new();

        let id0 = pos.insert(Deltas::root(), Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id1 = pos.insert(Deltas::root(), Transform3D::new(1f32, Quat::identity(), Vec3::new(-1f32, -1f32, -1f32)));

        let id0_0 = pos.insert(id0, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id0_1 = pos.insert(id0, Transform3D::new(1f32, Quat::identity(), Vec3::new(-1f32, -1f32, -1f32)));
        let id1_0 = pos.insert(id1, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id1_1 = pos.insert(id1, Transform3D::new(1f32, Quat::identity(), Vec3::new(-1f32, -1f32, -1f32)));
    
        let pos = pos.to_positions();
    
        let mat0 = pos.get_mat(id0_0);
        let mat1 = pos.get_mat(id0_1);
        let mat2 = pos.get_mat(id1_0);
        let mat3 = pos.get_mat(id1_1);

        let vec = Vec4::new(0f32, 0f32, 0f32, 1f32);

        assert!(mat0.mul_v(&vec) == Vec4::new(2f32, 2f32, 2f32, 1f32));
        assert!(mat1.mul_v(&vec) == Vec4::new(0f32, 0f32, 0f32, 1f32));
        assert!(mat2.mul_v(&vec) == Vec4::new(0f32, 0f32, 0f32, 1f32));
        assert!(mat3.mul_v(&vec) == Vec4::new(-2f32, -2f32, -2f32, 1f32));
    }

    #[test]
    fn calc_positions_opencl()
    {
        let (device, context, queue) = OpenCL::util::create_compute_context_prefer(OpenCL::util::GPUPrefered).unwrap();
        let mut ctx = CalcPositionsCl::new(&context, &device);

        let mut pos = Deltas::new();

        let id0 = pos.insert(Deltas::root(), Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id1 = pos.insert(id0, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id2 = pos.insert(id1, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id3 = pos.insert(id2, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id4 = pos.insert(id3, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));

        let pos = pos.to_positions_cl(&queue, &mut ctx);

        let mat0 = pos.get_mat(id0);
        let mat1 = pos.get_mat(id1);
        let mat2 = pos.get_mat(id2);
        let mat3 = pos.get_mat(id3);
        let mat4 = pos.get_mat(id4);

        let vec = Vec4::new(0f32, 0f32, 0f32, 1f32);

        assert!(mat0.mul_v(&vec) == Vec4::new(1f32, 1f32, 1f32, 1f32));
        assert!(mat1.mul_v(&vec) == Vec4::new(2f32, 2f32, 2f32, 1f32));
        assert!(mat2.mul_v(&vec) == Vec4::new(3f32, 3f32, 3f32, 1f32));
        assert!(mat3.mul_v(&vec) == Vec4::new(4f32, 4f32, 4f32, 1f32));
        assert!(mat4.mul_v(&vec) == Vec4::new(5f32, 5f32, 5f32, 1f32));
    }

    #[test]
    fn calc_positions_opencl_tree()
    {
        let (device, context, queue) = OpenCL::util::create_compute_context_prefer(OpenCL::util::GPUPrefered).unwrap();
        let mut ctx = CalcPositionsCl::new(&context, &device);

        let mut pos = Deltas::new();

        let id0 = pos.insert(Deltas::root(), Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id1 = pos.insert(Deltas::root(), Transform3D::new(1f32, Quat::identity(), Vec3::new(-1f32, -1f32, -1f32)));

        let id0_0 = pos.insert(id0, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id0_1 = pos.insert(id0, Transform3D::new(1f32, Quat::identity(), Vec3::new(-1f32, -1f32, -1f32)));
        let id1_0 = pos.insert(id1, Transform3D::new(1f32, Quat::identity(), Vec3::new(1f32, 1f32, 1f32)));
        let id1_1 = pos.insert(id1, Transform3D::new(1f32, Quat::identity(), Vec3::new(-1f32, -1f32, -1f32)));
    
        let pos = pos.to_positions_cl(&queue, &mut ctx);;
    
        let mat0 = pos.get_mat(id0_0);
        let mat1 = pos.get_mat(id0_1);
        let mat2 = pos.get_mat(id1_0);
        let mat3 = pos.get_mat(id1_1);

        let vec = Vec4::new(0f32, 0f32, 0f32, 1f32);

        assert!(mat0.mul_v(&vec) == Vec4::new(2f32, 2f32, 2f32, 1f32));
        assert!(mat1.mul_v(&vec) == Vec4::new(0f32, 0f32, 0f32, 1f32));
        assert!(mat2.mul_v(&vec) == Vec4::new(0f32, 0f32, 0f32, 1f32));
        assert!(mat3.mul_v(&vec) == Vec4::new(-2f32, -2f32, -2f32, 1f32));
    }
}