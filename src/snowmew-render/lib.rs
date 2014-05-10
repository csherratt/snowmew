 #![crate_id = "github.com/csherratt/snowmew#snowmew-render:0.1"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "A game engine in rust"]
#![allow(dead_code)]

extern crate std;
extern crate glfw;
extern crate cgmath;
extern crate snowmew;
extern crate cow;
extern crate gl;
extern crate OpenCL;
extern crate ovr = "oculus-vr";
extern crate collections;
extern crate native;
extern crate time;
extern crate libc;
extern crate sync;
extern crate gl_cl;
extern crate position = "snowmew-position";
extern crate graphics = "snowmew-graphics";

use std::ptr;
use std::mem;
use std::task::{TaskResult, TaskBuilder};
use std::comm::{Receiver, Sender, Empty, Disconnected};

use cgmath::matrix::{Matrix, ToMatrix4};
use cgmath::vector::{Vector3};
use cgmath::rotation::{Rotation3};
use cgmath::angle::{ToRad, deg};

use OpenCL::hl::{CommandQueue, Context, Device};
use sync::Arc;

use snowmew::common::ObjectKey;
use snowmew::camera::Camera;
use snowmew::io::Window;
use position::Positions;
use graphics::Graphics;

pub use config::Config;

use pipeline::{DrawTarget, Pipeline};
use drawlist::{Drawlist, DrawlistStandard};

mod db;
mod shader;
mod vertex_buffer;
mod drawlist;
mod hmd;
mod pipeline;
mod query;
mod compute_accelerator;
mod config;

pub trait RenderData : Graphics + Positions + Clone {}

enum RenderCommand {
    Update(Box<RenderData:Send>, ObjectKey, ObjectKey),
    Waiting(Sender<Option<DrawlistStandard>>),
    Complete(DrawlistStandard),
    Setup(Sender<Option<CommandQueue>>),
    Finish
}

fn swap_buffers(disp: &mut Window) {
    disp.swap_buffers();
    unsafe {
        gl::DrawElements(gl::TRIANGLES, 6i32, gl::UNSIGNED_INT, ptr::null());
        let sync = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
        gl::ClientWaitSync(sync, gl::SYNC_FLUSH_COMMANDS_BIT, 1_000_000_000u64);
        gl::DeleteSync(sync);
    }
}

fn render_task(chan: Sender<RenderCommand>) {
    let (sender, receiver) = channel();
    chan.send(Setup(sender));
    let _ = receiver.recv();

    let (sender, receiver) = channel();
    chan.send(Waiting(sender.clone()));
    loop {
        for dl in receiver.iter() {
            match dl {
                Some(mut dl) => {
                    dl.setup_scene_async();
                    chan.send(Waiting(sender.clone()));
                    chan.send(Complete(dl));                    
                },
                None => {
                    println!("render task: exiting");
                    return
                }
            }

        }
    }
}

