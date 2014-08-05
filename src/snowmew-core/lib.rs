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

#![crate_name = "snowmew-core"]
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
extern crate opencl;
extern crate device;
extern crate ovr;

pub use common::{ObjectKey};
pub use io::IOManager;

use sync::Arc;
use opencl::hl::{Device, get_platforms, GPU, CPU};
use std::io::timer::Timer;

pub mod common;
pub mod camera;
pub mod io;

fn get_cl() -> Option<Arc<Device>> {
    let platforms = get_platforms();

    // find a gpu
    for platform in platforms.iter() {
        let devices = platform.get_devices_by_types(&[GPU]);
        if devices.len() != 0 {
            return Some(Arc::new(devices[0]));
        } 
    }

    // use cpu if no gpu was found
    for platform in platforms.iter() {
        let devices = platform.get_devices_by_types(&[CPU, GPU]);
        if devices.len() != 0 {
            return Some(Arc::new(devices[0]));
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

        if !self.window {
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

pub trait Render<T> {
    fn update(&mut self, db: T, scene: ObjectKey, camera: ObjectKey);
}

pub trait RenderFactory<T, R: Render<T>> {
    fn init(self: Box<Self>, im: &IOManager, window: io::Window, size: (i32, i32), cl: Option<Arc<Device>>) -> R;
}

pub struct SnowmewConfig<GD, R> {
    pub display: DisplayConfig,
    pub use_opencl: bool,
    pub cadance_ms: u64,
    pub render: Option<Box<R>>
}

impl<GD: Clone, R: Render<GD>, RF: RenderFactory<GD, R>> SnowmewConfig<GD, RF> {
    pub fn new() -> SnowmewConfig<GD, RF> {
        SnowmewConfig {
            display: DisplayConfig {
                resolution: None,
                position: None,
                hmd: true,
                window: true,
            },
            use_opencl: true,
            cadance_ms: 8,
            render: None
        }
    }

    pub fn start(self, gd: GD, game: |GD, &io::InputState, &io::InputState| -> (GD, ObjectKey, ObjectKey)) {
        let mut gd = gd;
        let mut im = io::IOManager::new(setup_glfw());

        // create display
        let display = match self.display.create_display(&mut im) {
            None => return,
            Some(display) => display
        };
        let ih = display.handle();

        let res = im.get_framebuffer_size(&display);
        let dev = if self.use_opencl { get_cl() } else { None };
        let mut render = self.render.unwrap().init(&im, display, res, dev);

        let mut timer = Timer::new().unwrap();
        let timer_port = timer.periodic(self.cadance_ms);

        let mut input_last = im.get(&ih);
        while !input_last.should_close() {
            timer_port.recv();
            im.poll();
            let input = im.get(&ih);
            let (new_gd, scene, camera) = game(gd, &input, &input_last);
            render.update(new_gd.clone(), scene, camera);
            gd = new_gd;
            input_last = input;
        }
    }
}

