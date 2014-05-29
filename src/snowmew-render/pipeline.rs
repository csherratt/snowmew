
use std::iter::range_step;
use libc;

use gl;
use gl::types::{GLuint, GLint};
use ovr::{PerEye, EyeRenderDescriptor, HmdDescription, DistortionCapabilities};
use ovr::{RenderGLConfig, EyeType, SensorCapabilities, EyeLeft, EyeRight};
use ovr::Texture;
use ovr::ll::Sizei;

use cgmath::matrix::Matrix;
use cgmath::vector::{Vector4};
use cgmath::ptr::Ptr;

use graphics::{PointLight, Graphics};
use position::Positions;

use db::GlState;
use drawlist::Drawlist;

use snowmew::io::Window;
use snowmew::camera::DrawMatrices;
use snowmew::common::Common;
use snowmew::camera::Camera;
use snowmew::ObjectKey;

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
        q.time("forward setup".to_owned());
        dt.bind();
        gl::ClearColor(0., 0., 0., 1.);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        q.time("forward render".to_owned());
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

    framebuffer: GLuint,
}

impl<PIPELINE: PipelineState> Defered<PIPELINE> {
    pub fn new(input: PIPELINE) -> Defered<PIPELINE> {
        let textures: &mut [GLuint] = &mut [0, 0, 0, 0];
        let mut framebuffer: GLuint = 0;

        unsafe {
            gl::GenTextures(textures.len() as i32, textures.unsafe_mut_ref(0));
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

            framebuffer: framebuffer,
        };

        new.setup_framebuffer(1024, 1024);

        new
    }

    fn draw_target(&self) -> DrawTarget {
        DrawTarget {
            framebuffer: self.framebuffer,
            x: 0,
            y: 0,
            width: self.width,
            height: self.height,
            draw_buffers: ~[gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1,
                            gl::COLOR_ATTACHMENT2]
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
        set_texture(self.depth_buffer, gl::DEPTH_COMPONENT24);

        gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, self.uv_texture, 0);
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, self.normals_texture, 0);
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT2, self.material_texture, 0);
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
        let textures: &mut [GLuint] = &mut [self.uv_texture,
                                            self.normals_texture,
                                            self.material_texture,
                                            self.depth_buffer];

        unsafe {
            gl::DeleteTextures(textures.len() as i32, textures.unsafe_ref(0));
            gl::GenTextures(textures.len() as i32, textures.unsafe_mut_ref(0));
        }

        self.uv_texture = textures[0];
        self.normals_texture = textures[1];
        self.material_texture = textures[2];
        self.depth_buffer = textures[3];

        let (w, h) = (width as i32, height as i32);
        self.setup_framebuffer(w, h);
    }
}

impl<PIPELINE: PipelineState> Defered<PIPELINE> {
    fn ambient(&mut self, drawlist: &mut Drawlist, db: &GlState) {
        let plane = drawlist.find("core/geometry/plane")
                .expect("plane not found");
        let plane = drawlist.geometry(plane)
                .expect("Could not fetch geometry of plane");
        let shader = db.defered_shader_ambient
                .as_ref().expect("Could not load defered_shader");
        let vbo = db.vertex.find(&plane.vb)
                .expect("No vbo found");

        vbo.bind();
        shader.bind();

        assert!(0 == gl::GetError());
        vbo.bind();
        assert!(0 == gl::GetError());

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, self.uv_texture);
        gl::Uniform1i(shader.uniform("uv"), 0);
        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_2D, self.normals_texture);
        gl::Uniform1i(shader.uniform("normal"), 1);
        gl::ActiveTexture(gl::TEXTURE2);
        gl::BindTexture(gl::TEXTURE_2D, self.material_texture);
        gl::Uniform1i(shader.uniform("pixel_drawn_by"), 2);

        let textures = db.texture.textures();
        let atlas_uniform = shader.uniform("atlas");
        let atlas_base = shader.uniform("atlas_base");

        gl::BindBufferBase(gl::UNIFORM_BUFFER,
                           shader.uniform_block_index("Materials"),
                           drawlist.material_buffer());
    
        let total_textures = if textures.len() == 0 { 1 } else { textures.len() };
        for idx in range_step(0, total_textures, 12) {
            unsafe {
                if textures.len() != 0 {
                    let end = if idx + 12 > textures.len() {
                        textures.len()
                    } else {
                        idx + 12
                    };
                    for (e, i) in range(idx, end).enumerate() {
                        gl::ActiveTexture(gl::TEXTURE4+e as u32);
                        gl::BindTexture(gl::TEXTURE_2D_ARRAY, *textures.get(i));
                    }

                    let slice: Vec<i32> = range(idx, end)
                            .enumerate().map(|(e, _)| (e+4) as i32).collect();
                    gl::Uniform1i(atlas_base, idx as i32);
                    gl::Uniform1iv(atlas_uniform,
                                   slice.len() as i32,
                                   slice.get(0));
                }
                gl::DrawElements(gl::TRIANGLES,
                                 plane.count as i32,
                                 gl::UNSIGNED_INT,
                                 (plane.offset * 4) as *libc::c_void);
            }
        }
    }

    fn point_light(&mut self,
                   drawlist: &mut Drawlist,
                   db: &GlState,
                   dm: &DrawMatrices,
                   pos: &Vector4<f32>,
                   light: &PointLight,
                   dt: &DrawTarget) {

        let plane = drawlist.find("core/geometry/plane")
                .expect("plane not found");
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

        unsafe {
            gl::UniformMatrix4fv(shader.uniform("mat_proj"), 1, gl::FALSE, dm.projection.ptr());
            gl::UniformMatrix4fv(shader.uniform("mat_view"), 1, gl::FALSE, dm.view.ptr());
            gl::Uniform4fv(shader.uniform("light_position"), 1, pos.ptr());
            gl::Uniform3fv(shader.uniform("light_color"), 1, light.color().ptr());
            gl::Uniform1f(shader.uniform("light_intensity"), light.intensity());
            gl::Uniform4fv(shader.uniform("viewport"), 1, dt.to_vec4().ptr());
        }


        let textures = db.texture.textures();
        let atlas_uniform = shader.uniform("atlas");
        let atlas_base = shader.uniform("atlas_base");

        gl::BindBufferBase(gl::UNIFORM_BUFFER,
                           shader.uniform_block_index("Materials"),
                           drawlist.material_buffer());
    
        let total_textures = if textures.len() == 0 { 1 } else { textures.len() };
        for idx in range_step(0, total_textures, 12) {
            unsafe {
                if textures.len() != 0 {
                    let end = if idx + 12 > textures.len() {
                        textures.len()
                    } else {
                        idx + 12
                    };
                    for (e, i) in range(idx, end).enumerate() {
                        gl::ActiveTexture(gl::TEXTURE4+e as u32);
                        gl::BindTexture(gl::TEXTURE_2D_ARRAY, *textures.get(i));
                    }

                    let slice: Vec<i32> = range(idx, end)
                            .enumerate().map(|(e, _)| (e+4) as i32).collect();
                    gl::Uniform1i(atlas_base, idx as i32);
                    gl::Uniform1iv(atlas_uniform,
                                   slice.len() as i32,
                                   slice.get(0));
                }
                gl::DrawElements(gl::TRIANGLES,
                                 plane.count as i32,
                                 gl::UNSIGNED_INT,
                                 (plane.offset * 4) as *libc::c_void);
            }
        }
    }
}

