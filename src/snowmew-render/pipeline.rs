
use std::ptr;

use gl;
use gl::types::{GLuint, GLint};
use cgmath::matrix::Matrix;
use cgmath::vector::Vector3;
use ovr::HMDInfo;

use shader::Shader; 

use snowmew::camera::DrawMatrices;
use graphics::material::{NoMaterial, Phong, Flat};
use graphics::Graphics;

use db::GlState;
use drawlist::Drawlist;

use snowmew::common::Common;

use query::Profiler;

pub struct DrawTarget {
    framebuffer: GLuint,
    width: GLint,
    height: GLint,
    x: GLint,
    y: GLint,
    draw_buffers: ~[u32]
}

impl DrawTarget {
    pub fn new(framebuffer: GLuint, offset: (int, int), size: (uint, uint), draw_buffers: ~[u32]) -> DrawTarget {
        let (x, y) = offset;
        let (width, height) = size;
        DrawTarget {
            framebuffer: framebuffer,
            width: width as GLint,
            height: height as GLint,
            x: x as GLint,
            y: y as GLint,
            draw_buffers: draw_buffers 
        }
    }

    pub fn bind(&self) {
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        gl::Viewport(self.x, self.y, self.width, self.height);
        gl::Scissor(self.x, self.y, self.width, self.height);

        unsafe {
            gl::DrawBuffers(self.draw_buffers.len() as i32, self.draw_buffers.unsafe_ref(0))
        }
    }

    pub fn size(&self) -> (uint, uint) {
        (self.width as uint, self.height as uint)
    }
}

pub trait Pipeline {
    fn render(&mut self, drawlist: &mut Drawlist, db: &GlState, dm: &DrawMatrices, dt: &DrawTarget, q: &mut Profiler);
}

pub struct Forward;

impl Forward {
    pub fn new() -> Forward { Forward }
}

impl Pipeline for Forward {
    fn render(&mut self, drawlist: &mut Drawlist, db: &GlState, dm: &DrawMatrices, dt: &DrawTarget, q: &mut Profiler) {
        q.time("forward setup".to_owned());
        dt.bind();
        gl::ClearColor(0., 0., 0., 1.);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        let proj_view = dm.projection.mul_m(&dm.view);
        q.time("forward render".to_owned());
        drawlist.render(db, proj_view);
    }
}

pub struct Defered<PIPELINE> {
    input: PIPELINE,

    width: i32,
    height: i32,

    pos_texture: GLuint,
    uv_texture: GLuint,
    normals_texture: GLuint,
    material_texture: GLuint,

    framebuffer: GLuint,
    renderbuffer: GLuint,
}

impl<PIPELINE: Pipeline> Defered<PIPELINE> {
    pub fn new(input: PIPELINE, width: uint, height: uint) -> Defered<PIPELINE> {
        let (w, h) = (width as i32, height as i32);
        let textures: &mut [GLuint] = &mut [0, 0, 0, 0];
        let mut framebuffer: GLuint = 0;
        let mut renderbuffer: GLuint = 0;

        unsafe {
            gl::GenTextures(textures.len() as i32, textures.unsafe_mut_ref(0));
            gl::GenFramebuffers(1, &mut framebuffer);
            gl::GenRenderbuffers(1, &mut renderbuffer);

            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
            assert!(0 == gl::GetError());

            // setup pos 
            gl::BindTexture(gl::TEXTURE_2D, textures[0]);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexStorage2D(gl::TEXTURE_2D, 1, gl::RGBA16F, w, h);
            assert!(0 == gl::GetError());

            // setup UV texture
            gl::BindTexture(gl::TEXTURE_2D, textures[1]);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexStorage2D(gl::TEXTURE_2D, 1, gl::RG16F, w, h);
            assert!(0 == gl::GetError());

            // setup normals
            gl::BindTexture(gl::TEXTURE_2D, textures[2]);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexStorage2D(gl::TEXTURE_2D, 1, gl::RGB16F, w, h);
            assert!(0 == gl::GetError());

            // setup material texture
            gl::BindTexture(gl::TEXTURE_2D, textures[3]);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexStorage2D(gl::TEXTURE_2D, 1, gl::RG32UI, w, h);
            assert!(0 == gl::GetError());

            gl::BindRenderbuffer(gl::RENDERBUFFER, renderbuffer);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT32F, w, h);
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, renderbuffer);
            assert!(0 == gl::GetError());

            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, textures[0], 0);
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, textures[1], 0);
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT2, textures[2], 0);
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT3, textures[3], 0);
            assert!(0 == gl::GetError());

            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                fail!("Failed to setup framebuffer {}", status);
            }
        }

        Defered {
            input: input,

            width: w,
            height: h,

            pos_texture: textures[0],
            uv_texture: textures[1],
            normals_texture: textures[2],
            material_texture: textures[3],

            framebuffer: framebuffer,
            renderbuffer: renderbuffer
        }
    }

    fn draw_target(&self) -> DrawTarget {
        DrawTarget {
            framebuffer: self.framebuffer,
            x: 0,
            y: 0,
            width: self.width,
            height: self.height,
            draw_buffers: ~[gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1,
                            gl::COLOR_ATTACHMENT2, gl::COLOR_ATTACHMENT3]
        }
    }
}

