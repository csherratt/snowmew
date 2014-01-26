#[crate_id = "github.com/csherratt/snowmew#snowmew-render:0.1"];
#[license = "ASL2"];
#[crate_type = "lib"];
#[comment = "A game engine in rust"];
#[allow(dead_code)];

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

//use drawlist::Drawlist;
use drawlist::{Expand, DrawCommand, Draw, BindShader, BindVertexBuffer, SetMatrix};
use drawlist_cl::{ObjectCullOffloadContext};

use cgmath::matrix::{Mat4, ToMat4, ToMat3, Matrix};

use snowmew::core::{object_key};

mod db;
mod shader;
mod vertex_buffer;
mod drawlist;
mod drawlist_cl;


pub struct RenderManager {
    db: db::Graphics,
    render_chan: Chan<(db::Graphics, object_key, Mat4<f32>)>,
    result_port: Port<Option<~[DrawCommand]>>,
}

fn render_db<'a>(db: db::Graphics, scene: object_key, camera: Mat4<f32>, chan: &Chan<Option<~[DrawCommand]>>,
    _: &mut ObjectCullOffloadContext)
{
    let mut list = Expand::new(db.current.walk_drawables(scene, &camera), &db);

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

        let (render_port, render_chan): (Port<(db::Graphics, object_key, Mat4<f32>)>, Chan<(db::Graphics, object_key, Mat4<f32>)>) = Chan::new();
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

    pub fn render(&mut self, scene: object_key, camera: object_key)
    {
        let projection = cgmath::projection::perspective(
            cgmath::angle::deg(60f32), 1920f32/1080f32, 1f32, 1000f32
        );

        let camera_obj = self.db.current.object(camera).unwrap();
        let camera_parent = self.db.current.position(camera_obj.parent);
        let camera_trans = self.db.current.location(camera).unwrap();

        let camera = camera_trans.get().rot.to_mat3().to_mat4().mul_m(&camera_parent);

        let projection = projection.mul_m(&camera);

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