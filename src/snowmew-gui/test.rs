
extern crate gui = "snowmew-gui";

#[test]
fn create() {
    let _ = gui::Manager::new();
}

mod Layout {
    extern crate gui = "snowmew-gui";

    #[test]
    fn item() {
        let mut layout = gui::Layout::new();
        layout.add((0., 0.), (100., 100.), 1., 10);

        assert!(None == layout.get_item(-1., -1.))
        assert!(None == layout.get_item(100., 100.))
        assert!(None == layout.get_item(0., 100.))
        assert!(None == layout.get_item(100., 0.))

        assert!(Some(10) == layout.get_item(0., 0.))
        assert!(Some(10) == layout.get_item(99., 99.))
    }

    #[test]
    fn item_ztest() {
        let mut layout = gui::Layout::new();
        layout.add((0., 0.), (100., 100.), 1., 10);
        layout.add((25., 25.), (75., 75.), 2., 11);

        assert!(None == layout.get_item(-1., -1.))
        assert!(None == layout.get_item(100., 100.))
        assert!(None == layout.get_item(0., 100.))
        assert!(None == layout.get_item(100., 0.))

        assert!(Some(10) == layout.get_item(0., 0.))
        assert!(Some(11) == layout.get_item(25., 25.))
        assert!(Some(11) == layout.get_item(99., 99.))
    }
}