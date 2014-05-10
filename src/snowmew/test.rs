#![feature(globs)]

extern crate snowmew;
extern crate OpenCL;
extern crate cow;

mod core {
    use snowmew::common::{CommonData, Common};

    #[test]
    fn db_new_object() {
        let mut db = CommonData::new();

        let id = db.new_object(None, "main");

        assert!(db.find("main").unwrap() == id);
    }
}