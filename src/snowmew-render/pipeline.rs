//   Copyright 2014 Colin Sherratt
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

use std::iter::range_step;
use libc;

use gl;
use gl::types::{GLuint, GLint};
use ovr::{PerEye, EyeRenderDescriptor, HmdDescription, DistortionCapabilities};
use ovr::{RenderGLConfig, EyeType, SensorCapabilities, EyeLeft, EyeRight};
use ovr::Texture;
use ovr::ll::Sizei;

use cgmath::Matrix;
use cgmath::{Vector4};
use cgmath::{Array1, Array2};

use graphics::Graphics;

use db::GlState;
use drawlist::Drawlist;

use snowmew::io::Window;
use snowmew::camera::DrawMatrices;
use snowmew::common::Common;
use snowmew::camera::Camera;

use query::Profiler;
use config::Config;

pub struct DrawTarget {
    framebuffer: GLuint,
    width: GLint,
    height: GLint,
    x: GLint,
    y: GLint,
    draw_buffers: &'static [u32]
}

impl DrawTarget {
    pub fn new(framebuffer: GLuint, offset: (int, int), size: (uint, uint), draw_buffers: &'static [u32]) -> DrawTarget {
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
            gl::DrawBuffers(self.draw_buffers.len() as i32, self.draw_buffers.unsafe_get(0))
        }
    }

    pub fn size(&self) -> (uint, uint) {
        (self.width as uint, self.height as uint)
    }

    pub fn to_vec4(&self) -> Vector4<f32> {
        Vector4::new(self.x as f32,
                     self.y as f32,
                     self.width as f32,
                     self.height as f32)
    }
}

pub trait Resize {
    fn resize(&mut self, _: uint, _: uint);
}

pub trait PipelineState: Resize {
    fn render(&mut self, drawlist: &mut Drawlist, db: &GlState, dm: &DrawMatrices, dt: &DrawTarget, q: &mut Profiler);
}

pub trait Pipeline: Resize {
    fn render(&mut self, drawlist: &mut Drawlist, db: &GlState, camera: &Camera, q: &mut Profiler);
}

pub struct Forward;

impl Forward {
    pub fn new() -> Forward { Forward }
}

impl PipelineState for Forward {
    fn render(&mut self, drawlist: &mut Drawlist, db: &GlState, dm: &DrawMatrices, dt: &DrawTarget, q: &mut Profiler) {
        q.time("forward setup".to_string());
        dt.bind();
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        q.time("forward render".to_string());
        drawlist.render(db, &dm.view, &dm.projection);
    }
}

impl Resize for Forward {
    fn resize(&mut self, _: uint, _: uint) {}
}

pub struct Defered<PIPELINE> {
    input: PIPELINE,

    width: i32,
    height: i32,

    uv_texture: GLuint,
    normals_texture: GLuint,
    material_texture: GLuint,
    depth_buffer: GLuint,
    dxdy_texture: GLuint,

    framebuffer: GLuint,
}

impl<PIPELINE: PipelineState> Defered<PIPELINE> {
    pub fn new(input: PIPELINE) -> Defered<PIPELINE> {
        let textures: &mut [GLuint] = &mut [0, 0, 0, 0, 0];
        let mut framebuffer: GLuint = 0;

        unsafe {
            gl::GenTextures(textures.len() as i32, textures.unsafe_mut(0));
            gl::GenFramebuffers(1, &mut framebuffer);
        }

        let mut new = Defered {
            input: input,

            width: 1024,
            height: 1024,

            uv_texture: textures[0],
            normals_texture: textures[1],
            material_texture: textures[2],
            depth_buffer: textures[3],
            dxdy_texture: textures[4],

            framebuffer: framebuffer,
        };

        new.setup_framebuffer(1024, 1024);

        new
    }

