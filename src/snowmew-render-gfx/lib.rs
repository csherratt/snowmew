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
#![feature(plugin)]
#![allow(unstable)]
#![allow(dead_code)]


extern crate libc;
extern crate opencl;
extern crate glfw;

#[plugin]
#[macro_use]
extern crate gfx_macros;
extern crate gfx;
extern crate device;
extern crate render;
extern crate genmesh;
extern crate cgmath;

extern crate "snowmew-core" as snowmew;
extern crate "snowmew-position" as position;
extern crate "snowmew-graphics" as graphics;
extern crate "snowmew-render" as sm_render;
extern crate "snowmew-input" as input;
extern crate collect;

use std::collections::{HashMap, BTreeSet};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::Thread;
use std::sync::Arc;

use opencl::hl;
use gfx::{Device, DeviceHelper};
use cgmath::{Vector4, Vector, EuclideanVector, Matrix};
use cgmath::{FixedArray, Matrix4, Vector3, Point};
use collect::iter::{OrderedMapIterator, OrderedSetIterator};

use position::Positions;
use graphics::Graphics;
use snowmew::common::Entity;
use snowmew::camera::Camera;
use graphics::Material;
use graphics::geometry::{VertexGeoTex, VertexGeoTexNorm};
use graphics::geometry::Vertex::{Geo, GeoTex, GeoNorm, GeoTexNorm, GeoTexNormTan};
use sm_render::Renderable;
use input::{Window, GetIoState};

#[derive(Copy)]
struct SharedMatrix {
    proj_mat: [[f32; 4]; 4],
    view_mat: [[f32; 4]; 4]
}

#[derive(Copy)]
struct SharedMaterial {
    ka_color: [f32; 4],
    kd_color: [f32; 4],
    ks_color: [f32; 4],

    ka_use_texture: i32,
    kd_use_texture: i32,
    ks_use_texture: i32,
}

const VERTEX_SRC: gfx::ShaderSource<'static> = shaders! {
glsl_150: b"
    #version 150 core
    layout(std140)
    uniform shadow_shared_mat {
        mat4 shadow_proj_mat;
        mat4 shadow_view_mat;
    };

    layout(std140)
    uniform shared_mat {
        mat4 proj_mat;
        mat4 view_mat;
    };

    uniform model {
        mat4 model_mat[512];
    };
    uniform int offset;

    uniform mat4 shadow_bias_mat;

    in vec3 position;
    in vec2 texture;
    in vec3 normal;

    out vec2 o_texture;
    out vec3 o_normal;
    out vec4 o_shadow_coord;

    void main() {
        gl_Position =
            proj_mat *
            view_mat *
            model_mat[gl_InstanceID + offset] *
            vec4(position, 1.0);
        o_texture = texture;
        o_normal = normalize((model_mat[gl_InstanceID + offset] * vec4(normal, 0.)).xyz);

        o_shadow_coord = shadow_bias_mat *
                         shadow_proj_mat *
                         shadow_view_mat *
                         model_mat[gl_InstanceID + offset] *
                         vec4(position, 1.0);
    }
"};

const FRAGMENT_SRC: gfx::ShaderSource<'static> = shaders! {
glsl_150: b"
    #version 150 core
    layout(std140)
    uniform material {
        vec4 ka_color;
        vec4 kd_color;
        vec4 ks_color;

        int ka_use_texture;
        int kd_use_texture;
        int ks_use_texture;
    };

    uniform sampler2D ka_texture;
    uniform sampler2D kd_texture;
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
        shadow_coord.z -= 0.0002;
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
"};

#[shader_param(DrawProgram)]
struct Params {
    shadow_shared_mat: gfx::RawBufferHandle,
    shared_mat: gfx::RawBufferHandle,

    shadow_bias_mat: [[f32; 4]; 4],

    material: gfx::RawBufferHandle,
    ka_texture: gfx::shade::TextureParam,
    kd_texture: gfx::shade::TextureParam,
    ks_texture: gfx::shade::TextureParam,

    light_normal: [f32; 4],
    light_color: [f32; 4],
    shadow: gfx::shade::TextureParam,

    model: gfx::RawBufferHandle,
    offset: i32
}

