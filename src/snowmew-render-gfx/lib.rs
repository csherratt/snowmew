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
#![feature(libc)]
#![feature(collections)]
#![allow(dead_code)]
#![plugin(gfx_macros)]


extern crate libc;
#[cfg(feature="use_opencl")]
extern crate opencl;
extern crate glfw;

extern crate gfx;
extern crate "gfx_device_gl" as device;
extern crate genmesh;
extern crate cgmath;
extern crate draw_state;

extern crate "snowmew-core" as snowmew;
extern crate "snowmew-position" as position;
extern crate "snowmew-graphics" as graphics;
extern crate "snowmew-render" as sm_render;
extern crate "snowmew-input" as input;
extern crate collect;

use std::collections::{HashMap, BTreeSet};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::spawn;

#[cfg(feature="use_opencl")]
use std::sync::Arc;

#[cfg(feature="use_opencl")]
use opencl::hl;
use gfx::{Device, DeviceExt};
use gfx::batch::RefBatch;
use cgmath::*;
use collect::iter::{OrderedMapIterator, OrderedSetIterator};

use position::Positions;
use graphics::Graphics;
use snowmew::common::Entity;
use sm_render::camera::Camera;
use graphics::Material;
use graphics::geometry::{VertexGeoTex, VertexGeoTexNorm};
use graphics::geometry::Vertex::{Geo, GeoTex, GeoNorm, GeoTexNorm, GeoTexNormTan};
use sm_render::Renderable;
use input::{Window, GetIoState};
use gfx::render;

#[derive(Copy, Clone)]
struct SharedMatrix {
    proj_mat: [[f32; 4]; 4],
    view_mat: [[f32; 4]; 4]
}

#[derive(Copy, Clone)]
struct SharedMaterial {
    ka_color: [f32; 4],
    kd_color: [f32; 4],
    ks_color: [f32; 4],

    ka_use_texture: i32,
    kd_use_texture: i32,
    ks_use_texture: i32,
}

const VERTEX_SRC: &'static [u8] = b"
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
";

const FRAGMENT_SRC: &'static [u8] = b"
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

        float level = shadow_sum * max(0, dot(light_normal, normal));
        level = round(level * 2.) / 2.;

        color = ka * 0.2 + kd * level * light_color;

        o_Color = color;
    }
";

#[shader_param]
#[derive(Debug, Clone)]
struct Params<R: gfx::Resources> {
    shadow_shared_mat: gfx::RawBufferHandle<R>,
    shared_mat: gfx::RawBufferHandle<R>,

    shadow_bias_mat: [[f32; 4]; 4],

    material: gfx::RawBufferHandle<R>,
    ka_texture: gfx::shade::TextureParam<R>,
    kd_texture: gfx::shade::TextureParam<R>,
    ks_texture: gfx::shade::TextureParam<R>,

    light_normal: [f32; 4],
    light_color: [f32; 4],
    shadow: gfx::shade::TextureParam<R>,

    model: gfx::RawBufferHandle<R>,
    offset: i32
}

const BACK_FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    out vec4 o_Color;

    void main() {
        o_Color = vec4(0., 0., 0., 1.);
    }
";

static SHADOW_VERTEX_SRC: &'static [u8] = b"
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
";

static SHADOW_FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    void main() {}
";

#[shader_param]
#[derive(Debug, Clone)]
struct ShadowParams<R: gfx::Resources> {
    shared_mat: gfx::RawBufferHandle<R>,
    model: gfx::RawBufferHandle<R>,
    offset: i32
}

struct Mesh<R: gfx::Resources> {
    mesh: render::mesh::Mesh<R>,
    index: gfx::BufferHandle<device::GlResources, u32>
}

struct RenderMaterial {
    material: Material,
    buffer: gfx::BufferHandle<device::GlResources, SharedMaterial>,
    ka_texture: Option<Entity>,
    kd_texture: Option<Entity>,
    ks_texture: Option<Entity>,
}

pub struct RenderManagerContext {
    prog: gfx::ProgramHandle<device::GlResources>,
    data: Params<device::GlResources>,

