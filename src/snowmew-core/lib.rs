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
#![crate_type = "lib"]

#![feature(macro_rules)]
#![feature(globs)]
#![feature(associated_types)]
#![feature(old_orphan_check)]
#![allow(experimental)]

extern crate time;
extern crate glfw;
extern crate cgmath;
extern crate cow;
extern crate "rustc-serialize" as rustc_serialize;
extern crate nice_glfw;
extern crate collections;
extern crate libc;
extern crate opencl;
extern crate device;
extern crate ovr;
extern crate collect;

pub use common::{Entity};
pub use io::IOManager;

use std::sync::Arc;
use opencl::hl::{Device, get_platforms};
use opencl::hl::DeviceType::{GPU, CPU};
use std::io::timer::Timer;
use std::time::Duration;
use common::Common;

/// contains the common data for the Entity manager
pub mod common;
/// contains a few different formats that can be used
/// to represent a table in snowmew
pub mod table;
/// contains utility functions for managing a camera
pub mod camera;
/// used for the io manager
pub mod io;
/// contains the `Game` trait
pub mod game;
/// contains the input data that is applied to the `Game`
pub mod input;
/// used to convert actions into state that can be tracked
pub mod input_integrator;

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

    glfw.window_hint(glfw::WindowHint::OpenglForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::Visible(false));
    glfw.window_hint(glfw::WindowHint::DepthBits(24));
    glfw.window_hint(glfw::WindowHint::StencilBits(8));
    glfw.window_hint(glfw::WindowHint::Decorated(false));

    glfw
}

#[derive(Copy)]
/// Used to configure how a window should be created for the game
pub struct DisplayConfig {
    /// The resolution in pixels (width, height)
    /// if not set the engine will do a best guess
    pub resolution: Option<(u32, u32)>,
    /// The position of the window, if not set the window
    /// will be placed at the best guess for the engine
    pub position: Option<(i32, i32)>,
    /// Enable HMD for Oculus Rift support, Only supported by the AZDO backend
    pub hmd: bool,
    /// Should the window be created as a window instead of fullscreen.
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

/// Render is a trait that describes the describes how a render is implemented
/// in the engine. `update` is called once per cadence pulse.
pub trait Render<T> {
    fn update(&mut self, db: T);
}

/// RenderFactor is used to create a `Render` object. This is used to pass a configured
/// Window to the Render.
pub trait RenderFactory<T, R: Render<T>> {
    fn init(self: Box<Self>,
            im: &IOManager,
            window: io::Window,
            size: (i32, i32),
            cl: Option<Arc<Device>>) -> R;
}

#[derive(Copy)]
/// Used to configure the engine prior to the game stating.
pub struct SnowmewConfig {
    /// The display configuration
    pub display: DisplayConfig,
    /// Configure if the engine should use OpenCL
    pub use_opencl: bool,
    /// Configure the cadence, the minimum peroid for a frame update
    pub cadance_ms: i64
}

impl SnowmewConfig {
    /// Create a new configuration with sane defaults
    pub fn new() -> SnowmewConfig {
        SnowmewConfig {
            display: DisplayConfig {
                resolution: None,
                position: None,
                hmd: true,
                window: true,
            },
            use_opencl: true,
            cadance_ms: 15
        }
    }

    /// Start the game engine running based on the confirmation.
    pub fn start<GameData: Common+Clone,
                 Game: game::Game<GameData, input::Event>,
                 R: Render<GameData>,
                 RF: RenderFactory<GameData, R>>
                 (self,
                  render: Box<RF>,
                  mut game: Game,
                  mut gd: GameData) {
        let mut im = io::IOManager::new(setup_glfw());

        // create display
        let display = match self.display.create_display(&mut im) {
            None => return,
            Some(display) => display
        };
        let ih = display.handle();

        let res = im.get_framebuffer_size(&display);
        let dev = if self.use_opencl { get_cl() } else { None };
        let mut render = render.init(&im, display, res, dev);

        let mut timer = Timer::new().unwrap();
        let timer_port = timer.periodic(Duration::milliseconds(self.cadance_ms));
        let candance_scale = self.cadance_ms as f64 / 1000.;

        while !im.should_close(&ih) {
            timer_port.recv();
            im.poll();
            loop {
                match im.next_event(&ih) {
                    input::EventGroup::Game(evt) => gd = game.step(evt, gd),
                    input::EventGroup::Window(evt) => gd.window_action(evt),
                    input::EventGroup::Nop => break
                }
            }

            gd = game.step(input::Event::Cadance(candance_scale), gd);

            let next_title = gd.io_state().window_title.clone();
            im.set_title(&ih, next_title);
            render.update(gd.clone());
        }
    }
}

