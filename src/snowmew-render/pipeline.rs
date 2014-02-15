
use std::ptr;

use gl;
use gl::types::{GLuint, GLint};
use cgmath::matrix::Matrix;
use ovr::HMDInfo;

use shader::Shader; 

use snowmew::camera::DrawMatrices;

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
    fn render(&mut self, drawlist: &mut Drawlist, db: &Graphics, dm: &DrawMatrices, dt: &DrawTarget);
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
    fn render(&mut self, drawlist: &mut Drawlist, db: &Graphics, dm: &DrawMatrices, dt: &DrawTarget)
    {
        dt.bind();
        gl::ClearColor(0., 0., 0., 1.);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        let proj_view = dm.projection.mul_m(&dm.view);

        drawlist.render(db, proj_view);
    }
}

pub struct Hmd<PIPELINE>
{
    input: PIPELINE,

    scale: f32,
    texture: GLuint,
    framebuffer: GLuint,
    renderbuffer: GLuint,

    hmd: HMDInfo
}

impl<PIPELINE: Pipeline> Hmd<PIPELINE>
{
    pub fn new(input: PIPELINE, scale: f32, hmd: &HMDInfo) -> Hmd<PIPELINE>
    {
        let (w, h) = hmd.resolution();
        let (w, h) = ((w as f32 * scale) as i32, (h as f32 * scale) as i32);
        let textures: &mut [GLuint] = &mut [0];
        let framebuffers: &mut [GLuint] = &mut [0];
        let renderbuffer: &mut [GLuint] = &mut [0];

        unsafe {
            gl::GenTextures(1, textures.unsafe_mut_ref(0));
            gl::GenFramebuffers(1, framebuffers.unsafe_mut_ref(0));
            gl::GenRenderbuffers(1, renderbuffer.unsafe_mut_ref(0));

            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffers[0]);
            gl::BindTexture(gl::TEXTURE_2D, textures[0]);
        
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, w, h, 0, gl::RGB, gl::UNSIGNED_BYTE, ptr::null());
        
            gl::BindRenderbuffer(gl::RENDERBUFFER, renderbuffer[0]);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT, w, h);
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, renderbuffer[0]);

            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, textures[0], 0);

            let drawbuffers = &[gl::COLOR_ATTACHMENT0];
            gl::DrawBuffers(1, drawbuffers.unsafe_ref(0))
        }

        Hmd {
            input: input,
            scale: scale,
            texture: textures[0],
            framebuffer: framebuffers[0],
            renderbuffer: renderbuffer[0],
            hmd: hmd.clone()
        }        
    }

    fn left(&self) -> DrawTarget
    {
        let (w, h) = self.hmd.resolution();
        let (w, h) = ((w as f32 * self.scale) as i32, (h as f32 * self.scale) as i32);

        DrawTarget {
            framebuffer: self.framebuffer,
            x: 0,
            y: 0,
            width: w/2,
            height: h
        }
    }

    fn right(&self) -> DrawTarget
    {
        let (w, h) = self.hmd.resolution();
        let (w, h) = ((w as f32 * self.scale) as i32, (h as f32 * self.scale) as i32);

        DrawTarget {
            framebuffer: self.framebuffer,
            x: w/2,
            y: 0,
            width: w/2,
            height: h
        }
    }

    fn setup_viewport(&self, shader: &Shader, vp: (f32, f32, f32, f32), ws: (f32, f32), offset: f32)
    {
        let scale = 1./self.scale;

        let (vpx, vpy, vpw, vph) = vp;
        let (wsw, wsh) = ws;
        let (w, h, x, y) = (vpw/wsw, vph/wsh, vpx/wsw, vpy/wsh);

        let lens_center = &[x + (w + offset * 0.5)*0.5, y + h*0.5];
        let screen_center = &[x + w*0.5, y + h*0.5];

        let aspect_ratio = vpw / vph;

        let scale_out: &[f32] = &[scale * (w/2.),  scale * (h/2.) * aspect_ratio];
        let scale_in: &[f32] = &[2./w, (2./h) / aspect_ratio];

        gl::Scissor(vpx as i32, vpy as i32, vpw as i32, vph as i32);
        gl::Uniform2f(shader.uniform("ScreenCenter"), screen_center[0], screen_center[1]);
        gl::Uniform2f(shader.uniform("LensCenter"), lens_center[0], lens_center[1]);
        gl::Uniform2f(shader.uniform("ScaleIn"), scale_in[0], scale_in[1]);
        gl::Uniform2f(shader.uniform("ScaleOut"), scale_out[0], scale_out[1]);
    }

    fn draw_screen(&self, db: &Graphics, dt: &DrawTarget)
    {
        let billboard = db.current.find("core/geometry/billboard").unwrap();
        let billboard = db.current.geometry(billboard).unwrap();

        let shader = db.ovr_shader.as_ref().unwrap();

        let vbo = db.vertex.find(&billboard.vb).unwrap();
        shader.bind();
        vbo.bind();

        dt.bind();
        let (width, height) = self.hmd.resolution();
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::ClearColor(0., 0., 0., 1.);
        gl::Scissor(0, 0, width as i32, height as i32);
        gl::Viewport(0, 0, width as i32, height as i32);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        
        let (horz, _) = self.hmd.size();
        let lens_separation_distance = self.hmd.lens_separation_distance();
        let lense_center = 1. - 2.*lens_separation_distance/horz;

        let (width, height) = (width as f32, height as f32);
        unsafe {
            let distortion_K = self.hmd.distortion_K();
            let ChromAbParam = self.hmd.chroma_ab_correction();
            gl::Uniform1i(shader.uniform("Texture0"), 0);
            gl::Uniform4fv(shader.uniform("HmdWarpParam"), 1, distortion_K.unsafe_ref(0));
            gl::Uniform4fv(shader.uniform("ChromAbParam"), 1, ChromAbParam.unsafe_ref(0));

            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            self.setup_viewport(shader, (0., 0.,       width/2., height as f32), (width, height), lense_center);
            gl::DrawElements(gl::TRIANGLES, billboard.count as i32, gl::UNSIGNED_INT, ptr::null());

            self.setup_viewport(shader, (width/2., 0., width/2., height as f32), (width, height), -lense_center);
            gl::DrawElements(gl::TRIANGLES, billboard.count as i32, gl::UNSIGNED_INT, ptr::null());
        }
    }
}

impl<PIPELINE: Pipeline> Pipeline for Hmd<PIPELINE>
{
    fn render(&mut self, drawlist: &mut Drawlist, db: &Graphics, dm: &DrawMatrices, dt: &DrawTarget)
    {
        let (left_dm, right_dm) = dm.ovr(&self.hmd);

        let left = self.left();
        self.input.render(drawlist, db, &left_dm, &left);

        let right = self.right();
        self.input.render(drawlist, db, &right_dm, &right);

        self.draw_screen(db, dt);
    }
}