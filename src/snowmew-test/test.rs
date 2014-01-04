
extern mod snowmew;
extern mod cgmath;
extern mod extra;

mod db {
    use extra;
    use cgmath::vector::{Vec3, Vector};
    use snowmew::db::{Database, Position};

    #[test]
    fn test_db_create_scene()
    {
        let mut db = Database::new();

        let scene = db.create_scene(Some(~"main"));

        assert!(scene == db.find("main").unwrap());
    }

    #[test]
    fn test_db_add_to_scene()
    {
        let mut db = Database::new();

        let scene = db.create_scene(Some(~"main"));

        let kitten = db.create_object(scene, Some(~"kitten"));
        let mouse = db.create_object(scene, Some(~"mouse"));
        let dog = db.create_object(scene, Some(~"dog"));

        assert!(kitten == db.find("main/kitten").unwrap());
        assert!(mouse == db.find("main/mouse").unwrap());
        assert!(dog == db.find("main/dog").unwrap());
        assert!(scene == db.find("main").unwrap());
    }

    #[test]
    fn test_db_get_position()
    {
        let mut db = Database::new();

        let scene = db.create_scene(Some(~"main"));

        let kitten = db.create_object(scene, Some(~"kitten"));
        let mouse = db.create_object(scene, Some(~"mouse"));
        let dog = db.create_object(scene, Some(~"dog"));

        assert!(db.get_position(scene).is_none());
        assert!(db.get_position(kitten).is_some());
        assert!(db.get_position(mouse).is_some());
        assert!(db.get_position(dog).is_some());
    }


    #[test]
    fn test_db_set_position()
    {
        let mut db = Database::new();

        let scene = db.create_scene(Some(~"main"));
        let kitten = db.create_object(scene, Some(~"kitten"));

        let old_pos = db.get_position(kitten).unwrap();

        let new_pos = Position {
            position: Vec3::new(1f32, 1f32, 1f32).add_v(&old_pos.position),
            rotation: old_pos.rotation,
            scale: old_pos.scale
        };

        db.set_position(kitten, new_pos);

        assert!(db.get_position(kitten).unwrap() == new_pos);
    }

    #[test]
    fn test_db_add_1K()
    {
        let mut db = Database::new();

        let scene = db.create_scene(Some(~"main"));
        
        for i in range(0, 1_000) {
            db.create_object(scene, Some(format!("id_{}", i)));
        }

        for i in range(0, 1_000) {
            db.find(format!("main/id_{}", i)).unwrap();
        }
    }

    #[bench]
    fn bench_db_add(b: &mut extra::test::BenchHarness)
    {
        let mut db = Database::new();

        let scene = db.create_scene(Some(~"main"));
        
        b.iter(|| {
            db.create_object(scene, None);
        });
    }

    #[bench]
    fn bench_find(b: &mut extra::test::BenchHarness)
    {
        let mut db = Database::new();

        let scene = db.create_scene(Some(~"main"));
        
        for i in range(0, 1_000) {
            db.create_object(scene, Some(format!("id_{}", i)));
        }

        let mut i = 0;

        b.iter(|| {
            if i > 1_000 {
                i -= 1_000;
            }
            db.find(format!("main/id_{}", i));
            i += 41;
        });
    }
}