    fn draw_target(&self) -> DrawTarget {
        static DRAW_BUFFERS: &'static [u32] = &[gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1,
                                                gl::COLOR_ATTACHMENT2, gl::COLOR_ATTACHMENT3];
        DrawTarget {
            framebuffer: self.framebuffer,
            x: 0,
            y: 0,
            width: self.width,
            height: self.height,
            draw_buffers: DRAW_BUFFERS
        }
    }

    fn setup_framebuffer(&mut self, width: i32, height: i32) {
        let (w, h) = (width as i32, height as i32);
        let set_texture = |texture, gl_type| {
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexStorage2D(gl::TEXTURE_2D, 1, gl_type, w, h);
        };

        set_texture(self.uv_texture, gl::RG16F);
        set_texture(self.normals_texture, gl::RGB16F);
        set_texture(self.material_texture, gl::RG32UI);
        set_texture(self.dxdy_texture, gl::RGBA16F);
        set_texture(self.depth_buffer, gl::DEPTH_COMPONENT24);

        gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, self.uv_texture, 0);
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, self.normals_texture, 0);
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT2, self.material_texture, 0);
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT3, self.dxdy_texture, 0);
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, self.depth_buffer, 0);

        let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
        if status != gl::FRAMEBUFFER_COMPLETE {
            fail!("Failed to setup framebuffer {}", status);
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

        self.width = w;
        self.height = h;
    }

}

impl<PIPELINE: PipelineState> Resize for Defered<PIPELINE> {
    fn resize(&mut self, width: uint, height: uint) {
        if self.width == width as i32 && self.height == height as i32 {
            return;
        }

        let textures: &mut [GLuint] = &mut [self.uv_texture,
                                            self.normals_texture,
                                            self.material_texture,
                                            self.depth_buffer,
                                            self.dxdy_texture];

        unsafe {
            gl::DeleteTextures(textures.len() as i32, textures.unsafe_get(0));
            gl::GenTextures(textures.len() as i32, textures.unsafe_mut(0));
        }

        self.uv_texture = textures[0];
        self.normals_texture = textures[1];
        self.material_texture = textures[2];
        self.depth_buffer = textures[3];
        self.dxdy_texture = textures[4];

        let (w, h) = (width as i32, height as i32);
        self.setup_framebuffer(w, h);
    }
}

impl<PIPELINE: PipelineState> Defered<PIPELINE> {
    fn point_light(&mut self,
                   drawlist: &mut Drawlist,
                   db: &GlState,
                   dm: &DrawMatrices,
                   dt: &DrawTarget) {

        let plane = drawlist.standard_graphics().shapes.plane;
        let plane = drawlist.geometry(plane)
                .expect("Could not fetch geometry of plane");
        let shader = db.defered_shader_point_light
                .as_ref().expect("Could not load defered_shader");
        let vbo = db.vertex.find(&plane.vb)
                .expect("No vbo found");

        vbo.bind();
        shader.bind();

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, self.uv_texture);
        gl::Uniform1i(shader.uniform("uv"), 0);
        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_2D, self.normals_texture);
        gl::Uniform1i(shader.uniform("normal"), 1);
        gl::ActiveTexture(gl::TEXTURE2);
        gl::BindTexture(gl::TEXTURE_2D, self.material_texture);
        gl::Uniform1i(shader.uniform("pixel_drawn_by"), 2);
        gl::ActiveTexture(gl::TEXTURE3);
        gl::BindTexture(gl::TEXTURE_2D, self.depth_buffer);
        gl::Uniform1i(shader.uniform("depth"), 3);
        gl::ActiveTexture(gl::TEXTURE4);
        gl::BindTexture(gl::TEXTURE_2D, self.dxdy_texture);
        gl::Uniform1i(shader.uniform("dxdt"), 4);

        unsafe {
            gl::UniformMatrix4fv(shader.uniform("mat_proj"), 1, gl::FALSE, dm.projection.ptr());
            gl::UniformMatrix4fv(shader.uniform("mat_view"), 1, gl::FALSE, dm.view.ptr());
            gl::UniformMatrix4fv(shader.uniform("mat_inv_proj"), 1, gl::FALSE,
                dm.projection.invert().expect("failed to invert").ptr()
            );
            gl::UniformMatrix4fv(shader.uniform("mat_inv_view"), 1, gl::FALSE,
                dm.view.invert().expect("failed to invert").ptr()
            );
            gl::Uniform4fv(shader.uniform("viewport"), 1, dt.to_vec4().ptr());
        }


        let textures = db.texture.textures();
        let atlas_uniform = shader.uniform("atlas");
        let atlas_base = shader.uniform("atlas_base");

        let lights = shader.uniform_block_index("Lights");
        let materials = shader.uniform_block_index("Materials");

        gl::BindBufferBase(gl::UNIFORM_BUFFER, lights, drawlist.lights_buffer());
        shader.uniform_block_bind(lights, lights);

        gl::BindBufferBase(gl::UNIFORM_BUFFER, materials, drawlist.material_buffer());
        shader.uniform_block_bind(materials, materials);

        let texture_base = gl::TEXTURE7 - gl::TEXTURE0;
        let texture_range = gl::TEXTURE15 - gl::TEXTURE0 - texture_base;
        let text: Vec<i32> = range(texture_base as i32,
                                  (texture_base+texture_range) as i32).collect();

        unsafe {
            gl::Uniform1iv(atlas_uniform,
                           text.len() as i32,
                           (&text[0] as *const i32));
        } 

        let total_textures = if textures.len() == 0 { 1 } else { textures.len() };
        for idx in range_step(0, total_textures, texture_range as uint) {
            unsafe {
                if textures.len() != 0 {
                    let end = if idx + texture_range as uint > textures.len() {
                        textures.len()
                    } else {
                        idx + texture_range as uint
                    };
                    for (e, i) in range(idx, end).enumerate() {
                        gl::ActiveTexture(gl::TEXTURE0+texture_base+e as u32);
                        gl::BindTexture(gl::TEXTURE_2D_ARRAY, textures[i]);
                    }
                    gl::Uniform1i(atlas_base, idx as i32);
                }
                gl::DrawElements(gl::TRIANGLES,
                                 plane.count as i32,
                                 gl::UNSIGNED_INT,
                                 (plane.offset * 4) as *const libc::c_void);
            }
        }
    }
}

