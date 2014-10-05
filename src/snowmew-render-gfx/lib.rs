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

#![crate_name = "snowmew-render-gfx"]
#![crate_type = "lib"]

#![feature(phase)]
#![allow(dead_code)]

//extern crate debug;

extern crate sync;
extern crate time;
extern crate libc;

extern crate opencl;
extern crate cow;
extern crate glfw;

#[phase(plugin)]
extern crate gfx_macros;
extern crate gfx;
extern crate device;
extern crate render;
extern crate genmesh;
extern crate cgmath;

extern crate "snowmew-core" as snowmew;
extern crate "snowmew-position" as position;
extern crate "snowmew-graphics" as graphics;
extern crate "snowmew-render-data" as render_data;

use std::collections::hashmap::HashMap;
use std::comm::{Receiver};

use opencl::hl;
use sync::Arc;

use position::Positions;
use graphics::Graphics;
use snowmew::common::ObjectKey;
use snowmew::io::Window;
use snowmew::camera::Camera;

use graphics::geometry::{Geo, GeoTex, GeoNorm, GeoTexNorm, GeoTexNormTan};
use graphics::geometry::{VertexGeoTex, VertexGeoTexNorm};

use cow::join::{join_set_to_map, join_maps};
use render_data::RenderData;

use gfx::{Device, DeviceHelper};

use cgmath::{Vector4, Vector, EuclideanVector, Matrix};
use cgmath::{FixedArray, Matrix4, Vector3, Point};

static VERTEX_SRC: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core
    uniform mat4 shadow_proj_mat;
    uniform mat4 shadow_view_mat;
    uniform mat4 shadow_bias_mat;
    uniform mat4 proj_mat;
    uniform mat4 view_mat;
    uniform mat4 model_mat;

    in vec3 position;
    in vec2 texture;
    in vec3 normal;

    out vec2 o_texture;
    out vec3 o_normal;
    out vec4 o_shadow_coord;

    void main() {
        gl_Position = proj_mat * view_mat * model_mat * vec4(position, 1.0);
        o_texture = texture;
        o_normal = normalize((model_mat * vec4(normal, 0.)).xyz);

        o_shadow_coord = shadow_bias_mat *
                         shadow_proj_mat *
                         shadow_view_mat *
                         model_mat *
                         vec4(position, 1.0);
    }
"
};

static FRAGMENT_SRC: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core
    uniform vec4 ka_color;
    uniform int ka_use_texture;
    uniform sampler2D ka_texture;

    uniform vec4 kd_color;
    uniform int kd_use_texture;
    uniform sampler2D kd_texture;

    uniform vec4 ks_color;
    uniform int ks_use_texture;
    uniform sampler2D ks_texture;

    uniform vec4 light_normal;
    uniform vec4 light_color;
    uniform sampler2DShadow shadow;

    in vec2 o_texture;
    in vec3 o_normal;
    in vec4 o_shadow_coord;

    out vec4 o_Color;

    void main() {
        vec4 normal = vec4(o_normal, 0.);
        vec3 shadow_coord = o_shadow_coord.xyz / o_shadow_coord.w;
        shadow_coord.z -= 0.001;
        vec4 color;
        vec4 ka, kd, ks;
        if (1 == ka_use_texture) {
            ka = texture(ka_texture, o_texture);
        } else {
            ka = ka_color;
        }
 
        if (1 == ks_use_texture) {
            ks = texture(ks_texture, o_texture);
        } else {
            ks = ka_color;
        }

        if (1 == kd_use_texture) {
            kd = texture(kd_texture, o_texture);
        } else {
            kd = ka_color;
        }

        float shadow_sum = 0;
        float x, y;

        shadow_sum += texture(shadow, shadow_coord) * 0.25;

        shadow_sum += textureOffset(shadow, shadow_coord, ivec2(1, 0)) * 0.125;
        shadow_sum += textureOffset(shadow, shadow_coord, ivec2(-1, 0)) * 0.125;
        shadow_sum += textureOffset(shadow, shadow_coord, ivec2(0, 1)) * 0.125;
        shadow_sum += textureOffset(shadow, shadow_coord, ivec2(0, -1)) * 0.125;

        shadow_sum += textureOffset(shadow, shadow_coord, ivec2(-1,-1)) * 0.0625;
        shadow_sum += textureOffset(shadow, shadow_coord, ivec2(-1, 1)) * 0.0625;
        shadow_sum += textureOffset(shadow, shadow_coord, ivec2( 1,-1)) * 0.0625;
        shadow_sum += textureOffset(shadow, shadow_coord, ivec2( 1, 1)) * 0.0625;

        color = ka * 0.2;
        color += shadow_sum *
                 kd *
                 light_color *
                 max(0, dot(light_normal, normal));
        o_Color = color;
    }
