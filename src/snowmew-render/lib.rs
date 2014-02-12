#[crate_id = "github.com/csherratt/snowmew#snowmew-render:0.1"];
#[license = "ASL2"];
#[crate_type = "lib"];
#[comment = "A game engine in rust"];
#[allow(dead_code)];

extern mod std;
extern mod glfw = "glfw-rs";
extern mod cgmath;
extern mod snowmew;
extern mod cow;
extern mod gl;
extern mod OpenCL;
extern mod ovr = "ovr-rs";
extern mod collections;

use std::ptr;
use std::vec;
use std::comm::{Chan, Port};

//use drawlist::Drawlist;
use drawlist::{Drawlist, Expand, DrawCommand, Draw, BindMaterial, BindVertexBuffer, SetModelMatrix, MultiDraw, DrawElements};

use cgmath::matrix::{Mat4, Matrix};
use cow::join::join_maps;

use snowmew::core::{object_key};
use snowmew::camera::Camera;

use snowmew::display::Display;


mod db;
mod shader;
mod vertex_buffer;
mod drawlist;
mod hmd;

pub struct RenderManager {
    db: db::Graphics,
    hmd: Option<hmd::HMD>,
    render_chans: ~[Chan<(db::Graphics, object_key, Mat4<f32>)>],
    result_ports: ~[Port<Option<~[DrawCommand]>>],
    drawlist: Drawlist

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

        let mut result_ports = ~[];
        let mut render_chans = ~[];

        for _ in range(0, 2) {
            let (render_port, render_chan): (Port<(db::Graphics, object_key, Mat4<f32>)>, Chan<(db::Graphics, object_key, Mat4<f32>)>) = Chan::new();
            let (result_port, result_chan): (Port<Option<~[DrawCommand]>>, Chan<Option<~[DrawCommand]>>) = Chan::new();
        
            spawn(proc() {
                let result_chan = result_chan;
                for (db, scene, camera) in render_port.iter() {
                    render_db(db, scene, camera, &result_chan);
                }
            });

            result_ports.push(result_port);
            render_chans.push(render_chan);

        }