static SHADOW_VERTEX_SRC: gfx::ShaderSource<'static> = shaders! {
glsl_150: b"
    #version 150 core
    layout(std140)
    uniform shared_mat {
        mat4 proj_mat;
        mat4 view_mat;
    };
    uniform model {
        mat4 model_mat[512];
    };
    uniform int offset;

    in vec3 position;

    void main() {
        gl_Position =
            proj_mat *
            view_mat *
            model_mat[gl_InstanceID+offset] *
            vec4(position, 1.0);
    }
"};

static SHADOW_FRAGMENT_SRC: gfx::ShaderSource<'static> = shaders! {
glsl_150: b"
    #version 150 core

    void main() {}
"};

#[shader_param(ShadowProgram)]
struct ShadowParams {
    shared_mat: gfx::RawBufferHandle,
    model: gfx::RawBufferHandle,
    offset: i32
}

struct Mesh {
    mesh: render::mesh::Mesh,
    index: device::BufferHandle<u32>
}

struct RenderMaterial {
    material: Material,
    buffer: device::BufferHandle<SharedMaterial>,
    ka_texture: Option<Entity>,
    kd_texture: Option<Entity>,
    ks_texture: Option<Entity>,
}

pub struct RenderManagerContext {
    prog: device::Handle<u32,device::shade::ProgramInfo>,
    data: Params,

    shadow_data: ShadowParams,
    shadow_prog: device::Handle<u32,device::shade::ProgramInfo>,
    shadow_frame: render::target::Frame,
    shadow: device::TextureHandle,
    shadow_sampler: device::SamplerHandle,
    shadow_shared_mat: device::BufferHandle<SharedMatrix>,
    shared_mat: device::BufferHandle<SharedMatrix>,

    graphics: gfx::Graphics<device::gl_device::GlDevice>,
    frame: render::target::Frame,
    state: render::state::DrawState,
    meshes: HashMap<Entity, Mesh>,
    textures: HashMap<Entity, device::TextureHandle>,
    sampler: device::SamplerHandle,
    window: Window,

    material: HashMap<Entity, RenderMaterial>,

    batch: BTreeSet<(Entity, Entity, Entity)>,
    shadow_batches: HashMap<Entity, ShadowProgram>,
    draw_batches: HashMap<Entity, DrawProgram>,

    spare_matrix_buffers: Vec<device::BufferHandle<[[f32; 4]; 4]>>,
    used_matrix_buffers: Vec<device::BufferHandle<[[f32; 4]; 4]>>,
    shared_geometry: Vec<(u32, device::BufferHandle<[[f32; 4]; 4]>, usize, usize)>,
    shared_geometry_material: Vec<(u32, u32, device::BufferHandle<[[f32; 4]; 4]>, usize, usize)>,
}

pub struct RenderManager<R> {
    channel: Sender<R>,
    res: std::thread::Thread
}