"
};

#[shader_param(MyProgram)]
struct Params {
    shadow_proj_mat: [[f32, ..4], ..4],
    shadow_view_mat: [[f32, ..4], ..4],
    shadow_bias_mat: [[f32, ..4], ..4],
    proj_mat: [[f32, ..4], ..4],
    view_mat: [[f32, ..4], ..4],
    model_mat: [[f32, ..4], ..4],

    ka_color: [f32, ..4],
    ka_use_texture: i32,
    ka_texture: gfx::shade::TextureParam,

    kd_color: [f32, ..4],
    kd_use_texture: i32,
    kd_texture: gfx::shade::TextureParam,

    ks_color: [f32, ..4],
    ks_use_texture: i32,
    ks_texture: gfx::shade::TextureParam,

    light_normal: [f32, ..4],
    light_color: [f32, ..4],
    shadow: gfx::shade::TextureParam
}

static SHADOW_VERTEX_SRC: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core
    uniform mat4 proj_mat;
    uniform mat4 view_mat;
    uniform mat4 model_mat;

    in vec3 position;

    void main() {
        gl_Position = proj_mat * view_mat * model_mat * vec4(position, 1.0);
    }
"
};

static SHADOW_FRAGMENT_SRC: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core

    void main() {}
"
};

#[shader_param(ShadowProgram)]
struct ShadowParams {
    proj_mat: [[f32, ..4], ..4],
    view_mat: [[f32, ..4], ..4],
    model_mat: [[f32, ..4], ..4],
}

struct Mesh {
    mesh: render::mesh::Mesh,
    index: device::BufferHandle<u32>
}

pub struct RenderManagerContext {
    prog: device::Handle<u32,device::shade::ProgramInfo>,
    data: Params,

    shadow_data: ShadowParams,
    shadow_prog: device::Handle<u32,device::shade::ProgramInfo>,
    shadow_frame: render::target::Frame,
    shadow: device::TextureHandle,
    shadow_sampler: device::SamplerHandle,

    graphics: gfx::Graphics<device::gl_device::GlDevice,
                            device::gl_device::GlCommandBuffer>,
    frame: render::target::Frame,
    state: render::state::DrawState,
    meshes: HashMap<ObjectKey, Mesh>,
    textures: HashMap<ObjectKey, device::TextureHandle>,
    sampler: device::SamplerHandle,
    window: Window,
}

pub struct RenderManager<R> {
    channel: Sender<(R, ObjectKey, ObjectKey)>
}

impl RenderManagerContext {

