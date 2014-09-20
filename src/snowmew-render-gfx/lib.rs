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

extern crate "snowmew-core" as snowmew;
extern crate "snowmew-position" as position;
extern crate "snowmew-graphics" as graphics;
extern crate "snowmew-render-data" as render_data;

use std::collections::hashmap::HashMap;

use opencl::hl;
use sync::Arc;

use position::Positions;
use graphics::Graphics;
use snowmew::common::ObjectKey;
use snowmew::io::Window;

use graphics::geometry::{Geo, GeoTex, GeoNorm, GeoTexNorm, GeoTexNormTan};
use graphics::geometry::{VertexGeoTex, VertexGeoTexNorm};
use graphics::Drawable;

use cow::join::{join_set_to_map, join_maps};
use render_data::RenderData;

use gfx::{Device, DeviceHelper};

static VERTEX_SRC: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core
    uniform mat4 proj_mat;
    uniform mat4 view_mat;
    uniform mat4 model_mat;

    in vec3 position;
    in vec2 texture;

    out vec2 o_texture;

    void main() {
        gl_Position = proj_mat * view_mat * model_mat * vec4(position, 1.0);
        o_texture = texture;
    }
"
};

static FRAGMENT_SRC: gfx::ShaderSource = shaders! {
GLSL_150: b"
    #version 150 core
    uniform vec4 ka_color;
    uniform int ka_use_texture;
    uniform sampler2D ka_texture;

    in vec2 o_texture;

    out vec4 o_Color;

    void main() {
        if (1 == ka_use_texture) {
            o_Color = texture(ka_texture, o_texture);
        } else {
            o_Color = ka_color;
        }
    }
"
};

#[shader_param(MyProgram)]
struct Params {
    proj_mat: [[f32, ..4], ..4],
    view_mat: [[f32, ..4], ..4],
    model_mat: [[f32, ..4], ..4],
    ka_color: [f32, ..4],
    ka_use_texture: i32,
    ka_texture: gfx::shade::TextureParam,
}

struct Mesh {
    mesh: render::mesh::Mesh,
    index: device::BufferHandle<u32>
}

pub struct RenderManager {
    data: Params,
    graphics: gfx::Graphics<device::gl_device::GlDevice,
                            device::gl_device::GlCommandBuffer>,
    frame: render::target::Frame,
    state: render::state::DrawState,
    prog: device::Handle<u32,device::shade::ProgramInfo>,
    meshes: HashMap<ObjectKey, Mesh>,
    textures: HashMap<ObjectKey, device::TextureHandle>,
    sampler: device::SamplerHandle,
    window: Window
}

impl RenderManager {

    fn _new(mut device: gfx::GlDevice,
            window: Window,
            size: (i32, i32),
            _: Option<Arc<hl::Device>>) -> RenderManager {

        let (width, height) = size;
        let frame = gfx::Frame::new(width as u16, height as u16);
        let state = gfx::DrawState::new().depth(gfx::state::LessEqual, true);

        let sampler = device.create_sampler(
            gfx::tex::SamplerInfo::new(
                    gfx::tex::Bilinear, gfx::tex::Tile
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

            let img_info = tinfo.to_image_info();
            let dummy_texture = device.create_texture(tinfo)
                                      .ok().expect("Failed to create texture");
            device.update_texture(&dummy_texture,
                                  &img_info,
                                  vec![0u8, 0, 0, 0].as_slice());

            let data = Params {
                proj_mat: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
                view_mat: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
                model_mat: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
                ka_use_texture: 0,
                ka_color: [1., 1., 1., 1.],
                ka_texture: (dummy_texture, Some(sampler))
            };
            (device.link_program(VERTEX_SRC.clone(), FRAGMENT_SRC.clone())
                  .ok().expect("Failed to link program"),
             data)
        };

        RenderManager {
            data: data,
            graphics: gfx::Graphics::new(device),
            frame: frame,
            state: state,
            prog: prog,
            meshes: HashMap::new(),
            textures: HashMap::new(),
            sampler: sampler,
            window: window
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
                self.graphics.device.update_texture(&texture, &img_info, text.data());
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

    fn draw<RD: RenderData>(&mut self, db: &RD, scene: ObjectKey, camera: ObjectKey) {
        let cdata = gfx::ClearData {
            color: [0.3, 0.3, 0.3, 1.0],
            depth: 1.0,
            stencil: 0,
        };
        let start = time::precise_time_s();
        self.graphics.clear(cdata, gfx::Color | gfx::Depth, &self.frame);

        let camera_trans = db.position(camera);
        let camera = snowmew::camera::Camera::new(camera_trans);

        let proj = camera.projection_matrix(16. / 9.);
        self.data.proj_mat =
            [[proj.x.x, proj.x.y, proj.x.z, proj.x.w],
             [proj.y.x, proj.y.y, proj.y.z, proj.y.w],
             [proj.z.x, proj.z.y, proj.z.z, proj.z.w],
             [proj.w.x, proj.w.y, proj.w.z, proj.w.w]];

        let view = camera.view_matrix();
        self.data.view_mat =
            [[view.x.x, view.x.y, view.x.z, view.x.w],
             [view.y.x, view.y.y, view.y.z, view.y.w],
             [view.z.x, view.z.y, view.z.z, view.z.w],
             [view.w.x, view.w.y, view.w.z, view.w.w]];

        let batches = self.create_geometry_batches(db, scene);

        for (id, (draw, _)) in join_set_to_map(db.scene_iter(scene),
                                               join_maps(db.drawable_iter(),
                                                         db.location_iter())) {

            let mat = db.material(draw.material).expect("Could not find material");
            let model = db.position(*id);

            self.data.model_mat =
                [[model.x.x, model.x.y, model.x.z, model.x.w],
                 [model.y.x, model.y.y, model.y.z, model.y.w],
                 [model.z.x, model.z.y, model.z.z, model.z.w],
                 [model.w.x, model.w.y, model.w.z, model.w.w]];


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
}


impl<RD: RenderData+Send> snowmew::Render<RD> for RenderManager {
    fn update(&mut self, db: RD, scene: ObjectKey, camera: ObjectKey) {
        self.load_meshes(&db);
        self.load_textures(&db);
        self.draw(&db, scene, camera);
    }
}

impl<RD: RenderData+Send> snowmew::RenderFactory<RD, RenderManager> for RenderFactory {
    fn init(self: Box<RenderFactory>,
            io: &snowmew::IOManager,
            window: Window,
            size: (i32, i32),
            cl: Option<Arc<hl::Device>>) -> RenderManager {

        window.make_context_current();

        let mut device = gfx::GlDevice::new(|s| io.get_proc_address(s));
        RenderManager::_new(device, window, size, cl)
    }
}

pub struct RenderFactory;

impl RenderFactory {
    pub fn new() -> RenderFactory { RenderFactory }
}