use geometry::Geometry;
use shader::Shader;
use coregl::{Uniforms, Texture};

pub trait DrawSize {
    fn size(&self) -> (uint, uint);
}

pub trait DrawTarget: DrawSize {
    fn draw(&mut self, &Shader, &Geometry, &[(i32, &Uniforms)], &[&Texture]);
}

pub trait FrameBuffer: DrawSize {
    fn viewport(&mut self, offset :(uint, uint), size :(uint, uint), f: &fn(&mut DrawTarget));
}

pub struct FrameInfo {
    count: uint,  /* unique frame identifier */
    time: f64,    /* current time in seconds */
    delta: f64,   /* time from last frame */
}

pub trait Object  {
    fn setup(&mut self, frame: &FrameInfo);

    fn draw(&mut self, frame: &FrameInfo, target: &mut DrawTarget);
}

pub struct Render  {
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