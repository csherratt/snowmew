
extern crate snowmew;

mod db {
    use snowmew::common::Database;

    #[test]
    fn create_db() {
        let mut db = Database::new();

        let key = db.new_object(None, ~"main");

        assert!(db.find("main").unwrap() == key); 
    }

    #[test]
    fn scene_children() {
        let mut db = Database::new();

        let key = db.new_object(None, ~"main");

        let mut keys = ~[];

        for i in range(0, 100) {
            keys.push((i, db.new_object(Some(key), format!("obj_{}", i))));
        }

        for &(idx, key) in keys.iter() {
            assert!(db.find(format!("main/obj_{}", idx)).unwrap() == key);
        }
    }
}