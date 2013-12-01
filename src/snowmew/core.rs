use geometry::Geometry;
use shader::Shader;
use coregl::{Uniforms, Texture};
use render::Context;

pub trait DrawSize {
    fn size(&self) -> (uint, uint);
}

pub trait DrawTarget: DrawSize {
    fn draw(&mut self, ctx: &mut Context, &Shader, &Geometry, &[(i32, &Uniforms)], &[&Texture]);
}

pub trait FrameBuffer: DrawSize {
    fn viewport(&mut self, ctx: &mut Context, offset :(uint, uint), size :(uint, uint), f: |&mut DrawTarget, ctx: &mut Context|);
}

pub struct FrameInfo {
    count: uint,  /* unique frame identifier */
    time: f64,    /* current time in seconds */
    delta: f64,   /* time from last frame */
}

pub trait Object  {
    fn setup(&mut self, ctx: &mut Context, frame: &FrameInfo, target: &DrawTarget);

    fn draw(&mut self, ctx: &mut Context, frame: &FrameInfo, target: &mut DrawTarget);
}

pub struct Render  {
    fb: ~FrameBuffer,
    root: ~Object,
    ctx: Context
}

impl Render {
    fn draw(&mut self, fi: &FrameInfo) {
        let (w, h) = self.fb.size();
        self.fb.viewport(&mut self.ctx, (0, 0), (w, h), |viewport, ctx| {
            self.root.setup(ctx, fi, viewport);
            self.root.draw(ctx, fi, viewport);
        });
    }
}