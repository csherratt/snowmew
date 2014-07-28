 #![crate_name = "snowmew-render"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "A game engine in rust"]
#![allow(dead_code)]

//extern crate debug;
extern crate std;
extern crate glfw;
extern crate cgmath;
extern crate snowmew;
extern crate cow;
extern crate gl;
extern crate opencl;
extern crate ovr = "oculus-vr";
extern crate collections;
extern crate native;
extern crate time;
extern crate libc;
extern crate sync;
extern crate gl_cl;
extern crate collision;
extern crate position = "snowmew-position";
extern crate graphics = "snowmew-graphics";
extern crate render_data = "render-data";

use std::task;
use std::rt;
use std::comm::{Receiver, Sender};
use std::mem;
use std::sync::TaskPool;
use std::sync::Future;
use time::precise_time_s;

use opencl::hl::{CommandQueue, Context, Device};
use sync::Arc;

use snowmew::common::ObjectKey;
use snowmew::camera::Camera;
use snowmew::io::Window;
use position::Positions;

pub use config::Config;

use pipeline::Pipeline;
use drawlist::{Drawlist, create_drawlist};
use query::{ProfilerDummy, TimeQueryManager, Profiler};

use render_data::RenderData;

mod db;
mod shader;
mod vertex_buffer;
mod drawlist;
mod pipeline;
mod query;
mod compute_accelerator;
mod config;
mod texture;
mod material;
mod light;
mod model;
mod matrix;
mod command;

enum RenderCommand {
    Update(Box<RenderData+Send>, ObjectKey, ObjectKey),
    Finish
}

fn render_thread(input: Receiver<(Box<Drawlist+Send>, ObjectKey)>,
                 output: Sender<Box<Drawlist+Send>>,
                 window: Window,
                 size: (i32, i32),
                 config: Config,
                 cl: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) {

    window.make_context_current();
    let mut db = db::GlState::new();

    let mut pipeline = {
        if !window.is_hmd() {
            box pipeline::Swap::new(
                pipeline::Defered::new(pipeline::Forward::new()),
                window
            ) as Box<Pipeline>
        } else {
            box pipeline::Hmd::new(
                pipeline::Defered::new(pipeline::Forward::new()),
                window,
                &config
            ) as Box<Pipeline>
        }
    };

    let (width, height) = size;
    pipeline.resize(width as uint, height as uint);

    // todo move!
    gl::Enable(gl::SCISSOR_TEST);

    for _ in range(1, config.drawlist_count()) {
        let mut dl = create_drawlist(&config, cl.clone());
        dl.setup_begin();
        output.send(dl);
    }

    let mut next_dl = create_drawlist(&config, cl.clone());

    let mut qm = if config.profile() {
        box TimeQueryManager::new() as Box<Profiler>
    } else {
        box ProfilerDummy as Box<Profiler>
    };
    let mut last_frame = precise_time_s();
    for (mut dl, camera) in input.iter() {
        qm.time("setup complete".to_string());
        dl.setup_complete(&mut db, &config);

        let capture = precise_time_s();
        let camera_trans = dl.position(camera);
        let camera = Camera::new(camera_trans);

        pipeline.render(dl, &mut db, &camera, qm);
        // if the device is a hmd we need to stall the gpu
        // to make sure it actually flipped the buffers

        if config.fps() {
            let end = precise_time_s();
            println!("total: {:4.2f}ms capture: {:4.2f}ms {:4.1}fps", 
                (end - dl.start_time()) * 1000., (end - capture) * 1000.,
                1. / (end - last_frame));
            last_frame = end;
        }

        qm.time("setup begin".to_string());
        mem::swap(&mut next_dl, &mut dl);
        dl.setup_begin();
        output.send(dl);

        qm.dump();
        qm.reset();
    }
}