impl<PIPELINE: PipelineState> PipelineState for Defered<PIPELINE> {
    fn render(&mut self, drawlist: &mut Drawlist, db: &GlState, dm: &DrawMatrices, ddt: &DrawTarget, q: &mut Profiler) {
        let dt = self.draw_target();
        self.input.render(drawlist, db, dm, &dt, q);

        gl::Disable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::BlendEquation(gl::FUNC_ADD);
        gl::BlendFunc(gl::ONE, gl::ONE);

        q.time("defered: setup".to_string());
        ddt.bind();
        gl::ClearColor(0., 0., 0., 1.);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        q.time("defered: lighting".to_string());
        self.point_light(drawlist, db, dm, &dt);

        q.time("defered: cleanup".to_string());
        for i in range(0i, 16) {
            gl::ActiveTexture(gl::TEXTURE0 + i as u32);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
        }

        gl::Disable(gl::BLEND);
    }
}

pub struct Swap<PIPELINE> {
    input: PIPELINE,
    window: Window,
    width: uint,
    height: uint
}

impl<PIPELINE: PipelineState> Swap<PIPELINE> {
    pub fn new(input: PIPELINE, window: Window) -> Swap<PIPELINE> {
        Swap {
            input: input,
            window: window,
            width: 0,
            height: 0
        }
    }
}

impl<PIPELINE: PipelineState> Pipeline for Swap<PIPELINE> {
    fn render(&mut self, drawlist: &mut Drawlist, db: &GlState, camera: &Camera, q: &mut Profiler) {
        static DRAW_BUFFERS: &'static [u32] = &[gl::BACK_LEFT];
        let dt = DrawTarget::new(0, (0, 0), (self.width, self.height), DRAW_BUFFERS);
        let dm = camera.get_matrices((self.width as i32, self.height as i32));

        q.time("cull data".to_string());
        drawlist.cull(db, &dm.view, &dm.projection);

        self.input.render(drawlist, db, &dm, &dt, q);
        self.window.swap_buffers()
    }
}

impl<PIPELINE: PipelineState> Resize for Swap<PIPELINE> {
    fn resize(&mut self, width: uint, height: uint) {
        self.input.resize(width, height);
        self.width = width;
        self.height = height;
    }
}

pub struct Hmd<PIPELINE> {
    input: PIPELINE,

    textures: PerEye<GLuint>,
    framebuffers: PerEye<GLuint>,
    eye_desc: PerEye<EyeRenderDescriptor>,
    size: PerEye<Sizei>,
    desc: HmdDescription,
    window: Window,
    frame_index: uint

}

