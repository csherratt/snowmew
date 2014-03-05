#[crate_id = "github.com/csherratt/snowmew#snowmew-render:0.1"];
#[license = "ASL2"];
#[crate_type = "lib"];
#[comment = "A game engine in rust"];
#[allow(dead_code)];

extern crate std;
extern crate glfw = "glfw-rs";
extern crate cgmath;
extern crate snowmew;
extern crate cow;
extern crate gl;
extern crate OpenCL;
extern crate ovr = "ovr-rs";
extern crate collections;
extern crate extra;
extern crate native;
extern crate time;

pub use config::Config;

use std::ptr;
use std::mem;
use std::comm::{Chan, Port, Empty, Disconnected, Data};

use cgmath::vector::Vec3;
use cgmath::matrix::{Matrix, ToMat4};
use cgmath::rotation::Rotation3;
use cgmath::angle::{ToRad, deg};

use snowmew::core::{object_key};
use snowmew::camera::Camera;
use snowmew::display::Display;
use snowmew::input::InputHandle;

use db::Graphics;
use pipeline::{DrawTarget, Pipeline};
use drawlist::{Drawlist, DrawlistBindless, DrawlistStandard};
use OpenCL::hl::{CommandQueue};
use query::Query;
use compute_accelerator::PositionGlAccelerator;

mod db;
mod shader;
mod vertex_buffer;
mod drawlist;
mod hmd;
mod pipeline;
mod query;
mod compute_accelerator;
mod config;

enum RenderCommand {
    Update(snowmew::core::Database, object_key, object_key),
    Waiting(Chan<~Drawlist>),
    Complete(~Drawlist),
    Setup(Chan<Option<CommandQueue>>),
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

fn render_task(chan: Chan<RenderCommand>)
{
    let (p, c) = Chan::new();
    chan.send(Setup(c.clone()));
    let queue = p.recv();

    println!("queue {:?}", queue);

    let (p, c) = Chan::new();
    chan.send(Waiting(c.clone()));
    loop {
        let mut dl = p.recv();
        dl.setup_scene_async();
        chan.send(Waiting(c.clone()));
        chan.send(Complete(dl));
    }
}

fn render_server(port: Port<RenderCommand>, db: snowmew::core::Database, display: Display, ih: InputHandle)
{
    let (device, context, queue) = OpenCL::util::create_compute_context_prefer(OpenCL::util::GPU_PREFERED).unwrap();

    let mut queue = Some(queue);

    let mut db = db::Graphics::new(db);
    let mut scene = 0;
    let mut camera = 0;
    let mut display = display;
    let mut cfg = Config::new(display.get_context_version());

    display.make_current();
    gl::load_with(glfw::get_proc_address);


    let mut pipeline = if display.is_hmd() {
        ~pipeline::Hmd::new(pipeline::Forward::new(), 1.7, &display.hmd()) as ~pipeline::Pipeline
    } else {
        let (width, height) = display.size();
        ~pipeline::Defered::new(pipeline::Forward::new(), width, height) as ~pipeline::Pipeline
    };

    // todo move!
    gl::Enable(gl::SCISSOR_TEST);
    gl::Enable(gl::DEPTH_TEST);
    gl::Enable(gl::CULL_FACE);
    gl::Enable(gl::LINE_SMOOTH);
    gl::Enable(gl::BLEND);
    gl::CullFace(gl::BACK);
    glfw::set_swap_interval(1);

    db.load(&cfg);

    //let accl = PositionGlAccelerator::new();

    let mut drawlists = if cfg.use_bindless() {
        ~[]
    } else {
        ~[DrawlistStandard::from_config(&cfg),
          DrawlistStandard::from_config(&cfg)]
    };

    let mut waiting = ~[];

    let mut render_calc = Query::new();
    let mut render_scene = Query::new();

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
            Some(Setup(ch)) => {
                let mut out = None;
                mem::swap(&mut queue, &mut out);
                ch.send(out)
            },
            Some(Update(new, s, c)) => {
                db.update(new);
                db.load(&cfg);
                scene = s;
                camera = c;
            },
            Some(Waiting(ch)) => {
                if scene != 0 && drawlists.len() != 0 {
                    let mut dl = drawlists.pop().unwrap();
                    dl.bind_scene(db.clone(), scene);
                    ch.send(dl);
                } else {
                    waiting.push(ch);
                }
            },
            Some(Complete(mut dl)) => {
                println!("drawing");
                let time = render_calc.start_time();
                //dl.calc_pos(&accl);
                time.end();

                let render = render_scene.start_time();
                let rot = db.current.location(camera).unwrap().get().rot;
                let camera_trans = db.current.position(camera);

                let input = ih.get();
                let rift = input.predicted;
                let rift = rift.mul_q(&Rotation3::from_axis_angle(&Vec3::new(0f32, 1f32, 0f32), deg(180 as f32).to_rad()));

                let camera = Camera::new(rot.mul_q(&rift), camera_trans.mul_m(&rift.to_mat4()));

                let dt = DrawTarget::new(0, (0, 0), display.size());

                println!("pipeline");
                pipeline.render(dl, &db, &camera.get_matrices(display.size()), &dt);
                render.end();

                swap_buffers(&mut display);
                drawlists.push(dl);

                println!("{} {}", time.time_sync(), render.time_sync());

                render_calc = time.to_query();
                render_scene = render.to_query();
            },
            Some(Finish) => {
                return
            },
            None => {
                if drawlists.len() > 0 && waiting.len() > 0 {
                    println!("sending");
                    let ch = waiting.pop().unwrap();
                    let mut dl = drawlists.pop().unwrap();
                    dl.bind_scene(db.clone(), scene);
                    ch.send(dl);
                }  
            }
        }
    }
}

pub struct RenderManager
{
    priv ch: Chan<RenderCommand>
}

impl RenderManager
{
    pub fn new(db: snowmew::core::Database, display: Display, ih: InputHandle) -> RenderManager
    {
        let (port, chan) = Chan::new();

        let mut taskopts = std::task::TaskOpts::new();
        taskopts.name = Some("render-main".into_maybe_owned());

        native::task::spawn_opts(taskopts, proc() {
            let db = db;
            let display = display;
            let ih = ih;

            render_server(port, db, display, ih);
        });


        let mut taskopts = std::task::TaskOpts::new();
        taskopts.name = Some("render worker #0".into_maybe_owned());

        let task_c = chan.clone();
        native::task::spawn_opts(taskopts, proc() {
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