impl<PIPELINE: Pipeline> Pipeline for Defered<PIPELINE> {
    fn render(&mut self, drawlist: &mut Drawlist, db: &GlState, dm: &DrawMatrices, ddt: &DrawTarget, q: &mut Profiler) {
        let dt = self.draw_target();
        self.input.render(drawlist, db, dm, &dt, q);
        q.time("defered: setup".to_owned());

        let billboard = drawlist.find("core/geometry/billboard").unwrap();
        let billboard = drawlist.geometry(billboard).unwrap();

        let shader = db.defered_shader.as_ref().unwrap();

        let vbo = db.vertex.find(&billboard.vb).unwrap();
        vbo.bind();

        shader.bind();

        assert!(0 == gl::GetError());
        vbo.bind();
        assert!(0 == gl::GetError());

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, self.pos_texture);
        gl::Uniform1i(shader.uniform("position"), 0);
        gl::ActiveTexture(gl::TEXTURE0+1);
        gl::BindTexture(gl::TEXTURE_2D, self.uv_texture);
        gl::Uniform1i(shader.uniform("uv"), 1);
        gl::ActiveTexture(gl::TEXTURE0+2);
        gl::BindTexture(gl::TEXTURE_2D, self.normals_texture);
        gl::Uniform1i(shader.uniform("normal"), 2);
        gl::ActiveTexture(gl::TEXTURE0+3);
        gl::BindTexture(gl::TEXTURE_2D, self.material_texture);
        gl::Uniform1i(shader.uniform("pixel_drawn_by"), 3);
        assert!(0 == gl::GetError());

        let materials = drawlist.materials();
        let mut gl_materials = Vec::new();

        for m in materials.iter() {
            match *m {
                NoMaterial | Phong(_) => gl_materials.push(Vector3::new(1f32, 1f32, 1f32)),
                Flat(mat) => gl_materials.push(mat.clone())
            }
        }

        unsafe {
            gl::Uniform3fv(shader.uniform("mat_color"), gl_materials.len() as i32, &gl_materials.get(0).x);
        }

        ddt.bind();
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        q.time("defered: shader".to_owned());
        unsafe {
            assert!(0 == gl::GetError());
            gl::DrawElements(gl::TRIANGLES, billboard.count as i32, gl::UNSIGNED_INT, ptr::null());
            assert!(0 == gl::GetError());
        }

        q.time("defered: cleanup".to_owned());
        for i in range(0, 16) {
            gl::ActiveTexture(gl::TEXTURE0 + i as u32);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}

pub struct Hmd<PIPELINE> {
    input: PIPELINE,

    scale: f32,
    texture: GLuint,
    framebuffer: GLuint,
    renderbuffer: GLuint,

    hmd: HMDInfo
}

impl<PIPELINE: Pipeline> Hmd<PIPELINE> {
    pub fn new(input: PIPELINE, scale: f32, hmd: &HMDInfo) -> Hmd<PIPELINE> {
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

            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, textures[0], 0);
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

