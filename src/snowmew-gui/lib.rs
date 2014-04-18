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
    pub pos: (f32, f32),
    pub global: (f32, f32),
    pub button: [bool, ..8]
}

impl Mouse {
    pub fn new() -> Mouse {
        Mouse {
            pos: (0., 0.),
            global: (0., 0.),
            button: [false, false, false, false,
                     false, false, false, false]
        }
    }

    pub fn next(&self, pos: (f32, f32)) -> Mouse {
        let (x, y) = pos;
        let (ox, oy) = self.pos;

        Mouse {
            pos: (ox-x, oy-y),
            global: self.global,
            button: self.button
        }
    }
}

pub enum Event {
    MouseEvent(Mouse)
}

pub trait Handler {
    fn handle(&mut self, evt: Event, queue: |id: ItemId, evt: Event|);
}