impl<PIPELINE: PipelineState> PipelineState for Defered<PIPELINE> {
    fn render(&mut self, drawlist: &mut Drawlist, db: &GlState, dm: &DrawMatrices, ddt: &DrawTarget, q: &mut Profiler) {
        let dt = self.draw_target();
        self.input.render(drawlist, db, dm, &dt, q);

        gl::Disable(gl::DEPTH_TEST);
        gl::Enable(gl::DITHER);
        gl::Enable(gl::BLEND);
        gl::BlendEquation(gl::FUNC_ADD);
        gl::BlendFunc(gl::ONE, gl::ONE);

        q.time("defered: setup".to_owned());
        ddt.bind();
        gl::Clear(gl::COLOR_BUFFER_BIT);
        q.time("defered: ambient lighting".to_owned());
        self.ambient(drawlist, db);

        let lights: Vec<(ObjectKey, PointLight)> =
                drawlist.light_iter().map(|(&k, &v)| (k, v)).collect();
        for &(key, light) in lights.iter() {
            q.time("defered: point light".to_owned());
            let mat = drawlist.position(key);
            let pos = mat.mul_v(&Vector4::new(0f32, 0., 0., 1.));
            self.point_light(drawlist, db, dm, &pos, &light, &dt);
        }

        q.time("defered: cleanup".to_owned());
        for i in range(0, 16) {
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
        let dt = DrawTarget::new(0, (0, 0), (self.width, self.height), ~[gl::BACK_LEFT]);
        let dm = camera.get_matrices((self.width as i32, self.height as i32));
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
        display: Some(window.get_x11_display()),
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
    pub fn new(input: PIPELINE, window: Window) -> Hmd<PIPELINE> {
        let hmd = window.get_hmd();
        let desc = hmd.get_description();
        let caps = SensorCapabilities::new().set_orientation(true);
        assert!(hmd.start_sensor(caps, caps));
        let dist = DistortionCapabilities::new()
                .set_chromatic(false)
                .set_vignette(false)
                .set_timewarp(false);

        let rc = render_config(&window, &desc);
        let eye_desc = hmd.configure_rendering(
                &rc,
                dist,
                desc.eye_fovs.map(|_, eye| eye.default_eye_fov)
        ).expect("Could not create hmd context");

        let size = desc.eye_fovs.map(|which, eye| {
            hmd.get_fov_texture_size(which, eye.default_eye_fov, 1.0)
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
        DrawTarget {
            framebuffer: *self.framebuffers.eye(which),
            x: 0,
            y: 0,
            width: self.size.eye(which).x,
            height: self.size.eye(which).y,
            draw_buffers: ~[gl::COLOR_ATTACHMENT0]
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
        q.time("ovr: end_frame".to_owned());
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