    shadow_data: ShadowParams<device::GlResources>,
    shadow_prog: gfx::ProgramHandle<device::GlResources>,
    shadow_frame: render::target::Frame<device::GlResources>,
    shadow: gfx::TextureHandle<device::GlResources>,
    shadow_sampler: gfx::SamplerHandle<device::GlResources>,
    shadow_shared_mat: gfx::BufferHandle<device::GlResources, SharedMatrix>,
    shared_mat: gfx::BufferHandle<device::GlResources, SharedMatrix>,

    back_data: ShadowParams<device::GlResources>,
    back_prog: gfx::ProgramHandle<device::GlResources>,

    render: gfx::render::Renderer<device::CommandBuffer>,
    device: device::GlDevice,
    context: gfx::render::batch::Context<device::GlResources>,
    frame: render::target::Frame<device::GlResources>,
    state: draw_state::DrawState,
    back_state: draw_state::DrawState,
    meshes: HashMap<Entity, Mesh<device::GlResources>>,
    textures: HashMap<Entity, gfx::TextureHandle<device::GlResources>>,
    sampler: gfx::SamplerHandle<device::GlResources>,
    window: Window,

    material: HashMap<Entity, RenderMaterial>,

    batch: BTreeSet<(Entity, Entity, Entity)>,
    shadow_batches: HashMap<Entity, RefBatch<ShadowParams<device::GlResources>>>,
    draw_batches: HashMap<Entity, RefBatch<Params<device::GlResources>>>,
    draw_back_batches: HashMap<Entity, RefBatch<ShadowParams<device::GlResources>>>,

    spare_matrix_buffers: Vec<gfx::BufferHandle<device::GlResources, [[f32; 4]; 4]>>,
    used_matrix_buffers: Vec<gfx::BufferHandle<device::GlResources, [[f32; 4]; 4]>>,
    shared_geometry: Vec<(u32, gfx::BufferHandle<device::GlResources, [[f32; 4]; 4]>, usize, usize)>,
    shared_geometry_material: Vec<(u32, u32, gfx::BufferHandle<device::GlResources, [[f32; 4]; 4]>, usize, usize)>,
}

pub struct RenderManager<R> {
    channel: Sender<R>,
    res: std::thread::JoinHandle
}

impl RenderManagerContext {
    fn _new(mut device: device::GlDevice,
            window: Window,
            size: (i32, i32)) -> RenderManagerContext {

        let (width, height) = size;
        let frame = gfx::Frame::new(width as u16, height as u16);
        let back_state = gfx::DrawState::new().depth(gfx::state::Comparison::LessEqual, true);
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

        let (back_prog, back_data) = {
            let data = ShadowParams {
                shared_mat: shared_mat.raw(),
                model: shared_mat.raw(),
                offset: 0
            };
            (device.link_program(SHADOW_VERTEX_SRC.clone(),
                                 BACK_FRAGMENT_SRC.clone())
                  .ok().expect("Failed to link program"),
             data)
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
            render: device.create_renderer(),
            device: device,
            context: gfx::batch::Context::new(),
            frame: frame,
            state: state,
            back_state: back_state,
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
            draw_back_batches: HashMap::new(),
            spare_matrix_buffers: Vec::new(),
            used_matrix_buffers: Vec::new(),
            shared_geometry: Vec::new(),
            shared_geometry_material: Vec::new(),
            back_prog: back_prog,
            back_data: back_data,
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
                        self.device.create_mesh(&data)
                    },
                    GeoTex(ref d) => {
                        self.device.create_mesh(&d)
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
                        self.device.create_mesh(&data)
                    },
                    GeoTexNorm(ref d) => {
                        self.device.create_mesh(&d)
                    },
                    GeoTexNormTan(ref d) => {
                        self.device.create_mesh(&d)
                    }
                };

                let vb: Vec<u32> = vb.index.iter().map(|&x| x as u32).collect();

                let index = self.device.create_buffer_static(&vb);

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
                let texture = self.device.create_texture(tinfo)
                                         .ok().expect("Failed to create texture");
                self.device.update_texture(&texture, &img_info, text.data())
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
            let buff = self.device.create_buffer_static(material);
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
        self.draw_back_batches.clear();