impl RenderManagerContext {
    fn _new(mut device: gfx::GlDevice,
            window: Window,
            size: (i32, i32),
            _: Option<Arc<hl::Device>>) -> RenderManagerContext {

        let (width, height) = size;
        let frame = gfx::Frame::new(width as u16, height as u16);
        let state = gfx::DrawState::new().depth(gfx::state::Comparison::LessEqual, true);

        let sampler = device.create_sampler(
            gfx::tex::SamplerInfo::new(
                gfx::tex::FilterMethod::Anisotropic(16), gfx::tex::WrapMode::Tile
            )
        );

        let (shadow_prog, shadow_data, shadow_shared_mat) = {
            let buff = device.create_buffer::<SharedMatrix>(1, gfx::BufferUsage::Static);
            let data = ShadowParams {
                shared_mat: buff.raw(),
                model: buff.raw(),
                offset: 0
            };
            (device.link_program(SHADOW_VERTEX_SRC.clone(),
                                 SHADOW_FRAGMENT_SRC.clone())
                  .ok().expect("Failed to link program"),
             data, buff)
        };

        let (prog, data, shared_mat) = {
            let tinfo = gfx::tex::TextureInfo {
                width: 1,
                height: 1,
                depth: 1,
                levels: 1,
                kind: gfx::tex::TextureKind::Texture2D,
                format: gfx::tex::RGBA8,
            };

            let dummy_texture = device.create_texture(tinfo)
                                      .ok().expect("Failed to create texture");

            let buff = device.create_buffer::<SharedMatrix>(1, gfx::BufferUsage::Static);
            let unused = device.create_buffer::<Material>(1, gfx::BufferUsage::Static);
            let data = Params {
                shared_mat: buff.raw(),
                shadow_shared_mat: shadow_shared_mat.raw(),
                shadow_bias_mat: [
                    [0.5, 0.0, 0.0, 0.0],
                    [0.0, 0.5, 0.0, 0.0],
                    [0.0, 0.0, 0.5, 0.0],
                    [0.5, 0.5, 0.5, 1.0],
                ],
                material: unused.raw(),
                ka_texture: (dummy_texture, Some(sampler)),
                kd_texture: (dummy_texture, Some(sampler)),
                ks_texture: (dummy_texture, Some(sampler)),

                light_color: [1., 1., 1., 1.],
                light_normal: [1., 0., 0., 0.],
                shadow: (dummy_texture, Some(sampler)),
                model: buff.raw(),
                offset: 0
            };
            (device.link_program(VERTEX_SRC.clone(), FRAGMENT_SRC.clone())
                  .ok().expect("Failed to link program"),
             data, buff)
        };

        let shadow_info = gfx::tex::TextureInfo {
            width: 2048,
            height: 2048,
            depth: 1,
            levels: 1,
            kind: gfx::tex::TextureKind::Texture2D,
            format: gfx::tex::Format::DEPTH24STENCIL8,
        };

        let mut shadow_sampler = gfx::tex::SamplerInfo::new(
            gfx::tex::FilterMethod::Anisotropic(16), gfx::tex::WrapMode::Tile
        );
        shadow_sampler.comparison = gfx::tex::ComparisonMode::CompareRefToTexture(gfx::state::Comparison::LessEqual);

        let shadow_sampler = device.create_sampler(shadow_sampler);

        let shadow = device.create_texture(shadow_info)
                           .ok().expect("Failed to create texture");

        let mut shadow_frame = gfx::Frame::new(
            shadow_info.width as u16,
            shadow_info.height as u16
        );

        shadow_frame.depth = Some(render::target::Plane::Texture(shadow, 0, None));


        RenderManagerContext {
            data: data,
            graphics: gfx::Graphics::new(device),
            frame: frame,
            state: state,
            prog: prog,
            meshes: HashMap::new(),
            textures: HashMap::new(),
            material: HashMap::new(),
            sampler: sampler,
            window: window,
            shadow_data: shadow_data,
            shadow_prog: shadow_prog,
            shadow_shared_mat: shadow_shared_mat,
            shared_mat: shared_mat,
            shadow: shadow,
            shadow_frame: shadow_frame,
            shadow_sampler: shadow_sampler,
            batch: BTreeSet::new(),
            shadow_batches: HashMap::new(),
            draw_batches: HashMap::new(),
            spare_matrix_buffers: Vec::new(),
            used_matrix_buffers: Vec::new(),
            shared_geometry: Vec::new(),
            shared_geometry_material: Vec::new(),
        }
    }

    fn load_meshes<RD: Renderable+GetIoState>(&mut self, db: &RD) {
        for (oid, vb) in db.vertex_buffer_iter() {
            if self.meshes.get(&oid).is_none() {
                let mesh = match vb.vertex {
                    Geo(ref d) => {
                        let data: Vec<VertexGeoTex> = d.iter()
                            .map(|v| {
                                VertexGeoTex {
                                    position: v.position,
                                    texture: [0., 0.]
                                }
                            })
                            .collect();
                        self.graphics.device.create_mesh(&data[])
                    },
                    GeoTex(ref d) => {
                        self.graphics.device.create_mesh(&d[])
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
                        self.graphics.device.create_mesh(&data[])
                    },
                    GeoTexNorm(ref d) => {
                        self.graphics.device.create_mesh(&d[])
                    },
                    GeoTexNormTan(ref d) => {
                        self.graphics.device.create_mesh(&d[])
                    }
                };

                let vb: Vec<u32> = vb.index.iter().map(|&x| x as u32).collect();

                let index = self.graphics.device.create_buffer_static(&vb[]);

                self.meshes.insert(oid, Mesh {
                    index: index,
                    mesh: mesh
                });
            }
        }
    }

