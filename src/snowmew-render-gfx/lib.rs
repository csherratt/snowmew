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
extern crate cow;
extern crate gl;
extern crate time;
extern crate device;
extern crate position = "snowmew-position";
extern crate graphics = "snowmew-graphics";
extern crate render_data = "render-data";


use std::collections::hashmap::HashMap;

use OpenCL::hl::Device;
use sync::Arc;

use position::Positions;
use graphics::Graphics;
use snowmew::common::ObjectKey;
use snowmew::io::Window;

use graphics::geometry::{Geo, GeoTex, GeoNorm, GeoTexNorm, GeoTexNormTan};

use cow::join::{join_set_to_map, join_maps};
use render_data::RenderData;

static VERTEX_SRC: &'static [u8] = b"
    #version 150 core
    uniform mat4 proj_mat;
    uniform mat4 view_mat;
    uniform mat4 model_mat;

    in vec3 a_Pos;
    out vec4 v_Color;
    void main() {
        v_Color = vec4(a_Pos.xy+0.5, 0.0, 1.0);
        gl_Position = proj_mat * view_mat * model_mat * vec4(a_Pos, 1.0);
    }
";

static FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core
    in vec4 v_Color;
    out vec4 o_Color;
    uniform vec4 color;
    void main() {
        o_Color = color;
    }
";


struct Mesh {
    mesh: uint,
    index: u32
}

struct Env {
    env: gfx::EnvirHandle,
    proj_mat: gfx::UniformVar,
    view_mat: gfx::UniformVar,
    model_mat: gfx::UniformVar,
    color: gfx::UniformVar,
}

pub struct RenderManager {
    client: gfx::Renderer,
    program: Option<gfx::ProgramHandle>,
    environment: Option<Env>,
    meshes: HashMap<ObjectKey, Mesh>
}

impl RenderManager {
    fn _new(server: gfx::Device<snowmew::io::Window, device::Device>,
            client: gfx::Renderer,
            _: (i32, i32),
            _: Option<Arc<Device>>) -> RenderManager {

        glfw::make_context_current(None);
        spawn(proc() {
            let mut server = server;
            server.make_current();
            let mut start = time::precise_time_s();
            loop { 
                server.update();
                let end = time::precise_time_s();
                println!("{:4.1}fps", 1. / (end - start));
                start = end;
            }
        });

        RenderManager {
            client: client,
            program: None,
            environment: None,
            meshes: HashMap::new()
        }
    }

    fn load<RD: RenderData>(&mut self, db: &RD) {
        if self.program.is_none() {
            self.program = Some(
                self.client.create_program(VERTEX_SRC.to_owned(),
                                           FRAGMENT_SRC.to_owned())
            );
        }

        if self.environment.is_none() {
            let mut env = gfx::Environment::new();
            let proj_mat = env.add_uniform("proj_mat",
                gfx::ValueF32Matrix(
                    [[1., 0., 0., 0.],
                     [0., 1., 0., 0.],
                     [0., 0., 1., 0.],
                     [0., 0., 0., 1.]]
                )
            );
            let view_mat = env.add_uniform("view_mat",
                gfx::ValueF32Matrix(
                    [[1., 0., 0., 0.],
                     [0., 1., 0., 0.],
                     [0., 0., 1., 0.],
                     [0., 0., 0., 1.]]
                )
            );
            let model_mat = env.add_uniform("model_mat",
                gfx::ValueF32Matrix(
                    [[1., 0., 0., 0.],
                     [0., 1., 0., 0.],
                     [0., 0., 1., 0.],
                     [0., 0., 0., 1.]]
                )
            );
            let color = env.add_uniform("color", gfx::ValueF32Vec([0.1, 0.1, 0.1, 0.1]));
            self.environment = Some(Env {
                env: self.client.create_environment(env),
                proj_mat: proj_mat,
                view_mat: view_mat,
                model_mat: model_mat,
                color: color
            });
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

    fn draw<RD: RenderData>(&mut self, db: &RD, scene: ObjectKey, camera: ObjectKey) {
        let env = self.environment.as_ref().expect("Could not get environment");
        let cdata = gfx::ClearData {
            color: Some(device::Color([0.3, 0.3, 0.3, 1.0])),
            depth: None,
            stencil: None,
        };
        self.client.clear(cdata, None);

        let camera_trans = db.position(camera);
        let camera = snowmew::camera::Camera::new(camera_trans);

        let proj = camera.projection_matrix(16. / 9.);
        self.client.set_env_uniform(
            env.env,
            env.proj_mat, 
            gfx::ValueF32Matrix(
                [[proj.x.x, proj.x.y, proj.x.z, proj.x.w],
                 [proj.y.x, proj.y.y, proj.y.z, proj.y.w],
                 [proj.z.x, proj.z.y, proj.z.z, proj.z.w],
                 [proj.w.x, proj.w.y, proj.w.z, proj.w.w]]
            )
        );

        let view = camera.view_matrix();
        self.client.set_env_uniform(
            env.env,
            env.view_mat, 
            gfx::ValueF32Matrix(
                [[view.x.x, view.x.y, view.x.z, view.x.w],
                 [view.y.x, view.y.y, view.y.z, view.y.w],
                 [view.z.x, view.z.y, view.z.z, view.z.w],
                 [view.w.x, view.w.y, view.w.z, view.w.w]]
            )
        );

        for (id, (draw, pos)) in join_set_to_map(db.scene_iter(scene),
                                                 join_maps(db.drawable_iter(),
                                                           db.location_iter())) {

            let geo = db.geometry(draw.geometry).expect("failed to find geometry");
            let mat = db.material(draw.material).expect("Could not find material");
            let vb = self.meshes.find(&geo.vb).expect("Could not get vertex buffer");

            let model = db.position(*id);
            self.client.set_env_uniform(
                env.env,
                env.model_mat, 
                gfx::ValueF32Matrix(
                    [[model.x.x, model.x.y, model.x.z, model.x.w],
                     [model.y.x, model.y.y, model.y.z, model.y.w],
                     [model.z.x, model.z.y, model.z.z, model.z.w],
                     [model.w.x, model.w.y, model.w.z, model.w.w]]
                )
            );

            let ka = mat.ka();
            self.client.set_env_uniform(
                env.env,
                env.color,
                gfx::ValueF32Vec([ka.x, ka.y, ka.z, 1.])
            );

            self.client.draw(vb.mesh, 
                             gfx::IndexSlice(vb.index, geo.offset as u16, geo.count as u16),
                             None,
                             self.program.unwrap(),
                             env.env
            );
        }

        self.client.end_frame();
    }
}


impl<RD: RenderData+Send> snowmew::Render<RD> for RenderManager {
    fn update(&mut self, db: RD, scene: ObjectKey, camera: ObjectKey) {
        self.load(&db);
        self.draw(&db, scene, camera);
    }
}

impl<RD: RenderData+Send> snowmew::RenderFactory<RD, RenderManager> for RenderFactory {
    fn init(~self,
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