        for (id, draw) in db.scene_iter(scene).inner_join_map(db.drawable_iter()) {
            self.batch.insert((draw.geometry, draw.material, id));

            if !self.shadow_batches.contains_key(&draw.geometry) {
                let geo = db.geometry(draw.geometry).expect("failed to find geometry");
                let vb = self.meshes.get(&geo.vb).expect("Could not get vertex buffer");

                let batch: RefBatch<ShadowParams<device::GlResources>> = self.context.make_batch(
                    &self.shadow_prog,
                    self.shadow_data.clone(),
                    &vb.mesh,
                    gfx::Slice {
                        start: geo.offset as u32,
                        end: (geo.offset + geo.count) as u32,
                        prim_type: gfx::PrimitiveType::TriangleList,
                        kind: gfx::SliceKind::Index32(vb.index.clone(), 0)
                    },
                    &self.state
                ).ok().expect("Failed to create batch.");
                self.shadow_batches.insert(draw.geometry, batch);

                let batch: RefBatch<Params<device::GlResources>> = self.context.make_batch(
                    &self.prog,
                    self.data.clone(),
                    &vb.mesh,
                    gfx::Slice {
                        start: geo.offset as u32,
                        end: (geo.offset + geo.count) as u32,
                        prim_type: gfx::PrimitiveType::TriangleList,
                        kind: gfx::SliceKind::Index32(vb.index.clone(), 0)
                    },
                    &self.state
                ).ok().expect("Failed to create batch.");
                self.draw_batches.insert(draw.geometry, batch);

                let batch: RefBatch<ShadowParams<device::GlResources>> = self.context.make_batch(
                    &self.back_prog,
                    self.back_data.clone(),
                    &vb.mesh,
                    gfx::Slice {
                        start: geo.offset as u32,
                        end: (geo.offset + geo.count) as u32,
                        prim_type: gfx::PrimitiveType::TriangleList,
                        kind: gfx::SliceKind::Index32(vb.index.clone(), 0)
                    },
                    &self.back_state
                ).ok().expect("Failed to create batch.");
                self.draw_back_batches.insert(draw.geometry, batch);
            }
        }
    }

    fn fetch_matrix(&mut self) -> gfx::BufferHandle<device::GlResources, [[f32; 4]; 4]> {
        let buffer = if let Some(buffer) = self.spare_matrix_buffers.pop() {
            buffer
        } else {
            let x = self.device.create_buffer(512, gfx::BufferUsage::Static);
            x
        };
        let clone = buffer.clone();
        self.used_matrix_buffers.push(clone);
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
                    shared_gm.push((lg, lm, mat.clone(), matrices.len()-idx_gm, idx_gm));
                    idx_gm = matrices.len();
                }
                if lg != g {
                    shared_g.push((lg, mat.clone(), matrices.len()-idx_g, idx_g));
                    idx_g = matrices.len();
                }
            
                if matrices.len() == max {
                    shared_gm.push((lg, lm, mat.clone(), matrices.len()-idx_gm, idx_gm));
                    shared_g.push((lg, mat.clone(), matrices.len()-idx_g, idx_g));
                    self.device.update_buffer(mat.clone(), &matrices, 0);
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
            shared_gm.push((g, m, mat.clone(), matrices.len()-idx_gm, idx_gm));
            shared_g.push((g, mat.clone(), matrices.len()-idx_g, idx_g));
            self.device.update_buffer(mat.clone(), &matrices, 0)
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
        self.render.clear(cdata, gfx::DEPTH, &self.shadow_frame.clone());

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

        self.device.update_buffer(self.shadow_shared_mat.clone(), shadow_mat, 0);

        for &(geo, matrix, len, offset) in self.shared_geometry.iter() {
            let batch = self.shadow_batches.get_mut(&geo).expect("Missing draw");
            batch.params.model = matrix.raw();
            batch.params.offset = offset as i32;
            self.render.draw_instanced(
                &(&*batch, &self.context),
                len as u32,
                0,
                &self.shadow_frame.clone(),
            ).unwrap();
        };

    }

    fn draw<RD: Renderable+GetIoState>(&mut self, db: &RD) {
        let camera = db.camera().expect("no camera set");

        let cdata = gfx::ClearData {
            color: [0.3, 0.3, 0.3, 1.0],
            depth: 1.0,
            stencil: 0,
        };
        self.render.clear(cdata, gfx::COLOR | gfx::DEPTH, &self.frame);

        let (width, height) = db.get_io_state().size;
        let camera_trans = db.position(camera);
        let camera = Camera::new(width, height, camera_trans);

        let proj = camera.projection_matrix();
        let view = camera.view_matrix();

        let shared_mat = &[SharedMatrix {
            view_mat: view.into_fixed(),
            proj_mat: proj.into_fixed()
        }];

        self.device.update_buffer(self.shared_mat.clone(), shared_mat, 0);

        let mut light_color = [0., 0., 0., 1.];
        let mut light_normal = [0., 0., 0., 1.];
        for (key, light) in db.light_iter() {
            match light {
                &graphics::Light::Point(_) => {}
                &graphics::Light::Directional(d) => {
                    let n = d.normal();
                    let n = Vector4::new(n.x, n.y, n.z, 0.);
                    let n = db.position(key).mul_v(&n).normalize();
                    let color = d.color().mul_s(d.intensity());
                    light_color = [color.x, color.y, color.z, 1.];
                    light_normal = [n.x, n.y, n.z, n.w];
                }
            }
        }

        self.draw_shadow(&camera);

        for &(geo, _, ref matrix, len, offset) in self.shared_geometry_material.iter() {
            let batch = self.draw_back_batches.get_mut(&geo).expect("Missing draw");
            batch.params.model = matrix.clone().raw();
            batch.params.offset = offset as i32;

            self.render.draw_instanced(
                &(&*batch, &self.context),
                len as u32,
                0,
                &self.frame.clone(),
            ).unwrap();
        };

        for &(geo, mat, ref matrix, len, offset) in self.shared_geometry_material.iter() {
            let batch = self.draw_batches.get_mut(&geo).expect("Missing draw");
            let mat = self.material.get(&mat).expect("Could not find material");
            if let Some(ka) = mat.ka_texture {
                batch.params.ka_texture =
                    (*self.textures.get(&ka)
                          .expect("Could not find texture"),
                     Some(self.sampler));
            }
            if let Some(kd) = mat.kd_texture {
                batch.params.kd_texture =
                    (*self.textures.get(&kd)
                          .expect("Could not find texture"),
                     Some(self.sampler));
            }
            if let Some(ks) = mat.ks_texture {
                batch.params.ks_texture =
                    (*self.textures.get(&ks)
                          .expect("Could not find texture"),
                     Some(self.sampler));
            }
            batch.params.material = mat.buffer.raw();
            batch.params.shadow = (self.shadow, Some(self.shadow_sampler));
            batch.params.model = matrix.clone().raw();
            batch.params.offset = offset as i32;
            batch.params.light_normal = light_normal;
            batch.params.light_color = light_color;

            self.render.draw_instanced(
                &(&*batch, &self.context),
                len as u32,
                0,
                &self.frame.clone(),
            ).unwrap();
        };

        self.device.submit(self.render.as_buffer());
        self.render.reset();
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

impl<RD: Renderable+GetIoState+Send+'static> sm_render::Render<RD> for RenderManager<RD> {
    fn update(&mut self, db: RD) {
        self.channel.send(db).unwrap();
    }
}