    fn load_textures<RD: Renderable+GetIoState>(&mut self, db: &RD) {
        for (oid, text) in db.texture_iter() {
            if self.textures.get(&oid).is_none() {
                let tinfo = gfx::tex::TextureInfo {
                    width: text.width() as u16,
                    height: text.height() as u16,
                    depth: 1 as u16,
                    levels: 1,
                    kind: gfx::tex::TextureKind::Texture2D,
                    format: match text.depth() {
                        4 => gfx::tex::Format::Unsigned(gfx::tex::Components::RGBA, 8, gfx::attrib::IntSubType::Normalized),
                        3 => gfx::tex::Format::Unsigned(gfx::tex::Components::RGB, 8, gfx::attrib::IntSubType::Normalized),
                        _ => panic!("Unsupported color depth")
                    }
                };

                let img_info = tinfo.to_image_info();
                let texture = self.graphics.device.create_texture(tinfo)
                                         .ok().expect("Failed to create texture");
                self.graphics.device.update_texture(&texture, &img_info, text.data())
                    .ok().expect("Failed to update texture.");
                self.textures.insert(oid, texture);
            }
        }
    }

    fn load_materials<RD: Renderable+GetIoState>(&mut self, db: &RD) {
        for (oid, &mat) in db.material_iter() {
            let update = if let Some(material) = self.material.get(&oid) {
                Some(mat != material.material)
            } else {None};

            if update == Some(true) {
                self.material.remove(&oid);
            } else if update == Some(false) {
                continue;
            }

            let ka = mat.ka();
            let kd = mat.kd();
            let ks = mat.ks();
            let material = &[SharedMaterial {
                ka_color: [ka[0], ka[1], ka[2], 1.],
                kd_color: [kd[0], kd[1], kd[2], 1.],
                ks_color: [ks[0], ks[1], ks[2], 1.],
                ka_use_texture: if mat.map_ka().is_some() {1} else {0},
                kd_use_texture: if mat.map_kd().is_some() {1} else {0},
                ks_use_texture: if mat.map_ks().is_some() {1} else {0},
            }];
            let buff = self.graphics.device.create_buffer_static(material);
            self.material.insert(oid, RenderMaterial {
                material: mat,
                buffer: buff,
                ka_texture: mat.map_ka(),
                ks_texture: mat.map_ks(),
                kd_texture: mat.map_kd(),
            }); 
        }       
    }

    fn load_batches<RD: Renderable+GetIoState>(&mut self, db: &RD) {
        let scene = db.scene().expect("no scene set");
        self.batch.clear();
        self.shadow_batches.clear();
        self.draw_batches.clear();

        for (id, draw) in db.scene_iter(scene).inner_join_map(db.drawable_iter()) {
            self.batch.insert((draw.geometry, draw.material, id));

            if !self.shadow_batches.contains_key(&draw.geometry) {
                let geo = db.geometry(draw.geometry).expect("failed to find geometry");
                let vb = self.meshes.get(&geo.vb).expect("Could not get vertex buffer");

                let batch: ShadowProgram = self.graphics.make_batch(
                    &self.shadow_prog,
                    &vb.mesh,
                    gfx::Slice {
                        start: geo.offset as u32,
                        end: (geo.offset + geo.count) as u32,
                        prim_type: gfx::PrimitiveType::TriangleList,
                        kind: gfx::SliceKind::Index32(vb.index, 0)

                    },
                    &self.state
                ).ok().expect("Failed to create batch.");

                self.shadow_batches.insert(draw.geometry, batch);

                let batch: DrawProgram = self.graphics.make_batch(
                    &self.prog,
                    &vb.mesh,
                    gfx::Slice {
                        start: geo.offset as u32,
                        end: (geo.offset + geo.count) as u32,
                        prim_type: gfx::PrimitiveType::TriangleList,
                        kind: gfx::SliceKind::Index32(vb.index, 0)

                    },
                    &self.state
                ).ok().expect("Failed to create batch.");
                self.draw_batches.insert(draw.geometry, batch);
            }
        }
    }

    fn fetch_matrix(&mut self) -> device::BufferHandle<[[f32; 4]; 4]> {
        let buffer = if let Some(buffer) = self.spare_matrix_buffers.pop() {
            buffer
        } else {
            self.graphics.device.create_buffer(512, gfx::BufferUsage::Static)
        };
        self.used_matrix_buffers.push(buffer);
        buffer
    }

