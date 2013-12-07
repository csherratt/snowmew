
extern mod snowmew;

mod db {
    use snowmew::db::Database;

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
}