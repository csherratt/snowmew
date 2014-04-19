use std::vec::Vec;

use {ItemId};

pub struct Repeater {
    ids: Vec<ItemId>
}

impl Repeater {
    pub fn new() -> Repeater {
        Repeater {
            ids: Vec::new()
        }
    }

    pub fn add(&mut self, id: ItemId) {
        self.ids.push(id)
    }
}