    fn load_matrices<RD: Renderable+GetIoState>(&mut self, db: &RD) {
        let max = 512;
        let mut matrices = Vec::new();
        matrices.reserve(512);

        let mut shared_g = Vec::new();
        let mut shared_gm = Vec::new();

        for m in self.used_matrix_buffers.drain() {
            self.spare_matrix_buffers.push(m);
        }

        let mut mat = self.fetch_matrix();

        let mut last = None;
        for &(g, m, id) in self.batch.clone().iter() {
            last = if let Some((lg, lm, mut idx_gm, mut idx_g)) = last {
                if (lg, lm) != (g, m) {
                    shared_gm.push((lg, lm, mat, matrices.len()-idx_gm, idx_gm));
                    idx_gm = matrices.len();
                }
                if lg != g {
                    shared_g.push((lg, mat, matrices.len()-idx_g, idx_g));
                    idx_g = matrices.len();
                }
            
                if matrices.len() == max {
                    shared_gm.push((lg, lm, mat, matrices.len()-idx_gm, idx_gm));
                    shared_g.push((lg, mat, matrices.len()-idx_g, idx_g));
                    self.graphics.device.update_buffer(mat, &matrices[], 0);
                    mat = self.fetch_matrix();
                    matrices.clear();
                    None
                } else {
                    Some((g, m, idx_gm, idx_g))
                }
            } else {
                Some((g, m, 0, 0))
            };

            matrices.push(db.position(id).into_fixed());
        }

        if let Some((g, m, idx_gm, idx_g)) = last {
            shared_gm.push((g, m, mat, matrices.len()-idx_gm, idx_gm));
            shared_g.push((g, mat, matrices.len()-idx_g, idx_g));
            self.graphics.device.update_buffer(mat, &matrices[], 0)
        }

        self.shared_geometry = shared_g;
        self.shared_geometry_material = shared_gm;
    }

    fn draw_shadow(&mut self, cam: &Camera) {
        let cdata = gfx::ClearData {
            color: [0.3, 0.3, 0.3, 1.0],
            depth: 2.0,
            stencil: 0,
        };
        self.graphics.clear(cdata, gfx::DEPTH, &self.shadow_frame);

        let pos = cam.move_with_vector(&Vector3::new(0f32, 15., 0.));
        let proj = cgmath::ortho(
            -25.,  // + pos.x,
             25.,  // + pos.x,
            -25.,  // + pos.y,
             25.,  // + pos.y,
            -500., // + pos.z,
             500., // + pos.z
        );

        let dir = self.data.light_normal;

        let view: Matrix4<f32> = cgmath::Matrix4::look_at(
            &pos.add_v(&Vector3::new(dir[0], dir[1], dir[2])),
            &pos,
            &Vector3::new(0f32, 1., 0.)
        );

        let shadow_mat = &[SharedMatrix {
            proj_mat: proj.into_fixed(),
            view_mat: view.into_fixed()
        }];

        self.graphics.device.update_buffer(self.shadow_shared_mat, shadow_mat, 0);

        for &(geo, matrix, len, offset) in self.shared_geometry.iter() {
            self.shadow_data.model = matrix.raw();
            self.shadow_data.offset = offset as i32;
            self.graphics.draw_instanced(
                self.shadow_batches.get(&geo).expect("Missing draw"),
                &self.shadow_data,
                len as u32,
                0,
                &self.shadow_frame,
            );
        };

    }

