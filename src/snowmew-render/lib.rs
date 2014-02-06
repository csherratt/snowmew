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
extern mod ovr = "ovr-rs";

use std::ptr;
use std::vec;
use std::comm::{Chan, Port};

//use drawlist::Drawlist;
use drawlist::{Expand, DrawCommand, Draw, BindShader, BindVertexBuffer, SetMatrix};

use cgmath::matrix::{Mat4, Matrix};
use cow::join::join_maps;

use snowmew::core::{object_key};
use snowmew::camera::Camera;

use ovr::{HMDInfo, create_reference_matrices};

use snowmew::display::Display;


mod db;
mod shader;
mod vertex_buffer;
mod drawlist;
mod drawlist_cl;
mod hmd;

pub struct RenderManager {
    db: db::Graphics,
    hmd: Option<hmd::HMD>,
    render_chan: Chan<(db::Graphics, object_key, Mat4<f32>)>,
    result_port: Port<Option<~[DrawCommand]>>,
}

fn render_db<'a>(db: db::Graphics, scene: object_key, camera: Mat4<f32>, chan: &Chan<Option<~[DrawCommand]>>)
{
    let mut list = Expand::new(join_maps(db.current.walk_in_camera(scene, &camera), db.current.walk_drawables()), &db);

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

        spawn(proc() {
            let result_chan = result_chan;
            //let (device, context, queue) = OpenCL::util::create_compute_context_prefer(OpenCL::util::GPU_PREFERED).unwrap();
            //let mut offload = ObjectCullOffloadContext::new(&context, &device, queue);

            for (db, scene, camera) in render_port.iter() {
                render_db(db, scene, camera, &result_chan);
            }
        });

        RenderManager {
            db: db::Graphics::new(db),
            hmd: None,
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

    pub fn render(&mut self, scene: object_key, camera: object_key, win: &mut Display)
    {
        let (w, h) = win.size();
        let (w, h) = (w as f32, h as f32);
        let projection = cgmath::projection::perspective(
            cgmath::angle::deg(80f32), w/h, 1f32, 1000f32
        );

        let camera_rot = self.db.current.location(camera).unwrap().get().rot;
        let camera_trans = self.db.current.position(camera);
        let camera = Camera::new(camera_rot, camera_trans.clone()).view_matrix();

        let projection = projection.mul_m(&camera);

        self.render_chan.send((self.db.clone(), scene, projection));


        gl::ClearColor(0., 0., 0., 1.);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        let mut shader = None;
        for block in self.result_port.iter() {
            let block = match block {
                Some(block) => { block },
                None => {break;}
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
                            gl::DrawElements(gl::TRIANGLES, geo.count as i32, gl::UNSIGNED_INT, ptr::null());
                        }
                    },
                }
            }
        }

        win.swap_buffers();
        unsafe {
            gl::DrawElements(gl::TRIANGLES, 6i32, gl::UNSIGNED_INT, ptr::null());
            let sync = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
            gl::ClientWaitSync(sync, gl::SYNC_FLUSH_COMMANDS_BIT, 1_000_000_000u64);
        }
    }

    pub fn render_vr(&mut self, scene: object_key, camera: object_key, hmd: &HMDInfo, win: &mut Display)
    {
        if self.hmd.is_none() {
            self.hmd = Some(hmd::HMD::new(1.7, hmd));
        }

        let camera_rot = self.db.current.location(camera).unwrap().get().rot;
        let camera_trans = self.db.current.position(camera);
        let camera = Camera::new(camera_rot, camera_trans.clone()).view_matrix();

        let ((proj_left, proj_right), (view_left, view_right)) = 
                create_reference_matrices(hmd, &camera, self.hmd.unwrap().scale);


        for x in range(0, 2) {
            let proj = if x == 0 {
                self.hmd.unwrap().set_left(&self.db, hmd);
                proj_left.mul_m(&view_left)
            } else {
                self.hmd.unwrap().set_right(&self.db, hmd);
                proj_right.mul_m(&view_right)
            };
            self.render_chan.send((self.db.clone(), scene, proj));

            let mut shader = None;
            for block in self.result_port.iter() {
                let block = match block {
                    Some(block) => { block },
                    None => {break;}
                };

                for &item in block.iter() {
                    match item {
                        BindShader(id) => {
                            shader = self.db.shaders.find(&id);
                            shader.unwrap().bind();
                            shader.unwrap().set_projection(&proj);
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
                                gl::DrawElements(gl::TRIANGLES, geo.count as i32, gl::UNSIGNED_INT, ptr::null());
                            }
                        },
                    }
                }
            }
        }

        self.hmd.unwrap().draw_screen(&self.db, hmd);

        win.swap_buffers();
        //unsafe {
        //    gl::DrawElements(gl::TRIANGLES, 6i32, gl::UNSIGNED_INT, ptr::null());
        //    let sync = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
        //    gl::ClientWaitSync(sync, gl::SYNC_FLUSH_COMMANDS_BIT, 1_000_000_000u64);
        //}


    }
}