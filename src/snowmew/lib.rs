#![crate_id = "github.com/csherratt/snowmew#snowmew:0.1"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "A game engine in rust"]

#![feature(macro_rules)]
#![feature(globs)]
#![allow(experimental)]

extern crate semver;
extern crate std;
extern crate time;
extern crate glfw;
extern crate cgmath;
extern crate cow;
extern crate sync;
extern crate native;
extern crate std;
extern crate gl;
extern crate green;
extern crate collections;
extern crate libc;
extern crate OpenCL;
extern crate ovr = "oculus-vr";

pub use common::{ObjectKey};

use sync::Arc;
use OpenCL::hl::{Device, get_platforms, GPU, CPU};


pub mod common;
pub mod camera;
pub mod io;

fn get_cl() -> Option<Arc<Device>> {
    let platforms = get_platforms();

    // find a gpu
    for platform in platforms.iter() {
        let devices = platform.get_devices_by_types(&[GPU]);
        if devices.len() != 0 {
            return Some(Arc::new(*devices.get(0)));
        } 
    }

    // use cpu if no gpu was found
    for platform in platforms.iter() {
        let devices = platform.get_devices_by_types(&[CPU, GPU]);
        if devices.len() != 0 {
            return Some(Arc::new(*devices.get(0)));
        } 
    }

    None
}

fn setup_glfw() -> glfw::Glfw {
    let glfw = glfw::init(glfw::LOG_ERRORS).ok().unwrap();

    glfw.window_hint(glfw::OpenglProfile(glfw::OpenGlCoreProfile));
    glfw.window_hint(glfw::OpenglForwardCompat(true));
    glfw.window_hint(glfw::Visible(false));
    glfw.window_hint(glfw::DepthBits(24));
    glfw.window_hint(glfw::StencilBits(8));
    glfw.window_hint(glfw::Decorated(false));

    glfw
}

pub fn start_manual_input(f: proc(&mut io::IOManager)) {
    let glfw = setup_glfw();

    let f = f;
    let mut im = io::IOManager::new(glfw);
    f(&mut im);
    println!("done");
}

pub struct DisplayConfig {
    pub resolution: Option<(u32, u32)>,
    pub position: Option<(i32, i32)>,
    pub hmd: bool,
    pub window: bool,
}

impl DisplayConfig {
    fn create_display(&self, im: &mut io::IOManager) -> Option<io::Window> {
        let window = if self.hmd { im.hmd() } else { None };
        if window.is_some() {
            return window;
        }

        let resolution = match self.resolution {
            Some(res) => res,
            None => im.get_primary_resolution()
        };

        let position = match self.position {
            Some(pos) => pos,
            None => im.get_primary_position()
        };

        if self.window {
            im.primary(resolution)
        } else {
            let win = im.window(resolution);
            match win {
                Some(win) => {
                    im.set_window_position(&win, position);
                    Some(win)
                }
                None => None
            }
        }
    }
}

pub struct SnowmewConfig {
    pub display: DisplayConfig,
    pub use_opencl: bool,
    pub cadance_ms: uint
    // render
}

impl SnowmewConfig {
    pub fn new() -> SnowmewConfig {
        SnowmewConfig {
            display: DisplayConfig {
                resolution: None,
                position: None,
                hmd: true,
                window: true,
            },
            use_opencl: true,
            cadance_ms: 8
        }
    }

    pub fn start(&self) {
        let mut im = io::IOManager::new(setup_glfw());

        // create display
        self.display.create_display(&mut im);

        let dev = if self.use_opencl { get_cl() } else { None };


    }
}