fn render_server(command: Receiver<RenderCommand>,
                 window: Window,
                 size: (i32, i32),
                 dev: Option<Arc<Device>>) {

    let config = Config::new(window.get_context_version());

    let cl = if config.opencl() {
        setup_opencl(&window, dev)
    } else {
        None
    };

    let mut taskbuilder = task::TaskBuilder::new();
    taskbuilder = taskbuilder.named("render-thread".into_maybe_owned());

    let (send_drawlist_setup, receiver_drawlist_setup) = channel();
    let (send_drawlist_ready, receiver_drawlist_ready) = channel();
    taskbuilder.spawn(proc() {
        let window = window;
        render_thread(receiver_drawlist_setup,
                      send_drawlist_ready,
                      window,
                      size,
                      config,
                      cl
        );
    });

    let (send_drawlist_render, receiver_drawlist_render)
        : (Sender<Box<Drawlist+Send>>, Receiver<Box<Drawlist+Send>>) = channel();
    let mut taskpool = TaskPool::new(config.thread_pool_size(), || { 
        let ch = send_drawlist_render.clone();
        proc(_: uint) { ch.clone() }
    });

    let mut drawlists_ready = Vec::new();

    let (mut db, mut scene, mut camera) = match command.recv() {
        Update(rd, s, c) => (rd, s, c),
        Finish => {
            return;
        }
    };

    let select = std::comm::Select::new();
    let mut receiver_drawlist_ready_handle = select.handle(&receiver_drawlist_ready);
    let mut receiver_drawlist_render_handle = select.handle(&receiver_drawlist_render);
    let mut command_handle = select.handle(&command);

    unsafe {
        receiver_drawlist_ready_handle.add();
        receiver_drawlist_render_handle.add();
        command_handle.add();
    }

    'finished: loop {
        let id = select.wait();
        if id == receiver_drawlist_ready_handle.id() {
            let dl = receiver_drawlist_ready_handle.recv();
            drawlists_ready.push(dl);
        } else if id == receiver_drawlist_render_handle.id() {
            let dl = receiver_drawlist_render_handle.recv();
            send_drawlist_setup.send((dl, camera));
        } else if id == command_handle.id() {
            let command = command_handle.recv();
            match command {
                Update(rd, s, c) => {
                    scene = s;
                    camera = c;
                    db = rd;
                }
                Finish => {
                    break 'finished;
                }
            }
        }

        if drawlists_ready.len() > 0 && scene != 0 {
            let dl = drawlists_ready.pop().unwrap();
            dl.setup_compute(db, &mut taskpool, scene);
            scene = 0;           
        }
    }
}

fn setup_opencl(window: &Window, dev: Option<Arc<Device>>) -> Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)> {
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
    cl 
}

pub struct RenderManager {
    ch: Sender<RenderCommand>,
    render_done: Future<rt::task::Result>
}

impl RenderManager {
    fn _new(window: Window, size: (i32, i32), dev: Option<Arc<Device>>) -> RenderManager {
        let mut taskbuilder = task::TaskBuilder::new();
        taskbuilder = taskbuilder.named("render-server".into_maybe_owned());

        let (sender, receiver) = channel();
        let render_main_result = taskbuilder.try_future(proc() {
            let window = window;

            render_server(receiver, window, size, dev.clone());
        });

        RenderManager {
            ch: sender,
            render_done: render_main_result
        }
    }

    pub fn new_cl(window: Window, size: (i32, i32), device: Arc<Device>) -> RenderManager {
        RenderManager::_new(window, size, Some(device))
    }

    pub fn new(window: Window, size: (i32, i32)) -> RenderManager {
        RenderManager::_new(window, size, None)
    }

    pub fn update(&mut self, db: Box<RenderData+Send>, scene: ObjectKey, camera: ObjectKey) {
        self.ch.send(Update(db, scene, camera));
    }
}

impl Drop for RenderManager {
    fn drop(&mut self) {
        self.ch.send(Finish);
        self.render_done.get_ref();
    }
}

impl<RD: RenderData+Send> snowmew::Render<RD> for RenderManager {
    fn update(&mut self, db: RD, scene: ObjectKey, camera: ObjectKey) {
        self.ch.send(Update(box db, scene, camera));
    }
}

impl<RD: RenderData+Send> snowmew::RenderFactory<RD, RenderManager> for RenderFactory {
    fn init(self: Box<RenderFactory>, _: &snowmew::IOManager, window: Window, size: (i32, i32), cl: Option<Arc<Device>>) -> RenderManager {
        RenderManager::_new(window, size, cl)
    }
}

pub struct RenderFactory;

impl RenderFactory {
    pub fn new() -> RenderFactory { RenderFactory }
}