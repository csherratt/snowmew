

mod Manager {
    extern crate gui = "snowmew-gui";
    extern crate sync;

    struct Pinged {
        count: sync::Arc<sync::Mutex<uint>>
    }

    impl gui::Handler for Pinged {
        fn handle(&mut self, _: gui::Event, _: |id: gui::ItemId, evt: gui::Event|) {
            let mut guard = self.count.lock();
            *guard += 1;
        }
    }

    #[test]
    fn add() {
        let count = sync::Arc::new(sync::Mutex::new(0));
        let mut manager = gui::Manager::new();
        
        let widget = ~Pinged {
            count: count.clone()
        };

        let id = manager.add(widget);
        manager.root(id);

        manager.event(gui::MouseEvent(gui::Mouse::new()));

        let guard = count.lock();
        assert!(*guard == 1);
    }
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

    #[test]
    fn get_pos() {
        let mut layout = gui::Layout::new();
        layout.add((0., 0.), (100., 100.), 1., 10);
        layout.add((25., 25.), (75., 75.), 2., 11);

        assert!(Some((0., 0.)) == layout.pos(10))
        assert!(Some((25., 25.)) == layout.pos(11))
    }
}