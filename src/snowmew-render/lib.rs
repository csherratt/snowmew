#[crate_id = "github.com/csherratt/snowmew#snowmew-render:0.1"];
#[license = "ASL2"];
#[crate_type = "lib"];
#[comment = "A game engine in rust"];

extern mod std;
extern mod glfw;
extern mod cgmath;
extern mod snowmew;
extern mod cow;
extern mod gl;
extern mod OpenCL;

use std::ptr;
use std::vec;
use std::comm::{Chan, Port};

use cgmath::matrix::{ToMat4, Matrix};
//use drawlist::Drawlist;
use drawlist::{ObjectCull, Expand, DrawCommand, Draw, BindShader, BindVertexBuffer, SetMatrix};
use drawlist_cl::{ObjectCullOffloadContext};

use cgmath::matrix::{Mat4, ToMat4, Matrix, ToMat3};
use cgmath::vector::{Vec4, Vector};

use snowmew::core::{object_key, IterObjs};

mod db;
mod shader;
mod vertex_buffer;
mod drawlist;
mod drawlist_cl;


pub struct RenderManager {
    db: db::Graphics,
    render_chan: Chan<(db::Graphics, i32, Mat4<f32>)>,
    result_port: Port<Option<~[DrawCommand]>>,
}

fn render_db<'a>(db: db::Graphics, scene: i32, camera: Mat4<f32>, chan: &Chan<Option<~[DrawCommand]>>,
    cull_cl: &mut ObjectCullOffloadContext)
{
    let mut list = Expand::new(cull_cl.iter(db.current.walk_drawables(scene), camera), &db);

    let mut out = vec::with_capacity(512);
    for cmd in list {
        out.push(cmd);

        if out.len() == 512 {
            chan.send(Some(out));
            out = vec::with_capacity(512);
        }
    }
    chan.send(Some(out));
    chan.send(None);
}

impl RenderManager
{
    pub fn new(db: snowmew::core::Database) -> RenderManager
    {
        // todo move!
        gl::Enable(gl::SCISSOR_TEST);
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::LINE_SMOOTH);
        gl::Enable(gl::BLEND);
        gl::CullFace(gl::BACK);

        let (render_port, render_chan): (Port<(db::Graphics, i32, Mat4<f32>)>, Chan<(db::Graphics, i32, Mat4<f32>)>) = Chan::new();
        let (result_port, result_chan): (Port<Option<~[DrawCommand]>>, Chan<Option<~[DrawCommand]>>) = Chan::new();

        do spawn {
            let result_chan = result_chan;
            let (device, context, queue) = OpenCL::util::create_compute_context_prefer(OpenCL::util::GPU_PREFERED).unwrap();
            let mut offload = ObjectCullOffloadContext::new(&context, &device, queue);

            for (db, scene, camera) in render_port.iter() {
                render_db(db, scene, camera, &result_chan, &mut offload);
            }
        }

        RenderManager {
            db: db::Graphics::new(db),
            result_port: result_port,
            render_chan: render_chan
        }
    }

    pub fn load(&mut self)
    {
        self.db.load();
    }

    pub fn update(&mut self, db: snowmew::core::Database)
    {
        self.db.update(db);
    }

    pub fn render(&mut self, scene: i32, camera: i32)
    {
        let projection = cgmath::projection::perspective(
            cgmath::angle::deg(60f32), 1920f32/1080f32, 0.01f32, 1000f32
        );
        let camera = self.db.current.location(camera).unwrap();
        let camera = camera.translate().rotate().to_mat4();


        let projection = projection.mul_m(&camera);

        let vec = Vec4::new(0_f32, 0_f32, 0_f32, 1_f32);
        let vec = projection.mul_v(&vec);
        let vec = Vec4::new(vec.x/vec.w, vec.y/vec.w, vec.z/vec.w, vec.z/vec.w);

        self.render_chan.send((self.db.clone(), scene, projection));

        let mut shader = None;
        let mut cmds = 0;
        for block in self.result_port.iter() {
            let block = match block {
                Some(block) => { block },
                None => {println!("batches: {}", cmds); return;}
            };

            for &item in block.iter() {
                match item {
                    BindShader(id) => {
                        shader = self.db.shaders.find(&id);
                        shader.unwrap().bind();
                        shader.unwrap().set_projection(&projection);
                    },
                    BindVertexBuffer(id) => {
                        let vb = self.db.vertex.find(&id);
                        vb.unwrap().bind();
                    },
                    SetMatrix(mat) => {
                        shader.unwrap().set_position(&mat);
                    },
                    Draw(geo) => {
                        unsafe {
                            gl::DrawElements(gl::TRIANGLES_ADJACENCY, geo.count as i32, gl::UNSIGNED_INT, ptr::null());
                        }
                        cmds += 1;
                    },
                }
            }
        }

        println!("batches: {}\n", cmds);
    }
}