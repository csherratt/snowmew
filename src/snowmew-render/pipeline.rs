
use gl;
use gl::types::{GLuint, GLint};
use cgmath::matrix::{Mat4, Matrix};

use snowmew::camera::Camera;

use db::Graphics;
use drawlist::Drawlist;


pub struct DrawTarget
{
    framebuffer: GLuint,
    width: GLint,
    height: GLint,
    x: GLint,
    y: GLint
}

impl DrawTarget
{
    pub fn new(framebuffer: GLuint, offset: (int, int), size: (uint, uint)) -> DrawTarget
    {
        let (x, y) = offset;
        let (width, height) = size;
        DrawTarget {
            framebuffer: framebuffer,
            width: width as GLint,
            height: height as GLint,
            x: x as GLint,
            y: y as GLint
        }
    }

    pub fn bind(&self)
    {
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        gl::Viewport(self.x, self.y, self.width, self.height);
        gl::Scissor(self.x, self.y, self.width, self.height);
    }

    pub fn size(&self) -> (uint, uint)
    {
        (self.width as uint, self.height as uint)
    }
}

pub trait Pipeline
{
    fn render(&mut self, drawlist: &mut Drawlist, db: &Graphics, camera: &Camera, dt: &DrawTarget);
}

pub struct Forward;

impl Forward
{
    pub fn new() -> Forward
    {
        Forward
    }
}

impl Pipeline for Forward
{
    fn render(&mut self, drawlist: &mut Drawlist, db: &Graphics, camera: &Camera, dt: &DrawTarget)
    {
        dt.bind();
        gl::ClearColor(0., 0., 0., 1.);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        let camera = camera.get_matrices(dt.size());
        let proj_view = camera.projection.mul_m(&camera.view);

        drawlist.render(db, proj_view);
    }
}

pub struct Hmd<PIPELINE>
{
    input: PIPELINE,

    scale: f32,
    texture: GLuint,
    framebuffer: GLuint,
    renderbuffer: GLuint
}