    fn draw<RD: Renderable+GetIoState>(&mut self, db: &RD) {
        let camera = db.camera().expect("no camera set");

        let cdata = gfx::ClearData {
            color: [0.3, 0.3, 0.3, 1.0],
            depth: 1.0,
            stencil: 0,
        };
        self.graphics.clear(cdata, gfx::COLOR | gfx::DEPTH, &self.frame);

        let (width, height) = db.get_io_state().size;
        let camera_trans = db.position(camera);
        let camera = Camera::new(width, height, camera_trans);

        let proj = camera.projection_matrix();
        let view = camera.view_matrix();

        let shared_mat = &[SharedMatrix {
            view_mat: view.into_fixed(),
            proj_mat: proj.into_fixed()
        }];

        self.graphics.device.update_buffer(self.shared_mat, shared_mat, 0);

        for (key, light) in db.light_iter() {
            match light {
                &graphics::Light::Point(_) => {}
                &graphics::Light::Directional(d) => {
                    let n = d.normal();
                    let n = Vector4::new(n.x, n.y, n.z, 0.);
                    let n = db.position(key).mul_v(&n).normalize();
                    let color = d.color().mul_s(d.intensity());
                    self.data.light_color = [color.x, color.y, color.z, 1.];
                    self.data.light_normal = [n.x, n.y, n.z, n.w];
                }
            }
        }

        self.draw_shadow(&camera);

        for &(geo, mat, matrix, len, offset) in self.shared_geometry_material.iter() {
            let mat = self.material.get(&mat).expect("Could not find material");
            if let Some(ka) = mat.ka_texture {
                self.data.ka_texture =
                    (*self.textures.get(&ka)
                          .expect("Could not find texture"),
                     Some(self.sampler));
            }
            if let Some(kd) = mat.kd_texture {
                self.data.kd_texture =
                    (*self.textures.get(&kd)
                          .expect("Could not find texture"),
                     Some(self.sampler));
            }
            if let Some(ks) = mat.ks_texture {
                self.data.ks_texture =
                    (*self.textures.get(&ks)
                          .expect("Could not find texture"),
                     Some(self.sampler));
            }
            self.data.material = mat.buffer.raw();
            self.data.shadow = (self.shadow, Some(self.shadow_sampler));
            self.data.model = matrix.raw();
            self.data.offset = offset as i32;

            self.graphics.draw_instanced(
                self.draw_batches.get(&geo).expect("Missing draw"),
                &self.data,
                len as u32,
                0,
                &self.frame,
            );
        };

        self.graphics.end_frame();
        self.window.swap_buffers();
    }

    fn config<RD: Renderable+GetIoState>(&mut self, db: &RD) {
        let (width, height) = db.get_io_state().size;
        if self.frame.width as u32 != width ||
           self.frame.height as u32 != height {
            self.frame = gfx::Frame::new(width as u16, height as u16);
        }
    }

    fn update<RD: Renderable+GetIoState>(&mut self, db: RD) {
        self.config(&db);
        self.load_meshes(&db);
        self.load_textures(&db);
        self.load_materials(&db);
        self.load_batches(&db);
        self.load_matrices(&db);
        self.draw(&db);
    }
}

impl<RD: Renderable+GetIoState+Send> sm_render::Render<RD> for RenderManager<RD> {
    fn update(&mut self, db: RD) {
        self.channel.send(db);
    }
}

impl<RD: Renderable+GetIoState+Send> sm_render::RenderFactory<RD, RenderManager<RD>> for RenderFactory {
    fn init(self: Box<RenderFactory>,
            io: &input::IOManager,
            mut window: Window,
            size: (i32, i32),
            cl: Option<Arc<hl::Device>>) -> RenderManager<RD> {

        let (sender, recv) = channel();
        window.make_context_current();
        let device = gfx::GlDevice::new(|s| io.get_proc_address(s));
        glfw::make_context_current(None);

        let (free_send, free_recv) = channel();
        Thread::spawn(move || {
            for db in free_recv.iter() {
                drop(db)
            }
        });

        let res = Thread::spawn(move || {
            let mut window = window;
            window.make_context_current();
            let recv: Receiver<RD> = recv;

            let mut rc = RenderManagerContext::_new(device, window, size, cl);
            loop {
                // wait for a copy of the game
                let mut db = match recv.recv() {
                    Ok(db) => db,
                    Err(_) => return
                };

                loop {
                    match recv.try_recv() {
                        Ok(mut _db) => {
                            std::mem::swap(&mut db, &mut _db);
                            free_send.send(_db);
                        }
                        // no newer copy
                        Err(std::sync::mpsc::TryRecvError::Empty) => break,
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => return,
                    }
                }
                rc.update(db);
            }
        });

        RenderManager {
            channel: sender,
            res: res
        }
    }
}

#[derive(Copy)]
pub struct RenderFactory;

impl RenderFactory {
    pub fn new() -> RenderFactory { RenderFactory }
}