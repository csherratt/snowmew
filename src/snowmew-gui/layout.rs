use std::vec::Vec;

use {ItemId, Handler, Event, MouseEvent};


struct Item {
    start: (f32, f32),
    size: (f32, f32),
    z: f32,
    id: ItemId
}

pub struct Layout {
    default: Option<ItemId>,
    items: Vec<Item>
}

impl Layout {
    pub fn new() -> Layout {
        Layout {
            default: None,
            items: Vec::new()
        }
    }

    pub fn default(&mut self, id: ItemId) {
        self.default = Some(id);
    }

    pub fn add(&mut self, start: (f32, f32), size: (f32, f32), z: f32, id: ItemId) {
        self.items.push(Item {
            start: start,
            size: size,
            z: z,
            id: id
        });
    }

    pub fn pos(&self, id: ItemId) -> Option<(f32, f32)> {
        for item in self.items.iter() {
            if item.id == id {
                return Some(item.start);
            }
        }
        None
    }

    pub fn get_item(&self, x: f32, y: f32) -> Option<ItemId> {
        let mut found_item = self.default;
        let mut item_z = 0.;

        for item in self.items.iter() {
            let (sx, sy) = item.start;
            let (mut ex, mut ey) = item.size;
            ex += sx;
            ey += sy;

            if x < ex && x >= sx && y < ey && y >= sy {
                if item_z < item.z {
                    item_z = item.z;
                    found_item = Some(item.id);
                }
            }
        }

        found_item
    }
}

impl Handler<Event> for Layout {
    fn handle(&mut self, evt: Event, queue: |id: ItemId, evt: Event|) {
        match evt {
            MouseEvent(evt) => {
                let (lx, ly) = evt.pos;
                match self.get_item(lx, ly) {
                    Some(id) => {
                        let old = self.pos(id);
                        match old {
                            Some(old) => queue(id, MouseEvent(evt.offset(old))),
                            None => queue(id, MouseEvent(evt)) 
                        }
                    }
                    None => ()
                }
            },
            _ => ()
        }
    }
}