fn render_server(port: Receiver<RenderCommand>, db: Box<RenderData>, window: Window, size: (i32, i32),
                 cl: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) {
    let (_, _, queue) = OpenCL::util::create_compute_context_prefer(OpenCL::util::GPUPrefered).unwrap();

    let mut queue = Some(queue);

    let mut db = db::GlState::new(db);
    let mut scene = 0;
    let mut camera = 0;
    let mut window = window;
    let cfg = Config::new(window.get_context_version());

    window.make_context_current();

    let mut pipeline = {
        let (width, height) = size;
        if !window.is_hmd() {
            box pipeline::Defered::new(pipeline::Forward::new(), width as uint, height as uint) as Box<Pipeline>
        } else {
            box pipeline::Hmd::new(
                pipeline::Defered::new(pipeline::Forward::new(), width as uint, height as uint),
                1.,
                window.hmdinfo()
            ) as Box<Pipeline>
        }
    };

    // todo move!
    gl::Enable(gl::SCISSOR_TEST);
    gl::Enable(gl::DEPTH_TEST);
    gl::Enable(gl::CULL_FACE);
    gl::Enable(gl::LINE_SMOOTH);
    gl::Enable(gl::BLEND);
    gl::CullFace(gl::BACK);

    db.load(&cfg);

    //let accl = PositionGlAccelerator::new();

    let mut drawlists = vec!(DrawlistStandard::from_config(&cfg, cl.clone()),
                             DrawlistStandard::from_config(&cfg, cl.clone()));

    let mut num_workers = 1;
    let mut waiting = Vec::new();

    loop {
        let cmd = if drawlists.len() == 0 || waiting.len() == 0 || scene == 0 {
            Some(port.recv())
        } else {
            match port.try_recv() {
                Err(Empty) => None,
                Err(Disconnected) => return,
                Ok(dat) => Some(dat)
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
                    ch.send(Some(dl));
                } else {
                    waiting.push(ch);
                }
            },
            Some(Complete(mut dl)) => {
                dl.setup_scene();
                let rot = db.location(camera).unwrap().get().rot;
                let camera_trans = db.position(camera);

                let camera = if window.is_hmd() {
                    let sf = window.sensor_fusion();
                    let rift = sf.get_predicted_orientation(None);
                    let rift = rift.mul_q(&Rotation3::from_axis_angle(&Vector3::new(0f32, 1f32, 0f32), deg(180 as f32).to_rad()));
                    Camera::new(rot.mul_q(&rift), camera_trans.mul_m(&rift.to_matrix4()))
                } else {
                    Camera::new(rot, camera_trans)
                };

                let dt = DrawTarget::new(0, (0, 0), (1280, 800), ~[gl::BACK_LEFT]);

                pipeline.render(&mut dl, &db, &camera.get_matrices(size), &dt);
                swap_buffers(&mut window);
                drawlists.push(dl);
            },
            Some(Finish) => {
                // flush the port, this should release any
                // async drawlist workers
                println!("render: dropping waiting");
                while waiting.len() > 0 {
                    let c = waiting.pop().unwrap();
                    c.send(None);
                    num_workers -= 1;
                }
                println!("render: waiting for open connections to close");
                while num_workers > 0 {
                    match port.recv() {
                        Waiting(ch) => {
                            num_workers -= 1;
                            ch.send(None)
                        },
                        _ => ()
                    }
                }
                println!("render: exiting");
                return;
            },
            None => {
                if drawlists.len() > 0 && waiting.len() > 0 {
                    println!("sending");
                    let ch = waiting.pop().unwrap();
                    let mut dl = drawlists.pop().unwrap();
                    dl.bind_scene(db.clone(), scene);
                    ch.send(Some(dl));
                }  
            }
        }
    }
}

pub struct RenderManager {
    ch: Sender<RenderCommand>,
    render_done: Receiver<TaskResult>
}

impl RenderManager {
    fn _new(db: Box<RenderData:Send>, window: Window, size: (i32, i32), dev: Option<Arc<Device>>) -> RenderManager {
        let (sender, receiver) = channel();

        window.make_context_current();
        let cl = match dev {
            Some(dev) => {
                let ctx = gl_cl::create_context(dev.deref());
                match ctx {
                    Some(ctx) => {
                        let queue = ctx.create_command_queue(dev.deref());
                        Some((Arc::new(ctx), Arc::new(queue), dev))
                    }
                    None => None
                }
            },
            None => None
        };
        glfw::make_context_current(None);

        let mut taskbuilder = TaskBuilder::new();
        taskbuilder = taskbuilder.named("render-main".into_maybe_owned());
        let render_main_result = taskbuilder.future_result();

        taskbuilder.spawn(proc() {
            let db = db;
            let window = window;

            render_server(receiver, db, window, size, cl);
        });

        let mut taskopts = std::task::TaskOpts::new();
        taskopts.name = Some("render worker #0".into_maybe_owned());

        let task_c = sender.clone();
        native::task::spawn_opts(taskopts, proc() {
            render_task(task_c);
        });

        RenderManager { 
            ch: sender,
            render_done: render_main_result
        }
    }

    pub fn new_cl(db: Box<RenderData:Send>, window: Window, size: (i32, i32), device: Arc<Device>) -> RenderManager {
        RenderManager::_new(db, window, size, Some(device))
    }

    pub fn new(db: Box<RenderData:Send>, window: Window, size: (i32, i32)) -> RenderManager {
        RenderManager::_new(db, window, size, None)
    }

    pub fn update(&mut self, db: Box<RenderData:Send>, scene: ObjectKey, camera: ObjectKey) {
        self.ch.send(Update(db, scene, camera));
    }
}

impl Drop for RenderManager {
    fn drop(&mut self) {
        self.ch.send(Finish);
        drop(self.render_done.recv());
    }
}