    fn _new(mut device: gfx::GlDevice,
            window: Window,
            size: (i32, i32),
            _: Option<Arc<hl::Device>>) -> RenderManagerContext {

        let (width, height) = size;
        let frame = gfx::Frame::new(width as u16, height as u16);
        let state = gfx::DrawState::new().depth(gfx::state::LessEqual, true);

        let sampler = device.create_sampler(
            gfx::tex::SamplerInfo::new(
                    gfx::tex::Anisotropic(16), gfx::tex::Tile
            )
        );

        let (prog, data) = {
            let tinfo = gfx::tex::TextureInfo {
                width: 1,
                height: 1,
                depth: 1,
                levels: 1,
                kind: gfx::tex::Texture2D,
                format: gfx::tex::RGBA8,
            };

            let dummy_texture = device.create_texture(tinfo)
                                      .ok().expect("Failed to create texture");

            let matrix: Matrix4<f32> = Matrix4::identity();
            let data = Params {
                proj_mat: matrix.into_fixed(),
                view_mat: matrix.into_fixed(),
                model_mat: matrix.into_fixed(),
                shadow_proj_mat: matrix.into_fixed(),
                shadow_view_mat: matrix.into_fixed(),
                shadow_bias_mat: [
                    [0.5, 0.0, 0.0, 0.0],
                    [0.0, 0.5, 0.0, 0.0],
                    [0.0, 0.0, 0.5, 0.0],
                    [0.5, 0.5, 0.5, 1.0],
                ],
                ka_use_texture: 0,
                ka_color: [1., 1., 1., 1.],
                ka_texture: (dummy_texture, Some(sampler)),

                kd_use_texture: 0,
                kd_color: [1., 1., 1., 1.],
                kd_texture: (dummy_texture, Some(sampler)),

                ks_use_texture: 0,
                ks_color: [1., 1., 1., 1.],
                ks_texture: (dummy_texture, Some(sampler)),

                light_color: [1., 1., 1., 1.],
                light_normal: [1., 0., 0., 0.],
                shadow: (dummy_texture, Some(sampler)),
            };
            (device.link_program(VERTEX_SRC.clone(), FRAGMENT_SRC.clone())
                  .ok().expect("Failed to link program"),
             data)
        };

        let (shadow_prog, shadow_data) = {
            let matrix: Matrix4<f32> = Matrix4::identity();
            let data = ShadowParams {
                proj_mat: matrix.into_fixed(),
                view_mat: matrix.into_fixed(),
                model_mat: matrix.into_fixed(),
            };
            (device.link_program(SHADOW_VERTEX_SRC.clone(),
                                 SHADOW_FRAGMENT_SRC.clone())
                  .ok().expect("Failed to link program"),
             data)
        };

        let shadow_info = gfx::tex::TextureInfo {
            width: 4096,
            height: 4096,
            depth: 1,
            levels: 1,
            kind: gfx::tex::Texture2D,
            format: gfx::tex::DEPTH24STENCIL8,
        };

        let mut shadow_sampler = gfx::tex::SamplerInfo::new(
            gfx::tex::Anisotropic(16), gfx::tex::Tile
        );
        shadow_sampler.comparison = gfx::tex::CompareRefToTexture(gfx::state::LessEqual);

        let shadow_sampler = device.create_sampler(shadow_sampler);

        let shadow = device.create_texture(shadow_info)
                           .ok().expect("Failed to create texture");

        let mut shadow_frame = gfx::Frame::new(
            shadow_info.width as u16,
            shadow_info.height as u16
        );

        shadow_frame.depth = Some(render::target::PlaneTexture(shadow, 0, None));


        RenderManagerContext {
            data: data,
            graphics: gfx::Graphics::new(device),
            frame: frame,
            state: state,
            prog: prog,
            meshes: HashMap::new(),
            textures: HashMap::new(),
            sampler: sampler,
            window: window,
            shadow_data: shadow_data,
            shadow_prog: shadow_prog,
            shadow: shadow,
            shadow_frame: shadow_frame,
            shadow_sampler: shadow_sampler
        }
    }

    fn load_meshes<RD: RenderData>(&mut self, db: &RD) {
        for (oid, vb) in db.vertex_buffer_iter() {
            if self.meshes.find(oid).is_none() {
                let mesh = match vb.vertex {
                    Geo(ref d) => {
                        println!("Geo");
                        let data: Vec<VertexGeoTex> = d.iter()
                            .map(|v| {
                                VertexGeoTex {
                                    position: v.position,
                                    texture: [0., 0.]
                                }
                            })
                            .collect();
                        self.graphics.device.create_mesh(data.as_slice())
                    },
                    GeoTex(ref d) => {
                        println!("GeoTex");
                        self.graphics.device.create_mesh(d.as_slice())
                    },
                    GeoNorm(ref d) => {
                        let data: Vec<VertexGeoTexNorm> = d.iter()
                            .map(|v| {
                                VertexGeoTexNorm {
                                    position: v.position,
                                    texture: [0., 0.],
                                    normal: v.normal
                                }
                            })
                            .collect();
                        self.graphics.device.create_mesh(data.as_slice())
                    },
                    GeoTexNorm(ref d) => {
                        println!("GeoTexNorm");
                        self.graphics.device.create_mesh(d.as_slice())
                    },
                    GeoTexNormTan(ref d) => {
                        println!("GeoTexNormTan");
                        self.graphics.device.create_mesh(d.as_slice())
                    }
                };

                let vb: Vec<u32> = vb.index.iter().map(|&x| x as u32).collect();

                let index = self.graphics.device.create_buffer_static(vb.as_slice());

                self.meshes.insert(*oid, Mesh {
                    index: index,
                    mesh: mesh
                });
            }
        }
    }

    fn load_textures<RD: RenderData>(&mut self, db: &RD) {
        for (oid, text) in db.texture_iter() {
            if self.textures.find(oid).is_none() {
                let tinfo = gfx::tex::TextureInfo {
                    width: text.width() as u16,
                    height: text.height() as u16,
                    depth: 1 as u16,
                    levels: 1,
                    kind: gfx::tex::Texture2D,
                    format: match text.depth() {
                        4 => gfx::tex::Unsigned(gfx::tex::RGBA, 8, gfx::attrib::IntNormalized),
                        3 => gfx::tex::Unsigned(gfx::tex::RGB, 8, gfx::attrib::IntNormalized),
                        _ => fail!("Unsupported color depth")
                    }
                };

                let img_info = tinfo.to_image_info();
                let texture = self.graphics.device.create_texture(tinfo)
                                         .ok().expect("Failed to create texture");
                self.graphics.device.update_texture(&texture, &img_info, text.data()).unwrap();
                self.textures.insert(*oid, texture);
            }
        }
    }

