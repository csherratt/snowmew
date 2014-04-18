use collections::deque::Deque;
use collections::ringbuf::RingBuf;
use collections::trie::TrieMap;

use {ItemId, Event, Handler, Mouse, MouseEvent};
use glfw;

pub struct Manager {
    events: Option<RingBuf<(ItemId, Event)>>,
    widgets: TrieMap<~Handler:Send+Share>,
    root: ItemId,
    count: ItemId,
    mouse: Mouse
}

impl Manager {
    pub fn new() -> Manager {
        Manager {
            mouse: Mouse::new(),
            events: Some(RingBuf::new()),
            widgets: TrieMap::new(),
            root: 0,
            count: 1
        }
    }

    pub fn event(&mut self, evt: Event) {
        self.events.as_mut().unwrap().push_back((self.root, evt));
        self.flush()
    }

    pub fn event_glfw(&mut self, evt: glfw::WindowEvent) {
        let evt = match evt {
            glfw::PosEvent(x, y) => {
                self.mouse.pos((x as f32, y as f32));
                Some(MouseEvent(self.mouse.clone()))
            }
            _ => None
        };

        match evt {
            Some(evt) => self.event(evt),
            None => ()
        }
    }

    fn flush(&mut self) {
        let mut events = self.events.take().unwrap();

        'event_loop: loop {
            let (id, evt) = match events.pop_front() {
                Some(dat) => dat,
                None => {
                    break 'event_loop
                }
            };

            match self.widgets.find_mut(&id) {
                Some(handle) => {
                    handle.handle(evt, |id, evt| {
                        events.push_back((id, evt))
                    });
                }
                None => {
                    fail!("Could not find handler for {}", id);
                }
            }
        }

        self.events = Some(events);
    }

    pub fn add(&mut self, handler: ~Handler:Send+Share) -> ItemId {
        let id = self.count;
        self.count += 1;
        self.widgets.insert(id, handler);
        id
    }

    pub fn root(&mut self, root: ItemId) {
        self.root = root;
    }
}