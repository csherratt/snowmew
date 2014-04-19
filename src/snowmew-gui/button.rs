use {ItemId, Handler, ButtonPressed, ButtonReleased, MouseEvent, Event};

enum ButtonState {
    Pressed,
    Released
}

pub struct Button {
    state: ButtonState,
    id: Option<ItemId>
}

impl Button {
    pub fn new() -> Button {
        Button {
            state: Released,
            id: None
        }
    }

    pub fn setup(&mut self, id: ItemId) {
        self.id = Some(id);
    }
}

impl Handler for Button {
    fn handle(&mut self, evt: Event, queue: |id: ItemId, evt: Event|) {
        match (evt, self.state) {
            (MouseEvent(m), Released) => {
                if m.button[0] == true {
                    self.state = Pressed;
                    if self.id.is_some() {
                        queue(self.id.unwrap(), ButtonPressed)
                    }
                }
            },
            (MouseEvent(m), Pressed) => {
                if m.button[0] == false {
                    self.state = Released;
                    if self.id.is_some() {
                        queue(self.id.unwrap(), ButtonReleased)
                    }
                }
            },
            (_, _) => ()
        }
    }
}