    fn left(&self) -> DrawTarget {
        let (w, h) = self.hmd.resolution();
        let (w, h) = ((w as f32 * self.scale) as i32, (h as f32 * self.scale) as i32);

        DrawTarget {
            framebuffer: self.framebuffer,
            x: 0,
            y: 0,
            width: w/2,
            height: h,
            draw_buffers: ~[gl::COLOR_ATTACHMENT0]
        }
    }

    fn right(&self) -> DrawTarget {
        let (w, h) = self.hmd.resolution();
        let (w, h) = ((w as f32 * self.scale) as i32, (h as f32 * self.scale) as i32);

        DrawTarget {
            framebuffer: self.framebuffer,
            x: w/2,
            y: 0,
            width: w/2,
            height: h,
            draw_buffers: ~[gl::COLOR_ATTACHMENT0]
        }
    }

    fn setup_viewport(&self, shader: &Shader, vp: (f32, f32, f32, f32), ws: (f32, f32), offset: f32) {
        let scale = 1./self.scale;

        let (vpx, vpy, vpw, vph) = vp;
        let (wsw, wsh) = ws;
        let (w, h, x, y) = (vpw/wsw, vph/wsh, vpx/wsw, vpy/wsh);

        let lens_center = &[x + (w + offset * 0.5)*0.5, y + h*0.5];
        let screen_center = &[x + w*0.5, y + h*0.5];

        let aspect_ratio = vpw / vph;

        let scale_out: &[f32] = &[scale * (w/2.),  scale * (h/2.) * aspect_ratio];
        let scale_in: &[f32] = &[2./w, (2./h) / aspect_ratio];

        //gl::Viewport(vpx as i32, vpy as i32, vpw as i32, vph as i32);
        gl::Scissor(vpx as i32, vpy as i32, vpw as i32, vph as i32);
        gl::Uniform2f(shader.uniform("ScreenCenter"), screen_center[0], screen_center[1]);
        gl::Uniform2f(shader.uniform("LensCenter"), lens_center[0], lens_center[1]);
        gl::Uniform2f(shader.uniform("ScaleIn"), scale_in[0], scale_in[1]);
        gl::Uniform2f(shader.uniform("ScaleOut"), scale_out[0], scale_out[1]);
    }

    fn draw_screen(&self, rd: &Drawlist, db: &GlState, dt: &DrawTarget) {
        let billboard = rd.find("core/geometry/billboard").unwrap();
        let billboard = rd.geometry(billboard).unwrap();

        let shader = db.ovr_shader.as_ref().unwrap();

        let vbo = db.vertex.find(&billboard.vb).unwrap();
        shader.bind();
        vbo.bind();

        dt.bind();
        let (width, height) = self.hmd.resolution();
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
            let chrom_ab_param = self.hmd.chroma_ab_correction();
            gl::Uniform1i(shader.uniform("Texture0"), 0);
            gl::Uniform4fv(shader.uniform("HmdWarpParam"), 1, distortion_K.unsafe_ref(0));
            gl::Uniform4fv(shader.uniform("ChromAbParam"), 1, chrom_ab_param.unsafe_ref(0));

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            self.setup_viewport(shader, (0., 0.,       width/2., height as f32), (width, height), lense_center);
            gl::DrawElements(gl::TRIANGLES, billboard.count as i32, gl::UNSIGNED_INT, ptr::null());

            self.setup_viewport(shader, (width/2., 0., width/2., height as f32), (width, height), -lense_center);
            gl::DrawElements(gl::TRIANGLES, billboard.count as i32, gl::UNSIGNED_INT, ptr::null());
        }
    }
}

impl<PIPELINE: Pipeline> Pipeline for Hmd<PIPELINE> {
    fn render(&mut self, drawlist: &mut Drawlist, db: &GlState, dm: &DrawMatrices, dt: &DrawTarget, q: &mut Profiler) {
        let (left_dm, right_dm) = dm.ovr(&self.hmd, self.scale);

        let left = self.left();
        self.input.render(drawlist, db, &left_dm, &left, q);

        let right = self.right();
        self.input.render(drawlist, db, &right_dm, &right, q);

        q.time("ovr: draw screen".to_owned());
        self.draw_screen(drawlist, db, dt);
    }
}