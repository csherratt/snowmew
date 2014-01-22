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
}