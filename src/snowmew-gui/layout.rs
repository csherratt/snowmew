use std::vec::Vec;

use {ItemId, Handler, Event};


struct Item {
    start: (f32, f32),
    size: (f32, f32),
    z: f32,
    id: ItemId
}

pub struct Layout {
    items: Vec<Item>
}

impl Layout {
    pub fn new() -> Layout {
        Layout {
            items: Vec::new()
        }
    }

    pub fn add(&mut self, start: (f32, f32), size: (f32, f32), z: f32, id: ItemId) {
        self.items.push(Item {
            start: start,
            size: size,
            z: z,
            id: id
        });
    }

    pub fn get_item(&self, x: f32, y: f32) -> Option<ItemId> {
        let mut found_item = None;
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

impl Handler for Layout {
    fn handle(&mut self, evt: Event, queue: |id: ItemId, evt: Event|) {

    }
}