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

pub use config::Config;

use std::ptr;
use std::mem;
use std::comm::{Receiver, Sender, Empty, Disconnected};

use snowmew::core::{ObjectKey};
use snowmew::camera::Camera;
use snowmew::io::Window;

use pipeline::{DrawTarget, Pipeline};
use drawlist::{Drawlist, DrawlistStandard};
use OpenCL::hl::{CommandQueue};
use time::precise_time_s;

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
    Update(snowmew::core::Database, ObjectKey, ObjectKey),
    Waiting(Sender<Option<~DrawlistStandard>>),
    Complete(~DrawlistStandard),
    Setup(Sender<Option<CommandQueue>>),
    Finish(Sender<()>)
}

fn swap_buffers(disp: &mut Window)
{
    disp.swap_buffers();
    unsafe {
        gl::DrawElements(gl::TRIANGLES, 6i32, gl::UNSIGNED_INT, ptr::null());
        let sync = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
        gl::ClientWaitSync(sync, gl::SYNC_FLUSH_COMMANDS_BIT, 1_000_000_000u64);
        gl::DeleteSync(sync);
    }
}

fn render_task(chan: Sender<RenderCommand>)
{
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

fn render_server(port: Receiver<RenderCommand>, db: snowmew::core::Database, window: Window, size: (i32, i32))
{
    let (_, _, queue) = OpenCL::util::create_compute_context_prefer(OpenCL::util::GPUPrefered).unwrap();

    let mut queue = Some(queue);

    let mut db = db::Graphics::new(db);
    let mut scene = 0;
    let mut camera = 0;
    let mut window = window;
    let cfg = Config::new(window.get_context_version());

    window.make_context_current();

    let mut pipeline = //if window.is_hmd() {
        //~pipeline::Hmd::new(pipeline::Forward::new(), 1.7, &window.hmd()) as ~pipeline::Pipeline
    /*} else */ {
        let (width, height) = size;
        ~pipeline::Defered::new(pipeline::Forward::new(), width as uint, height as uint) as ~pipeline::Pipeline
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

    let mut drawlists = if cfg.use_bindless() {
        ~[]
    } else {
        ~[DrawlistStandard::from_config(&cfg),
          DrawlistStandard::from_config(&cfg)]
    };

    let mut num_workers = 1;
    let mut waiting = ~[];

    let mut time = precise_time_s();

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
                let rot = db.current.location(camera).unwrap().get().rot;
                let camera_trans = db.current.position(camera);

                //let input = ih.get();
                //let rift = input.predicted;
                //et rift = rift.mul_q(&Rotation3::from_axis_angle(&Vec3::new(0f32, 1f32, 0f32), deg(180 as f32).to_rad()));

                let camera = Camera::new(rot, camera_trans);

                let dt = DrawTarget::new(0, (0, 0), (1920, 1080));

                pipeline.render(&mut *dl as &mut Drawlist, &db, &camera.get_matrices(size), &dt);

                swap_buffers(&mut window);
                
                let end = precise_time_s();
                print!("\rfps: {:3.2f}", 1./(end-time));
                time = end;

                drawlists.push(dl);
            },
            Some(Finish(ack)) => {
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
                ack.send(());
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

pub struct RenderManager
{
    ch: Sender<RenderCommand>
}

impl RenderManager
{
    pub fn new(db: snowmew::core::Database, window: Window, size: (i32, i32)) -> RenderManager
    {
        let (sender, receiver) = channel();

        let mut taskopts = std::task::TaskOpts::new();
        taskopts.name = Some("render-main".into_maybe_owned());

        native::task::spawn_opts(taskopts, proc() {
            let db = db;
            let window = window;

            render_server(receiver, db, window, size);
        });


        let mut taskopts = std::task::TaskOpts::new();
        taskopts.name = Some("render worker #0".into_maybe_owned());

        let task_c = sender.clone();
        native::task::spawn_opts(taskopts, proc() {
            render_task(task_c);
        });
        
        RenderManager { ch: sender }
    }

    pub fn update(&mut self, db: snowmew::core::Database, scene: ObjectKey, camera: ObjectKey)
    {
        self.ch.send(Update(db, scene, camera));
    }
}

impl Drop for RenderManager
{
    fn drop(&mut self) {
        let (c, p) = channel();
        self.ch.send(Finish(c));
        let _ = self.ch;
        p.recv();
    }
}