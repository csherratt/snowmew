
use gl;
use gl::types::GLuint;
use db::Graphics;

pub struct DrawTarget
{
    framebuffer: uint,
    width: uint,
    height: uint,
    x: uint,
    y: uint
}

impl DrawTarget
{
    pub fn bind(&self)
    {
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        gl::Viewport(self.x, self.y, self.width, self.height);
        gl::Scissor(self.x, self.y, self.width, self.height);
    }
}

trait Pipeline
{
    fn render(&mut self, drawlist: &Drawlist, db: &Graphics, camera: object_key, dt: &DrawTarget);
}

pub struct Forward;

impl Forward
{
    fn new() -> Forward
    {
        Forward
    }
}

impl Pipeline for Forward
{
    fn render(&mut self, drawlist: &Drawlist, db: &Graphics, camera: object_key, dt: &DrawTarget)
    {
        drawlist.render();
        dt.bind();
    }
}