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

use std::ptr;
use std::comm::{Chan, Port};

use cgmath::matrix::{ToMat4, Matrix};
//use drawlist::Drawlist;
use drawlist::{ObjectCull, Expand, DrawCommand, Done, Draw, BindShader, BindVertexBuffer, SetMatrix};

use cgmath::matrix::{Mat4, ToMat4, Matrix};
use cgmath::vector::{Vec4, Vector};

mod db;
mod shader;
mod vertex_buffer;
mod drawlist;

pub struct RenderManager {
    db: db::Graphics,
    render_chan: Chan<(db::Graphics, i32, Mat4<f32>)>,
    result_port: Port<DrawCommand>
}

fn render_db(db: db::Graphics, scene: i32, camera: Mat4<f32>, chan: &Chan<DrawCommand>)
{
    let mut list = Expand::new(ObjectCull::new(db.current.walk_drawables(scene), camera), &db);

    for cmd in list {
        chan.try_send_deferred(cmd);
    }
    chan.send(Done);
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
        let (result_port, result_chan): (Port<DrawCommand>, Chan<DrawCommand>) = Chan::new();

        do spawn {
            let result_chan = result_chan;
            for (db, scene, camera) in render_port.iter() {
                render_db(db, scene, camera, &result_chan);
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
            cgmath::angle::deg(72f32), 1024f32/768f32, 0.01f32, 10000f32
        );
        let camera = self.db.current.location(camera).unwrap();
        let projection = projection.mul_m(&camera.get().to_mat4());

        self.render_chan.send((self.db.clone(), scene, projection));

        let mut shader = None;
        loop {
            match self.result_port.recv() {
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
                },
                Done => {
                    break;
                }
            }
        }
    }
}