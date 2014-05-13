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
use std::task::{TaskResult, TaskBuilder};
use std::comm::{Receiver, Sender};
use time::precise_time_s;

use cgmath::matrix::{Matrix, ToMatrix4, Matrix4};
use cgmath::ptr::Ptr;

use OpenCL::hl::{CommandQueue, Context, Device};
use sync::{TaskPool, Arc};

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
mod pipeline;
mod query;
mod compute_accelerator;
mod config;

pub trait RenderData : Graphics + Positions {}

enum RenderCommand {
    Update(Box<RenderData:Send>, ObjectKey, ObjectKey),
    Finish
}

fn swap_buffers_sync(gls: &db::GlState, disp: &mut Window) {
    disp.swap_buffers();
    unsafe {
        let mat: Matrix4<f32> = Matrix4::identity();
        let shader = gls.flat_shader.expect("shader not found");
        shader.bind();
        gl::UniformMatrix4fv(shader.uniform("mat_proj_view"), 1, gl::FALSE, mat.ptr());
        gl::DrawElements(gl::TRIANGLES, 3i32, gl::UNSIGNED_INT, ptr::null());
        let sync = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
        gl::ClientWaitSync(sync, gl::SYNC_FLUSH_COMMANDS_BIT, 1_000_000_000u64);
        gl::DeleteSync(sync);
    }
}

fn render_thread(input: Receiver<(DrawlistStandard, ObjectKey)>,
                 output: Sender<DrawlistStandard>,
                 mut window: Window,
                 mut db: db::GlState,
                 size: (i32, i32),
                 config: Config,
                 cl: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) {

    window.make_context_current();

    let mut pipeline = {
        let (width, height) = size;
        if !window.is_hmd() {
            box pipeline::Defered::new(pipeline::Forward::new(), width as uint, height as uint) as Box<Pipeline>
        } else {
            box pipeline::Hmd::new(
                pipeline::Defered::new(pipeline::Forward::new(), width as uint, height as uint),
                config.hmd_size(),
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

    for _ in range(0, 3) {
        let mut dl = DrawlistStandard::from_config(&config, cl.clone());
        dl.setup_begin();
        output.send(dl);
    }

    for (mut dl, camera) in input.iter() {
        dl.setup_complete(&mut db, &config);

        let capture = precise_time_s();
        let camera_trans = dl.position(camera);
        let camera = Camera::new(if window.is_hmd() {
            let sf = window.sensor_fusion();
            let rift = sf.get_predicted_orientation(None);
            camera_trans.mul_m(&rift.to_matrix4())
        } else {
            camera_trans
        });

        let (x, y) = size;
        let dt = DrawTarget::new(0, (0, 0), (x as uint, y as uint), ~[gl::BACK_LEFT]);
        pipeline.render(&mut dl, &mut db, &camera.get_matrices(size), &dt);
        // if the device is a hmd we need to stall the gpu
        // to make sure it actually flipped the buffers
        if window.is_hmd() || true {
            swap_buffers_sync(&db, &mut window);
        } else {
            window.swap_buffers();
        }

        let end = precise_time_s();
        println!("total: {:4.2f}ms capture: {:4.2f}ms", (end - dl.start_time()) * 1000., (end - capture) * 1000.);

        dl.setup_begin();
        output.send(dl);
    }
}

fn render_server(command: Receiver<RenderCommand>,
                 mut db: Box<RenderData:Send>,
                 window: Window,
                 size: (i32, i32),
                 cl: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) {

    let gl = db::GlState::new();
    let mut scene = 0;
    let mut camera = 0;
    let config = Config::new(window.get_context_version());

    let mut taskbuilder = TaskBuilder::new();
    taskbuilder = taskbuilder.named("render-thread".into_maybe_owned());

    let (send_drawlist_setup, receiver_drawlist_setup) = channel();
    let (send_drawlist_ready, receiver_drawlist_ready) = channel();
    taskbuilder.spawn(proc() {
        let window = window;
        render_thread(receiver_drawlist_setup,
                      send_drawlist_ready,
                      window,
                      gl,
                      size,
                      config,
                      cl
        );
    });

    let (send_drawlist_render, receiver_drawlist_render)
        : (Sender<DrawlistStandard>, Receiver<DrawlistStandard>) = channel();
    let mut taskpool = TaskPool::new(8, || { 
        let ch = send_drawlist_render.clone();
        proc(_: uint) { ch.clone() }
    });

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
            dl.setup_compute(db, &mut taskpool);
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
    render_done: Receiver<TaskResult>
}

impl RenderManager {
    fn _new(db: Box<RenderData:Send>, window: Window, size: (i32, i32), dev: Option<Arc<Device>>) -> RenderManager {
        let cl = setup_opencl(&window, dev);

        let mut taskbuilder = TaskBuilder::new();
        taskbuilder = taskbuilder.named("render-server".into_maybe_owned());
        let render_main_result = taskbuilder.future_result();

        let (sender, receiver) = channel();
        taskbuilder.spawn(proc() {
            let db = db;
            let window = window;

            render_server(receiver, db, window, size, cl);
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