    fn create_geometry_batches<RD: RenderData>(&mut self, db: &RD, scene: ObjectKey) -> HashMap<u32, MyProgram> {
        let mut batches: HashMap<u32, MyProgram> = HashMap::new();

        for (_, draw) in join_set_to_map(db.scene_iter(scene), db.drawable_iter()) {
            if batches.contains_key(&draw.geometry) {
                continue;
            }

            let geo = db.geometry(draw.geometry).expect("failed to find geometry");
            let vb = self.meshes.find(&geo.vb).expect("Could not get vertex buffer");

            let batch: MyProgram = self.graphics.make_batch(
                &self.prog,
                &vb.mesh,
                gfx::IndexSlice32(gfx::TriangleList,
                                  vb.index,
                                  geo.offset as u32,
                                  geo.count as u32),
                &self.state
            ).unwrap();

            batches.insert(draw.geometry, batch);
        }

        return batches;
    }


    fn create_shadow_batches<RD: RenderData>(&mut self, db: &RD, scene: ObjectKey) -> HashMap<u32, ShadowProgram> {
        let mut batches: HashMap<u32, ShadowProgram> = HashMap::new();

        for (_, draw) in join_set_to_map(db.scene_iter(scene), db.drawable_iter()) {
            if batches.contains_key(&draw.geometry) {
                continue;
            }

            let geo = db.geometry(draw.geometry).expect("failed to find geometry");
            let vb = self.meshes.find(&geo.vb).expect("Could not get vertex buffer");

            let batch: ShadowProgram = self.graphics.make_batch(
                &self.shadow_prog,
                &vb.mesh,
                gfx::IndexSlice32(gfx::TriangleList,
                                  vb.index,
                                  geo.offset as u32,
                                  geo.count as u32),
                &self.state
            ).unwrap();

            batches.insert(draw.geometry, batch);
        }

        return batches;
    }


    fn draw_shadow<RD: RenderData>(&mut self, db: &RD, scene: ObjectKey, cam: &Camera) {
        let batches = self.create_shadow_batches(db, scene);

        let cdata = gfx::ClearData {
            color: [0.3, 0.3, 0.3, 1.0],
            depth: 2.0,
            stencil: 0,
        };
        self.graphics.clear(cdata, gfx::Depth, &self.shadow_frame);

        let pos = cam.move_with_vector(&Vector3::new(0f32, 15., 0.));
        let proj = cgmath::ortho(
            -25.,  // + pos.x,
             25.,  // + pos.x,
            -25.,  // + pos.y,
             25.,  // + pos.y,
            -500., // + pos.z,
             500., // + pos.z
        );
        self.shadow_data.proj_mat = proj.into_fixed();
        self.data.shadow_proj_mat = proj.into_fixed();

        let dir = self.data.light_normal;

        let view: Matrix4<f32> = cgmath::Matrix4::look_at(
            &pos.add_v(&Vector3::new(dir[0], dir[1], dir[2])),
            &pos,
            &Vector3::new(0f32, 1., 0.)
        );

        self.shadow_data.view_mat = view.into_fixed();
        self.data.shadow_view_mat = view.into_fixed();

        for (id, (draw, _)) in join_set_to_map(db.scene_iter(scene),
                                               join_maps(db.drawable_iter(),
                                                         db.location_iter())) {

            let mat = db.material(draw.material).expect("Could not find material");
            let model = db.position(*id);

            self.shadow_data.model_mat = model.into_fixed();
            self.graphics.draw(
                batches.find(&draw.geometry).expect("Missing draw"),
                &self.shadow_data,
                &self.shadow_frame
            );
        }

        self.graphics.device.generate_mipmap(&self.shadow);

    }

