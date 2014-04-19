#![crate_id = "github.com/csherratt/snowmew#snowmew-gui:0.1"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "A game engine in rust"]

extern crate collections;
extern crate glfw;

pub use manager::Manager;
pub use layout::Layout;

mod layout;
mod manager;

pub type ItemId = uint;

pub struct Mouse {
    pub delta: (f32, f32),
    pub pos: (f32, f32),
    pub global: (f32, f32),
    pub scroll: (f32, f32),
    pub scroll_delta: (f32, f32),
    pub button: [bool, ..8]
}

impl Clone for Mouse {
    fn clone(&self) -> Mouse {
        Mouse {
            pos: self.pos,
            global: self.global,
            button: self.button,
            delta: self.delta,
            scroll: self.scroll,
            scroll_delta: self.scroll_delta
        }
    }
}

impl Mouse {
    pub fn new() -> Mouse {
        Mouse {
            pos: (0., 0.),
            global: (0., 0.),
            delta: (0., 0.),
            scroll: (0., 0.),
            scroll_delta: (0., 0.),
            button: [false, false, false, false,
                     false, false, false, false]
        }
    }

    pub fn pos(&mut self, pos: (f32, f32)) {
        let (ox, oy) = self.global;
        let (x, y) = pos;

        self.delta = (ox-x, oy-y);
        self.pos = pos;
        self.global = pos;
    }

    pub fn offset(&self, pos: (f32, f32)) -> Mouse {
        let (x, y) = pos;
        let (ox, oy) = self.pos;

        Mouse {
            pos: (ox-x, oy-y),
            delta: self.delta,
            global: self.global,
            button: self.button,
            scroll_delta: self.scroll_delta,
            scroll: self.scroll
        }
    }

    pub fn scroll(&mut self, delta: (f32, f32)) {
        let (x, y) = delta;
        let (sx, sy) = self.scroll;

        self.scroll_delta = delta;
        self.scroll = (x+sx, y+sy);
    }
}

#[deriving(Clone)]
pub struct Window {
    pub global: (f32, f32),
    pub size: (f32, f32)
}

impl Window {
    fn new() -> Window {
        Window {
            global: (0., 0.),
            size: (0., 0.)
        }
    }
}

pub enum Event {
    MouseEvent(Mouse),
    WindowEvent(Window),
}

pub trait Handler {
    fn handle(&mut self, evt: Event, queue: |id: ItemId, evt: Event|);
}

