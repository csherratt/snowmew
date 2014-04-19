use collections::deque::Deque;
use collections::ringbuf::RingBuf;
use collections::trie::TrieMap;

use {ItemId, Event, Handler, Mouse, MouseEvent, Window};
use glfw;

pub struct Manager {
    events: Option<RingBuf<(ItemId, Event)>>,
    widgets: TrieMap<~Handler>,
    root: ItemId,
    count: ItemId,
    mouse: Mouse,
    window: Window
}

fn to_index(button: glfw::MouseButton) -> uint {
    match button {
        glfw::MouseButton1 => 0,
        glfw::MouseButton2 => 1,
        glfw::MouseButton3 => 2,
        glfw::MouseButton4 => 3,
        glfw::MouseButton5 => 4,
        glfw::MouseButton6 => 5,
        glfw::MouseButton7 => 6,
        glfw::MouseButton8 => 7
    }
}

impl Manager {
    pub fn new() -> Manager {
        Manager {
            mouse: Mouse::new(),
            window: Window::new(),
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

    pub fn event_glfw(&mut self, _: f64, evt: glfw::WindowEvent) {
        let evt = match evt {
            glfw::CursorPosEvent(x, y) => {
                self.mouse.pos((x as f32, y as f32));
                Some(MouseEvent(self.mouse.clone()))
            },
            glfw::MouseButtonEvent(button, direction, _) => {
                match direction {
                    glfw::Press | glfw::Repeat => {
                        self.mouse.button[to_index(button)] = true;
                    },
                    glfw::Release => {
                        self.mouse.button[to_index(button)] = false;
                    }
                }
                Some(MouseEvent(self.mouse.clone()))
            },
            glfw::ScrollEvent(x, y) => {
                self.mouse.scroll((x as f32, y as f32));
                Some(MouseEvent(self.mouse.clone()))
            },
            glfw::CharEvent(_) => None,
            glfw::KeyEvent(_, _, _, _) => None,
            glfw::FramebufferSizeEvent(_, _) => None,
            glfw::IconifyEvent(_) => None,
            glfw::CloseEvent => None,
            glfw::SizeEvent(_, _) => None,
            glfw::PosEvent(_, _) => None,
            glfw::CursorEnterEvent(_) => None,
            glfw::FocusEvent(_) => None,
            glfw::RefreshEvent => None,
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
                    if id != 0 {
                        fail!("Could not find handler for {}", id);
                    } else {
                        println!("No handlers installed!");
                    }
                }
            }
        }

        self.events = Some(events);
    }

    pub fn add(&mut self, handler: ~Handler) -> ItemId {
        let id = self.count;
        self.count += 1;
        self.widgets.insert(id, handler);
        id
    }

    pub fn root(&mut self, root: ItemId) {
        self.root = root;
    }
}