    fn draw<RD: RenderData>(&mut self, db: &RD, scene: ObjectKey, camera: ObjectKey) {
        let cdata = gfx::ClearData {
            color: [0.3, 0.3, 0.3, 1.0],
            depth: 1.0,
            stencil: 0,
        };
        let start = time::precise_time_s();
        self.graphics.clear(cdata, gfx::Color | gfx::Depth, &self.frame);

        let camera_trans = db.position(camera);
        let camera = Camera::new(camera_trans);

        let proj = camera.projection_matrix(16. / 9.);
        let view = camera.view_matrix();

        self.data.view_mat = view.into_fixed();
        self.data.proj_mat = proj.into_fixed();

        let batches = self.create_geometry_batches(db, scene);

        for (key, light) in db.light_iter() {
            match light {
                &graphics::PointLight(p) => {}
                &graphics::DirectionalLight(d) => {
                    let n = d.normal();
                    let n = Vector4::new(n.x, n.y, n.z, 0.);
                    let n = db.position(*key).mul_v(&n).normalize();
                    let color = d.color().mul_s(d.intensity());
                    self.data.light_color = [color.x, color.y, color.z, 1.];
                    self.data.light_normal = [n.x, n.y, n.z, n.w];
                }
            }
        }

        self.draw_shadow(db, scene, &camera);

        for (id, (draw, _)) in join_set_to_map(db.scene_iter(scene),
                                               join_maps(db.drawable_iter(),
                                                         db.location_iter())) {

            let mat = db.material(draw.material).expect("Could not find material");
            let model = db.position(*id);

            self.data.model_mat = model.into_fixed();

            self.data.ka_use_texture = match mat.map_ka() {
                Some(tid) => {
                    let &texture = self.textures.find(&tid).expect("Texture not loaded");
                    self.data.ka_texture = (texture, Some(self.sampler));
                    1
                }
                None => {
                    let [r, g, b] = mat.ka();
                    self.data.ka_color = [r, g, b, 1.];
                    0
                }
            };

            self.data.kd_use_texture = match mat.map_kd() {
                Some(tid) => {
                    let &texture = self.textures.find(&tid).expect("Texture not loaded");
                    self.data.kd_texture = (texture, Some(self.sampler));
                    1
                }
                None => {
                    let [r, g, b] = mat.kd();
                    self.data.kd_color = [r, g, b, 1.];
                    0
                }
            };

            self.data.ks_use_texture = match mat.map_ks() {
                Some(tid) => {
                    let &texture = self.textures.find(&tid).expect("Texture not loaded");
                    self.data.ks_texture = (texture, Some(self.sampler));
                    1
                }
                None => {
                    let [r, g, b] = mat.ks();
                    self.data.ks_color = [r, g, b, 1.];
                    0
                }
            };

            self.data.shadow = (self.shadow, Some(self.shadow_sampler));

            self.graphics.draw(
                batches.find(&draw.geometry).expect("Missing draw"),
                &self.data,
                &self.frame
            );
        }

        self.graphics.end_frame();
        self.window.swap_buffers();
        let end = time::precise_time_s();
        println!("{0:4.3}ms", (end - start) * 1000.);
    }

    fn update<RD: RenderData>(&mut self, db: RD, scene: ObjectKey, camera: ObjectKey) {
        self.load_meshes(&db);
        self.load_textures(&db);
        self.draw(&db, scene, camera);
    }
}

impl<RD: RenderData+Send> snowmew::Render<RD> for RenderManager<RD> {
    fn update(&mut self, db: RD, scene: ObjectKey, camera: ObjectKey) {
        self.channel.send((db, scene, camera));
    }
}

impl<RD: RenderData+Send> snowmew::RenderFactory<RD, RenderManager<RD>> for RenderFactory {
    fn init(self: Box<RenderFactory>,
            io: &snowmew::IOManager,
            window: Window,
            size: (i32, i32),
            cl: Option<Arc<hl::Device>>) -> RenderManager<RD> {

        let (sender, recv) = channel();

        window.make_context_current();
        let device = gfx::GlDevice::new(|s| io.get_proc_address(s));
        glfw::make_context_current(None);

        spawn(proc() {
            let recv: Receiver<(RD, ObjectKey, ObjectKey)> = recv;
            window.make_context_current();

            let mut rc = RenderManagerContext::_new(device, window, size, cl);
            loop {
                // wait for a copy of the game
                let (mut db, mut scene, mut camera) = recv.recv();

                loop {
                    match recv.try_recv() {
                        Ok((_db, _scene, _camera)) => {
                            db = _db;
                            scene = _scene;
                            camera = _camera
                        }
                        // no newer copy
                        Err(std::comm::Empty) => break,
                        Err(std::comm::Disconnected) => return,
                    }
                }
                rc.update(db, scene, camera);
            }
            
        });

        RenderManager { channel: sender }

    }
}

pub struct RenderFactory;

impl RenderFactory {
    pub fn new() -> RenderFactory { RenderFactory }
}