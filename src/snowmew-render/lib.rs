//   Copyright 2014 Colin Sherratt
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

#![crate_name = "snowmew-render"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "A game engine in rust"]
#![allow(dead_code)]
#![feature(phase)]
#![feature(if_let)]

//extern crate debug;

extern crate collections;
extern crate time;
extern crate libc;
extern crate rustrt;

extern crate glfw;
extern crate cgmath;
extern crate cow;
extern crate gl;
extern crate opencl;
extern crate ovr;
extern crate collision;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;

extern crate gl_cl;

extern crate "snowmew-core" as snowmew;
extern crate "snowmew-position" as position;
extern crate "snowmew-graphics" as graphics;
extern crate "snowmew-render-data" as render_data;

use std::task;
use std::comm::{Receiver, Sender};
use std::mem;
use std::sync::{TaskPool, Future, Arc};
use time::precise_time_s;

use opencl::hl::{CommandQueue, Context, Device};

use snowmew::common::Entity;
use snowmew::camera::Camera;
use snowmew::io::Window;
use position::Positions;

pub use config::Config;

use pipeline::Pipeline;
use drawlist::{Drawlist, create_drawlist};
use query::{ProfilerDummy, TimeQueryManager, Profiler};

use render_data::Renderable;

mod db;
mod shader;
mod vertex_buffer;
mod drawlist;
mod pipeline;
mod query;
//mod compute_accelerator;
mod config;
mod texture;
mod material;
mod light;
mod model;
mod matrix;
mod command;

enum RenderCommand {
    Update(Box<Renderable+Send>),
    Finish
}

fn render_thread(input: Receiver<(Box<Drawlist+Send>, Entity)>,
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
    unsafe {gl::Enable(gl::SCISSOR_TEST);}

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

        let (w, h) = dl.io_state().size;
        pipeline.resize(w, h);
        pipeline.render(&mut *dl, &mut db, &camera, &mut *qm);
        // if the device is a hmd we need to stall the gpu
        // to make sure it actually flipped the buffers

        if config.fps() {
            let end = precise_time_s();
            println!("total: {:4.2}ms capture: {:4.2}ms {:4.1}fps",
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
    let mut taskpool = TaskPool::new(config.thread_pool_size());

    let mut drawlists_ready = Vec::new();

    let select = std::comm::Select::new();
    let mut receiver_drawlist_ready_handle = select.handle(&receiver_drawlist_ready);
    let mut receiver_drawlist_render_handle = select.handle(&receiver_drawlist_render);
    let mut command_handle = select.handle(&command);

    unsafe {
        receiver_drawlist_ready_handle.add();
        receiver_drawlist_render_handle.add();
        command_handle.add();
    }

    let mut db = None;

    'finished: loop {
        let id = select.wait();
        if id == receiver_drawlist_ready_handle.id() {
            let dl = receiver_drawlist_ready_handle.recv();
            drawlists_ready.push(dl);
        } else if id == receiver_drawlist_render_handle.id() {
            let dl = receiver_drawlist_render_handle.recv();
            let camera = dl.camera().expect("no camera found");
            send_drawlist_setup.send((dl, camera));
        } else if id == command_handle.id() {
            let command = command_handle.recv();
            match command {
                RenderCommand::Update(rd) => db = Some(rd),
                RenderCommand::Finish => break 'finished
            }
        }

        if drawlists_ready.len() > 0 {
            if let Some(mut db) = db {
                let dl = drawlists_ready.pop().unwrap();
                let scene = db.scene().expect("no scene set");
                dl.setup_compute(&mut *db, &mut taskpool, scene, send_drawlist_render.clone());
            }
            db = None;
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
    render_done: Future<rustrt::task::Result>
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

    pub fn update(&mut self, db: Box<Renderable+Send>) {
        self.ch.send(RenderCommand::Update(db));
    }
}

impl Drop for RenderManager {
    fn drop(&mut self) {
        self.ch.send(RenderCommand::Finish);
        self.render_done.get_ref();
    }
}

impl<RD: Renderable+Send> snowmew::Render<RD> for RenderManager {
    fn update(&mut self, db: RD) {
        self.ch.send(RenderCommand::Update(box db));
    }
}

impl<RD: Renderable+Send> snowmew::RenderFactory<RD, RenderManager> for RenderFactory {
    fn init(self: Box<RenderFactory>, _: &snowmew::IOManager, window: Window, size: (i32, i32), cl: Option<Arc<Device>>) -> RenderManager {
        RenderManager::_new(window, size, cl)
    }
}

pub struct RenderFactory;

impl RenderFactory {
    pub fn new() -> RenderFactory { RenderFactory }
}