        RenderManager {
            db: db::Graphics::new(db),
            hmd: None,
            result_ports: result_ports,
            render_chans: render_chans,
            drawlist: Drawlist::new()
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

    fn drawsink(&mut self, projection: Mat4<f32>, port: int)
    {
        let mut material = None;
        let mut shader = self.db.flat_shader.unwrap();

        shader.bind();
        shader.set_projection(&projection);

        for block in self.result_ports[port].iter() {
            let block = match block {
                Some(block) => { block },
                None => {break;}
            };

            for &item in block.iter() {
                match item {
                    BindMaterial(id) => {
                        material = self.db.current.material(id);
                        match material {
                            Some(material) => shader.set_material(material),
                            None => {
                                println!("material {} not found", id);
                            }
                        }
                    },
                    BindVertexBuffer(id) => {
                        let vb = self.db.vertex.find(&id);
                        vb.unwrap().bind();
                    },
                    SetModelMatrix(mat) => {
                        shader.set_model(&mat);
                    },
                    Draw(geo) => {
                        unsafe {
                            gl::DrawElements(gl::TRIANGLES, geo.count as i32, gl::UNSIGNED_INT, ptr::null());
                        }
                    },
                    _ => (),
                }
            }
        }       
    }

    fn swap_buffers(&mut self, win: &mut Display)
    {
        win.swap_buffers();
        unsafe {
            gl::DrawElements(gl::TRIANGLES, 6i32, gl::UNSIGNED_INT, ptr::null());
            let sync = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
            gl::ClientWaitSync(sync, gl::SYNC_FLUSH_COMMANDS_BIT, 1_000_000_000u64);
            gl::DeleteSync(sync);
        }
    }

    pub fn render(&mut self, scene: object_key, camera: object_key, win: &mut Display)
    {
        if win.is_hmd() {
            self.render_vr(scene, camera, win)
        } else {
            self.render_normal(scene, camera, win)
        }
    }

    fn render_normal(&mut self, scene: object_key, camera: object_key, win: &mut Display)
    {
        let camera_rot = self.db.current.location(camera).unwrap().get().rot;
        let camera_trans = self.db.current.position(camera);
        let camera = Camera::new(camera_rot, camera_trans.clone()).get_matrices(win);

        let projection = camera.projection.mul_m(&camera.view);

        self.db.current.mark_time(~"start");

        gl::ClearColor(0., 0., 0., 1.);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        {
            let mut material = None;
            let mut shader = self.db.flat_instanced_shader.unwrap();

            shader.bind();
            shader.set_projection(&projection);

            self.drawlist.setup_scene(&self.db, scene);
            let list = self.drawlist.generate(&self.db);

            self.db.current.mark_time(~"drawlist");

            for cmd in list.iter() {
                match *cmd {
                    BindMaterial(id) => {
                        material = self.db.current.material(id);
                        match material {
                            Some(material) => shader.set_material(material),
                            None => {
                                println!("material {} not found", id);
                            }
                        }
                    },
                    BindVertexBuffer(id) => {
                        let vb = self.db.vertex.find(&id);
                        vb.unwrap().bind();
                    },
                    SetModelMatrix(mat) => {
                        shader.set_model(&mat);
                    },
                    Draw(geo) => {
                        unsafe {
                            gl::DrawElements(gl::TRIANGLES, geo.count as i32, gl::UNSIGNED_INT, ptr::null());
                        }
                    },
                    MultiDraw(vbo, mat, indirect, offset, len) => {
                        let vb = self.db.vertex.find(&vbo);
                        vb.unwrap().bind();
                        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, mat);
                        gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, indirect);

                        unsafe {
                        gl::DrawElementsIndirect(gl::TRIANGLES,
                                                 gl::UNSIGNED_INT,
                                                (20*offset) as *std::libc::c_void);
                        }

                    },
                    DrawElements(vbo, mat, count, first_index, base_vertex, base_instance, instance) => {
                        let vb = self.db.vertex.find(&vbo);
                        vb.unwrap().bind();
                        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, mat);
                        gl::Uniform1i(1, base_instance as i32);
                        unsafe {
                            gl::DrawElementsInstancedBaseVertexBaseInstance(gl::TRIANGLES,
                                                count,
                                                gl::UNSIGNED_INT,
                                                first_index as *std::libc::c_void,
                                                instance,
                                                base_vertex,
                                                base_instance);
                        }
                    }
                }

            }
        }

//        self.render_chans[0].send((self.db.clone(), scene, projection));//



//        self.drawsink(projection, 0);

        self.db.current.mark_time(~"render scene");
        self.swap_buffers(win);
        self.db.current.mark_time(~"swap_buffers");
        self.db.current.dump_time();
    }

    fn render_vr(&mut self, scene: object_key, camera: object_key, win: &mut Display)
    {
        let hmd_info = win.hmd();

        if self.hmd.is_none() {
            self.hmd = Some(hmd::HMD::new(1.7, &hmd_info));
        }

        let camera_rot = self.db.current.location(camera).unwrap().get().rot;
        let camera_trans = self.db.current.position(camera);
        let (left, right) = Camera::new(camera_rot, camera_trans.clone()).get_matrices_ovr(win);

        let proj_left = left.projection.mul_m(&left.view);
        self.render_chans[0].send((self.db.clone(), scene, proj_left));
        let proj_right = right.projection.mul_m(&right.view);
        self.render_chans[1].send((self.db.clone(), scene, proj_right));

        self.hmd.unwrap().set_left(&self.db, &hmd_info);
        self.drawsink(proj_left, 0);

        self.hmd.unwrap().set_right(&self.db, &hmd_info);
        self.drawsink(proj_right, 1);

        self.hmd.unwrap().draw_screen(&self.db, &hmd_info);
        self.swap_buffers(win);
    }
}