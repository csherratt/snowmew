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

pub enum Event {
    MouseMove((f32, f32), (f32, f32))
}

pub trait Handler {
    fn handle(&mut self, evt: Event, queue: |id: ItemId, evt: Event|);
}

