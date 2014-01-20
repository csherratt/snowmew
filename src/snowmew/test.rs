#[feature(macro_rules)];
#[feature(globs)];

extern mod snowmew;
extern mod cgmath;

mod core {
    use std::hashmap::HashMap;
    use snowmew::core::{Database, Drawable};
    use cgmath::transform::*;
    use cgmath::vector::*;
    use cgmath::quaternion::*;
    use cgmath::matrix::*;

    #[test]
    fn test_db_new_object()
    {
        let mut db = Database::new();

        let id = db.new_object(None, ~"main");

        assert!(db.find("main").unwrap() == id);
    }

    #[test]
    fn test_db_set_location()
    {
        let mut db = Database::new();

        let id = db.new_object(None, ~"main");

        let trans = Transform3D::new(2f32,
                        Quat::new(1f32, 2f32, 3f32, 4f32),
                        Vec3::new(1f32, 2f32, 3f32));

        db.update_location(id, trans);

        assert!(db.location(id).unwrap().get().to_mat4() == trans.get().to_mat4());
    }

    #[test]
    fn test_db_set_draw()
    {
        let mut db = Database::new();

        let id = db.new_object(None, ~"main");

        let draw = Drawable{
            shader: 1,
            geometry: 2,
            textures: ~[3, 4]
        };

        db.update_drawable(id, draw.clone());

        assert!(db.drawable(id).unwrap() == &draw);
    }

    #[test]
    fn test_walk_drawable()
    {
        let mut db = Database::new();

        let id = db.new_object(None, ~"scene");
        let child_id0 = db.new_object(Some(id), ~"child_0");
        let child_id1 = db.new_object(Some(id), ~"child_1");
        let child_id1_0 = db.new_object(Some(child_id1), ~"child_1_0");

        let trans1 = Transform3D::new(0.25f32,
                        Quat::new(0f32, 0f32, 0f32, 0f32),
                        Vec3::new(3f32, 3f32, 3f32));

        let trans2 = Transform3D::new(1f32,
                        Quat::new(0f32, 0f32, 0f32, 0f32),
                        Vec3::new(9f32, 9f32, 9f32));

        db.update_location(id, trans1.clone());
        db.update_location(child_id0, trans1.clone());
        db.update_location(child_id1, trans2.clone());
        db.update_location(child_id1_0, trans2.clone());

        let draw = Drawable{
            shader: 1,
            geometry: 2,
            textures: ~[3, 4]
        };

        db.update_drawable(child_id0, draw.clone());
        db.update_drawable(child_id1, draw.clone());
        db.update_drawable(child_id1_0, draw.clone());

        let mut map = HashMap::new();
        let mat1 = trans1.get().to_mat4();
        let mat2 = trans2.get().to_mat4();

        map.insert(child_id0, mat1.mul_m(&mat1));
        map.insert(child_id1, mat1.mul_m(&mat2));
        map.insert(child_id1_0, mat1.mul_m(&mat2.mul_m(&mat2)));

        for (id, mat) in db.walk_drawables(id) {
            assert!(map.find(&id).unwrap() == &mat);
        }
    }
}