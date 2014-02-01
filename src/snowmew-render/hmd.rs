use gl;
use gl::types::GLuint;
use db::Graphics;
use std::ptr;
use ovr::{HMDInfo};

use shader::Shader; 

pub struct HMD
{
    scale: f32,
    texture: GLuint,
    framebuffer: GLuint,
    renderbuffers: GLuint
}

impl HMD
{
    pub fn new(scale: f32, hmd: &HMDInfo) -> HMD
    {
        let (w, h) = hmd.resolution();
        let (w, h) = ((w as f32 * scale) as i32, (h as f32 * scale) as i32);
        let textures: &mut [GLuint] = &mut [0];
        let framebuffers: &mut [GLuint] = &mut [0];
        let renderbuffers: &mut [GLuint] = &mut [0];

        unsafe {
            gl::GenTextures(1, textures.unsafe_mut_ref(0));
            gl::GenFramebuffers(1, framebuffers.unsafe_mut_ref(0));
            gl::GenRenderbuffers(1, renderbuffers.unsafe_mut_ref(0));

            println!("{:?} {:?} {:?}", textures, framebuffers, renderbuffers);
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffers[0]);
            gl::BindTexture(gl::TEXTURE_2D, textures[0]);
        
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, w, h, 0, gl::RGB, gl::UNSIGNED_BYTE, ptr::null());
        
            gl::BindRenderbuffer(gl::RENDERBUFFER, renderbuffers[0]);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT, w, h);
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, renderbuffers[0]);

            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, textures[0], 0);

            let drawbuffers = &[gl::COLOR_ATTACHMENT0];
            gl::DrawBuffers(1, drawbuffers.unsafe_ref(0))
        }

        HMD {
            scale: scale,
            texture: textures[0],
            framebuffer: framebuffers[0],
            renderbuffers: renderbuffers[0]
        }
    }

    pub fn set_left(&self, _: &Graphics, hmd: &HMDInfo)
    {
        let (w, h) = hmd.resolution();
        let (w, h) = ((w as f32 * self.scale) as i32, (h as f32 * self.scale) as i32);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);

        gl::Viewport(0, 0, w/2, h);
        gl::Scissor(0, 0, w/2, h);

        gl::ClearColor(0., 0., 0., 1.);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    pub fn set_right(&self, _: &Graphics, hmd: &HMDInfo)
    {
        let (w, h) = hmd.resolution();
        let (w, h) = ((w as f32 * self.scale) as i32, (h as f32 * self.scale) as i32);
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);

        gl::Viewport(w/2, 0, w/2, h);
        gl::Scissor(w/2, 0, w/2, h);

        gl::ClearColor(0., 0., 0., 1.);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    fn setup_viewport(&self, shader: &Shader, vp: (f32, f32, f32, f32), ws: (f32, f32), offset: f32, scale: f32)
    {
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

    pub fn draw_screen(&self, db: &Graphics, hmd: &HMDInfo)
    {
        let billboard = db.current.find("core/geometry/billboard").unwrap();
        let shader = db.current.find("core/shaders/ovr_hmd").unwrap();

        let billboard = db.current.geometry(billboard).unwrap();
        let shader = db.shaders.find(&shader).unwrap();

        let vbo = db.vertex.find(&billboard.vb).unwrap();
        shader.bind();
        vbo.bind();

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::ClearColor(0., 0., 0., 1.);
        gl::Scissor(0, 0, 1280, 800);
        gl::Viewport(0, 0, 1280, 800);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        
        let (horz, _) = hmd.size();
        let (width, height) = hmd.resolution();
        let (width, height) = (width as f32, height as f32);
        let lens_separation_distance = hmd.lens_separation_distance();
        let lense_center = 1. - 2.*lens_separation_distance/horz;
        let scale = 1.;

//        print!("w: {} h: {} x: {} y: {}\n", w, h, x, y);
//        print!("center offset {}\n", lense_center);
//        print!("scaleFactor: {} as: {}\n", scale, aspect_ratio);
//        print!("lens center: {} {}\n", x + (w + lense_center * 0.5)*0.5, y + h*0.5);
//        print!("screen center: {} {}\n", x + w*0.5, y + h*0.5);
//        print!("scale out: {} {}\n", (w/2.) * scale, (h/2.) * scale * aspect_ratio);
//        print!("scale in: {} {}\n", (2./w),               (2./h) / aspect_ratio);

        unsafe {
            let distortion_K = hmd.distortion_K();
            gl::Uniform1i(shader.uniform("Texture0"), 0);
            gl::Uniform4fv(shader.uniform("HmdWarpParam"), 1, distortion_K.unsafe_ref(0));

            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            self.setup_viewport(shader, (0., 0.,       width/2., 800.), (width, height), lense_center, scale);
            gl::DrawElements(gl::TRIANGLES, billboard.count as i32, gl::UNSIGNED_INT, ptr::null());

            self.setup_viewport(shader, (width/2., 0., width/2., 800.), (width, height), -lense_center, scale);
            gl::DrawElements(gl::TRIANGLES, billboard.count as i32, gl::UNSIGNED_INT, ptr::null());
        }
    }
}