#[cfg(feature="use_opencl")]
impl<RD: Renderable+GetIoState+Send+'static> sm_render::RenderFactory<RD, RenderManager<RD>> for RenderFactory {
    fn init(self: Box<RenderFactory>,
            io: &input::IOManager,
            mut window: Window,
            size: (i32, i32),
            _: Option<Arc<hl::Device>>) -> RenderManager<RD> {

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

            let mut rc = RenderManagerContext::_new(device, window, size);
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

#[cfg(not(feature="use_opencl"))]
impl<RD: Renderable+GetIoState+Send+'static> sm_render::RenderFactory<RD, RenderManager<RD>> for RenderFactory {
    fn init(self: Box<RenderFactory>,
            io: &input::IOManager,
            mut window: Window,
            size: (i32, i32)) -> RenderManager<RD> {

        let (sender, recv) = channel();
        window.make_context_current();
        let device = device::GlDevice::new(|s| io.get_proc_address(s));
        glfw::make_context_current(None);

        let (free_send, free_recv) = channel();
        spawn(move || {
            for db in free_recv.iter() {
                drop(db)
            }
        });

        let res = spawn(move || {
            let mut window = window;
            window.make_context_current();
            let recv: Receiver<RD> = recv;

            let mut rc = RenderManagerContext::_new(device, window, size);
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
                            free_send.send(_db).unwrap();
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