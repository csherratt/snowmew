#![crate_id = "github.com/csherratt/snowmew#snowmew-render-gfx:0.1"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "A game engine in rust"]
#![allow(dead_code)]

//extern crate debug;
extern crate std;
extern crate glfw;
extern crate gfx;
extern crate snowmew;
extern crate OpenCL;
extern crate sync;
extern crate position = "snowmew-position";
extern crate graphics = "snowmew-graphics";

use std::task;
use std::rt;
use std::comm::{Receiver, Sender};
use std::mem;
use std::sync::TaskPool;
use std::sync::Future;
use std::collections::hashmap::HashMap;

use OpenCL::hl::{CommandQueue, Context, Device};
use sync::Arc;

use position::Positions;
use graphics::Graphics;
use snowmew::common::ObjectKey;
use snowmew::io::Window;

use graphics::geometry::{Vertex, VertexGeo, VertexGeoTex, VertexGeoNorm, VertexGeoTexNorm, VertexGeoTexNormTan};
use graphics::geometry::{Geo, GeoTex, GeoNorm, GeoTexNorm, GeoTexNormTan};


static VERTEX_SRC: &'static [u8] = b"
    #version 150 core
    in vec2 a_Pos;
    out vec4 v_Color;
    void main() {
        v_Color = vec4(a_Pos+0.5, 0.0, 1.0);
        gl_Position = vec4(a_Pos, 0.0, 1.0);
    }
";

static FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core
    in vec4 v_Color;
    out vec4 o_Color;
    uniform sampler3D tex3D;
    uniform MyBlock {
        vec4 color;
    } block;
    void main() {
        vec4 texel = texture(tex3D, vec3(0.5,0.5,0.5));
        vec4 unused = mix(texel, block.color, 0.5);
        o_Color = v_Color.x<0.0 ? unused : v_Color;
    }
";

pub trait RenderData : Graphics + Positions {}

struct Mesh {
    mesh: uint,
    index: u32
}

pub struct RenderManager {
    client: gfx::Renderer,
    program: Option<uint>,
    meshes: HashMap<ObjectKey, Mesh>
}

impl RenderManager {
    fn _new(server: gfx::Device<snowmew::io::Window>,
            client: gfx::Renderer,
            _: (i32, i32),
            _: Option<Arc<Device>>) -> RenderManager {

        glfw::make_context_current(None);
        spawn(proc() {
            let mut server = server;
            server.make_current();
            loop { server.update(); }
        });

        RenderManager {
            client: client,
            program: None,
            meshes: HashMap::new()
        }
    }

    fn load<RD: RenderData>(&mut self, db: &RD) {
        if self.program.is_none() {
            self.program = Some(self.client.create_program(VERTEX_SRC.to_owned(),
                                                           FRAGMENT_SRC.to_owned()))
        }

        for (oid, vb) in db.vertex_buffer_iter() {
            if self.meshes.find(oid).is_none() {
                let mut data: Vec<f32> = Vec::new();
                match vb.vertex {
                    Geo(ref d) => {
                        for v in d.iter() {
                            data.push(v.position.x);
                            data.push(v.position.y);
                            data.push(v.position.z);
                        }
                    },
                    GeoTex(ref d) => {
                        for v in d.iter() {
                            data.push(v.position.x);
                            data.push(v.position.y);
                            data.push(v.position.z);
                        }
                    },
                    GeoNorm(ref d) => {
                        for v in d.iter() {
                            data.push(v.position.x);
                            data.push(v.position.y);
                            data.push(v.position.z);
                        }
                    },
                    GeoTexNorm(ref d) => {
                        for v in d.iter() {
                            data.push(v.position.x);
                            data.push(v.position.y);
                            data.push(v.position.z);
                        }
                    },
                    GeoTexNormTan(ref d) => {
                        for v in d.iter() {
                            data.push(v.position.x);
                            data.push(v.position.y);
                            data.push(v.position.z);
                        }
                    }
                }
                let mesh = self.client.create_mesh((data.len() / 3) as u16,
                                                   data,
                                                   3,
                                                   12);

                let mut index = Vec::new();
                for &i in vb.index.iter() {
                    index.push(i as u16);
                }

                let index = self.client.create_index_buffer(index);

                self.meshes.insert(*oid, Mesh {
                    index: index,
                    mesh: mesh
                });
            }
        }
    }
}


impl<RD: RenderData+Send> snowmew::Render<RD> for RenderManager {
    fn update(&mut self, db: RD, scene: ObjectKey, camera: ObjectKey) {
        self.load(&db);
    }
}

impl<RD: RenderData+Send> snowmew::RenderFactory<RD, RenderManager> for RenderFactory {
    fn init(self,
            io: &snowmew::IOManager,
            window: Window,
            size: (i32, i32),
            cl: Option<Arc<Device>>) -> RenderManager {

        window.make_context_current();
        match gfx::start(window, io) {
            Ok((render, device)) => RenderManager::_new(device, render, size, cl),
            Err(err) => fail!("failed to start gfx: {}", err)
        }
    }
}

pub struct RenderFactory;

impl RenderFactory {
    pub fn new() -> RenderFactory { RenderFactory }
}