#[cfg(target_os="linux")]
fn render_config(window: &Window, hmd: &HmdDescription) -> RenderGLConfig {
    RenderGLConfig {
        size: hmd.resolution,
        multisample: 4,
        display: Some(window.get_x11_display() as *const libc::c_void),
        window: None
    }
}

#[cfg(target_os="macos")]
fn render_config(_: &Window, hmd: &HmdDescription) -> RenderGLConfig {
    RenderGLConfig {
        size: hmd.resolution,
        multisample: 4,
        display: None,
        window: None
    }
}

impl<PIPELINE: PipelineState> Hmd<PIPELINE> {
    pub fn new(input: PIPELINE, window: Window, cfg: &Config) -> Hmd<PIPELINE> {
        let hmd = window.get_hmd();
        let desc = hmd.get_description();
        let caps = SensorCapabilities::new().set_orientation(true);
        assert!(hmd.start_sensor(caps, caps));
        let dist = DistortionCapabilities::new()
                .set_chromatic(cfg.chromatic())
                .set_vignette(cfg.vignette())
                .set_timewarp(cfg.timewarp());

        let rc = render_config(&window, &desc);
        let eye_desc = hmd.configure_rendering(
                &rc,
                dist,
                desc.eye_fovs.map(|_, eye| eye.default_eye_fov)
        ).expect("Could not create hmd context");

        let size = desc.eye_fovs.map(|which, eye| {
            hmd.get_fov_texture_size(which, eye.default_eye_fov, cfg.hmd_size())
        });

        let mut textures: PerEye<GLuint> = PerEye::new(0, 0);
        let mut framebuffers: PerEye<GLuint> = PerEye::new(0, 0);

        unsafe {
            gl::GenTextures(2, textures.mut_ptr());
            gl::GenFramebuffers(2, framebuffers.mut_ptr());
        }

        size.map(|which, size| {
            gl::BindFramebuffer(gl::FRAMEBUFFER, *framebuffers.eye(which));
            gl::BindTexture(gl::TEXTURE_2D, *textures.eye(which));

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexStorage2D(gl::TEXTURE_2D, 1, gl::RGBA8, size.x, size.y);

            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, *textures.eye(which), 0);
        });
        gl::GetError();

        Hmd {
            input: input,
            textures: textures,
            framebuffers: framebuffers,
            desc: desc,
            eye_desc: eye_desc,
            size: size,
            window: window,
            frame_index: 0
        }
    }

    fn get_draw_target(&self, which: EyeType) -> DrawTarget {
        static DRAW_BUFFERS: &'static [u32] = &[gl::COLOR_ATTACHMENT0];

        DrawTarget {
            framebuffer: *self.framebuffers.eye(which),
            x: 0,
            y: 0,
            width: self.size.eye(which).x,
            height: self.size.eye(which).y,
            draw_buffers: DRAW_BUFFERS
        }
    }

    fn get_texture(&self, which: EyeType) -> Texture {
        let size = self.size.eye(which);
        let texture = self.textures.eye(which);
        Texture::new(size.x as int,
                     size.y as int,
                     0, 0,
                     size.x as int,
                     size.y as int,
                     *texture)
    }
}

impl<PIPELINE: PipelineState> Pipeline for Hmd<PIPELINE> {
    fn render(&mut self, drawlist: &mut Drawlist, db: &GlState, camera: &Camera, q: &mut Profiler) {
        let _ = self.window.get_hmd().begin_frame(self.frame_index);
        self.frame_index += 1;
        for &eye in [EyeLeft, EyeRight].iter() {
            let pose = self.window.get_hmd().begin_eye_render(eye);
            let dm = camera.ovr(&self.desc.eye_fovs.eye(eye).default_eye_fov, 
                                self.eye_desc.eye(eye),
                                &pose);

            let dt = self.get_draw_target(eye);
            let texture = self.get_texture(eye);
            self.input.render(drawlist, db, &dm, &dt, q);
            self.window.get_hmd().end_eye_render(eye, pose, &texture);
        }
        q.time("ovr: end_frame".to_string());
        self.window.get_hmd().end_frame();
        gl::GetError();
    }
}

impl<PIPELINE: PipelineState> Resize for Hmd<PIPELINE> {
    fn resize(&mut self, _: uint, _: uint) {
        let size = self.size.left;
        self.input.resize(size.x as uint, size.y as uint);
    }
}
