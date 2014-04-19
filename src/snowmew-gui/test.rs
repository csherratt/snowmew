extern crate gui = "snowmew-gui";
extern crate sync;


mod Manager {
    use gui;
    use sync;

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
    use gui;

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

mod Button {
    use gui;

    #[test]
    fn button_press() {
        use gui::Handler;
        let mut button = gui::Button::new();
        let mut mouse = gui::Mouse::new();
        let mut event = None;

        button.setup(10);

        button.handle(gui::MouseEvent(mouse.clone()), |id, evt| {
            fail!("Should not have got and event {:?} {:?}", id, evt)
        });
 

        mouse.button[0] = true;
        button.handle(gui::MouseEvent(mouse.clone()), |id, evt| {
            event = Some((id, evt));
        });

        event.expect("Missing event");

        button.handle(gui::MouseEvent(mouse.clone()), |id, evt| {
            fail!("Should not have got and event on mouse hold {:?} {:?}", id, evt)
        });

        mouse.button[0] = false;
        event = None;
        button.handle(gui::MouseEvent(mouse.clone()), |id, evt| {
            event = Some((id, evt));
        });

        event.expect("Missing event");
    }
}