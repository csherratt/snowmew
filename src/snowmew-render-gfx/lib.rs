#![crate_id = "github.com/csherratt/snowmew#snowmew-render-gfx:0.1"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "A game engine in rust"]
#![allow(dead_code)]

//extern crate debug;
extern crate std;
extern crate glfw;
extern crate gfx;
extern crate snowmew;
extern crate OpenCL;
extern crate sync;
extern crate position = "snowmew-position";
extern crate graphics = "snowmew-graphics";

use std::task;
use std::rt;
use std::comm::{Receiver, Sender};
use std::mem;
use std::sync::TaskPool;
use std::sync::Future;

use OpenCL::hl::{CommandQueue, Context, Device};
use sync::Arc;

use position::Positions;
use graphics::Graphics;
use snowmew::common::ObjectKey;
use snowmew::io::Window;


pub trait RenderData : Graphics + Positions {}

enum RenderCommand {
    Update(Box<RenderData+Send>, ObjectKey, ObjectKey),
    Finish
}

pub struct RenderManager;

impl RenderManager {
    fn _new(window: Window, size: (i32, i32), dev: Option<Arc<Device>>) -> RenderManager {
        fail!("not supported")
    }
}


impl<RD: RenderData+Send> snowmew::Render<RD> for RenderManager {
    fn update(&mut self, db: RD, scene: ObjectKey, camera: ObjectKey) {
        fail!("not supported")
    }
}

impl<RD: RenderData+Send> snowmew::RenderFactory<RD, RenderManager> for RenderFactory {
    fn init(self, window: Window, size: (i32, i32), cl: Option<Arc<Device>>) -> RenderManager {
        RenderManager::_new(window, size, cl)
    }
}

pub struct RenderFactory;

impl RenderFactory {
    pub fn new() -> RenderFactory { RenderFactory }
}