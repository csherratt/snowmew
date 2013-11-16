pub enum DrawType {
    TRIANGLE,
}

pub trait DrawSize {
    fn size(&self) -> (uint, uint);
}

pub trait DrawTarget: DrawSize {
    fn draw(&mut self /* shader, */ , DrawType /* uniform<s>, geometry<s>, texture<s> */);
}

pub trait FrameBuffer: DrawSize {
    fn viewport(&mut self, offset :(uint, uint), size :(uint, uint), f: &fn(&mut DrawTarget));
}

pub struct FrameInfo {
    frame_count: uint,  /* unique frame identifier */
    frame_time: f64,    /* current time in seconds */
    frame_delta: f64,   /* time from last frame */
}

pub trait Object {
    fn setup(&mut self, frame: &FrameInfo);

    fn draw(&mut self, frame: &FrameInfo, target: &mut DrawTarget);
}

pub struct Render {
    fb: ~FrameBuffer,
    root: ~Object
}

impl Render {
    fn draw(&mut self, fi: &FrameInfo) {
        let (w, h) = self.fb.size();
        do self.fb.viewport((0, 0), (w, h)) |viewport| {
            self.root.setup(fi);
            self.root.draw(fi, viewport);
        }
    }
}