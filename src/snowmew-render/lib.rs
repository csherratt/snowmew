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
extern mod extra;
extern mod native;

use std::ptr;
use std::vec;
use std::cast;
use std::comm::{SharedChan, Port, Empty, Disconnected, Data};

//use drawlist::Drawlist;
use cgmath::matrix::{Mat4, Matrix};
use cow::join::join_maps;

use snowmew::core::{object_key};
use snowmew::camera::Camera;
use snowmew::display::Display;
use snowmew::input::InputHandle;

use extra::time::precise_time_s;

use db::Graphics;
use pipeline::{DrawTarget, Pipeline};
use drawlist::{Drawlist, Expand, DrawCommand, Draw, BindMaterial, BindVertexBuffer, SetModelMatrix, MultiDraw, DrawElements2, DrawElements};

mod db;
mod shader;
mod vertex_buffer;
mod drawlist;
mod hmd;
mod pipeline;


enum RenderCommand {
    Update(snowmew::core::Database, object_key, object_key),
    Waiting(SharedChan<(db::Graphics, Drawlist, object_key)>),
    Complete(Drawlist),
    Finish
}

fn swap_buffers(disp: &mut Display)
{
    disp.swap_buffers();
    unsafe {
        gl::DrawElements(gl::TRIANGLES, 6i32, gl::UNSIGNED_INT, ptr::null());
        let sync = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
        gl::ClientWaitSync(sync, gl::SYNC_FLUSH_COMMANDS_BIT, 1_000_000_000u64);
        gl::DeleteSync(sync);
    }
}

fn render_task(chan: SharedChan<RenderCommand>)
{
    let (p, c) = SharedChan::new();
    chan.send(Waiting(c.clone()));
    loop {
        let (g, mut dl, oid) = p.recv();
        dl.setup_scene(&g, oid);
        chan.send(Waiting(c.clone()));
        chan.send(Complete(dl));
    }
}

fn render_server(port: Port<RenderCommand>, db: snowmew::core::Database, display: Display, ih: InputHandle)
{
    let mut db = db::Graphics::new(db);
    let mut scene = 0;
    let mut camera = 0;
    let mut display = display;
    let mut pipeline = pipeline::Forward::new();

    display.make_current();
    gl::load_with(glfw::get_proc_address);

    // todo move!
    gl::Enable(gl::SCISSOR_TEST);
    gl::Enable(gl::DEPTH_TEST);
    gl::Enable(gl::CULL_FACE);
    gl::Enable(gl::LINE_SMOOTH);
    gl::Enable(gl::BLEND);
    gl::CullFace(gl::BACK);
    glfw::set_swap_interval(0);

    db.load();

    let mut drawlists = ~[Drawlist::new(256*1024), Drawlist::new(256*1024)];
    let mut waiting = ~[];

    let mut last = precise_time_s();

    loop {
        let cmd = if drawlists.len() == 0 || waiting.len() == 0 || scene == 0{
            Some(port.recv())
        } else {
            match port.try_recv() {
                Empty => None,
                Disconnected => return,
                Data(dat) => Some(dat)
            }
        };

        match cmd {
            Some(Update(new, s, c)) => {
                db.update(new);
                db.load();
                scene = s;
                camera = c;
            },
            Some(Waiting(ch)) => {
                if scene != 0 && drawlists.len() != 0 {
                    ch.send((db.clone(), drawlists.pop().unwrap(), scene));
                } else {
                    waiting.push(ch);
                }
            },
            Some(Complete(mut dl)) => {
                let camera_rot = db.current.location(camera).unwrap().get().rot;
                let camera_trans = db.current.position(camera);
                let camera = Camera::new(camera_rot, camera_trans.clone());

                let dt = DrawTarget::new(0, (0, 0), display.size());

                pipeline.render(&mut dl, &db, &camera, &dt);
                swap_buffers(&mut display);
                drawlists.push(dl);
            },
            Some(Finish) => {
                return
            },
            None => {
                if drawlists.len() > 0 && waiting.len() > 0 {
                    let ch = waiting.pop().unwrap();
                    ch.send((db.clone(), drawlists.pop().unwrap(), scene));
                }  
            }
        }
    }
}


pub fn render(dl: &mut Drawlist, db: &Graphics, camera: object_key, display: &mut Display)
{
    if display.is_hmd() {
        //self.render_vr(dl, camera, win)
    } else {
        render_normal(dl, db, camera, display)
    }
}


fn render_normal(dl: &mut Drawlist, db: &Graphics, camera: object_key, display: &mut Display)
{
    gl::ClearColor(0., 0., 0., 1.);
    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

    //dl.render(db, projection);

    swap_buffers(display);
}

/*fn render_vr(dl: &mut Drawlist, db: &Graphics, scene: object_key, display: &mut Display)
{
    let hmd_info = display.hmd();

    if self.hmd.is_none() {
        self.hmd = Some(hmd::HMD::new(1.7, &hmd_info));
    }

    let camera_rot = self.db.current.location(camera).unwrap().get().rot;
    let camera_trans = self.db.current.position(camera);
    let (left, right) = Camera::new(camera_rot, camera_trans.clone()).get_matrices_ovr(display);

    let proj_left = left.projection.mul_m(&left.view);
    self.render_chans[0].send((self.db.clone(), scene, proj_left));
    let proj_right = right.projection.mul_m(&right.view);
    self.render_chans[1].send((self.db.clone(), scene, proj_right));

    self.hmd.unwrap().set_left(&self.db, &hmd_info);
    self.drawsink(proj_left, 0);

    self.hmd.unwrap().set_right(&self.db, &hmd_info);
    self.drawsink(proj_right, 1);

    self.hmd.unwrap().draw_screen(&self.db, &hmd_info);
    self.swap_buffers(display);
}*/


pub struct RenderManager
{
    priv ch: SharedChan<RenderCommand>
}

impl RenderManager
{
    pub fn new(db: snowmew::core::Database, display: Display, ih: InputHandle) -> RenderManager
    {
        let (port, chan) = SharedChan::new();

        native::task::spawn(proc() {
            let mut db = db;
            let mut display = display;
            let mut ih = ih;

            render_server(port, db, display, ih);
        });

        let task_c = chan.clone();
        native::task::spawn(proc() {
            render_task(task_c);
        });

        RenderManager {
            ch: chan
        }
    }

    pub fn update(&mut self, db: snowmew::core::Database, scene: object_key, camera: object_key)
    {
        self.ch.send(Update(db, scene, camera));
    }
}

impl Drop for RenderManager
{
    fn drop(&mut self) {
        self.ch.send(Finish)
    }
}