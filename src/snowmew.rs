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

#![crate_name = "snowmew"]
#![allow(unstable)]

extern crate "snowmew-core"         as _core;
extern crate "snowmew-graphics"     as _graphics;
extern crate "snowmew-loader"       as _loader;
extern crate "snowmew-position"     as _position;
extern crate "snowmew-render-mux"   as _mux;
extern crate "snowmew-render"  as _render;
extern crate "snowmew-debugger" as _debugger;
extern crate "snowmew-random" as _random;
extern crate "snowmew-timer" as _timer;
extern crate "snowmew-network" as _network;
extern crate "snowmew-input" as _input;
extern crate "snowmew-input-integrator" as _input_integrator;
extern crate opencl;
extern crate glfw;

pub use _core::table;
pub use _core::common::Entity as Entity;

pub mod render {
    pub use _mux::RenderFactory as DefaultRender;
    pub use _render::{
        RenderData,
        Renderable
    };
    pub use _render::{
        RenderFactory,
        Render
    };
    pub use _input::DisplayConfig;
    pub use _core::camera::{
        Camera
    };
}

pub mod graphics {
    pub use _graphics::{
        Drawable,
        Geometry,
        geometry,
        Graphics,
        GraphicsData,
        light,
        Light,
        material,
        Material,
        texture,
        Texture,
        VertexBuffer,
        VertexBufferIter,
    };
    pub mod vertex {
        pub use _graphics::geometry::VertexGeo as Geo;
        pub use _graphics::geometry::VertexGeoTex as GeoTex;
        pub use _graphics::geometry::VertexGeoNorm as GeoNorm;
        pub use _graphics::geometry::VertexGeoTexNorm as GeoTexNorm;
        pub use _graphics::geometry::VertexGeoTexNormTan as GeoTexNormTan;
    }
}

pub mod position {
    pub use _position::{
        MatrixManager,
        PositionData,
        Positions
    };
}

pub mod core {
    pub use _core::game::Game;
}

pub mod common {
    pub use _core::common::{
        Common,
        CommonData,
        Entity,
        Duplicate,
        Delete
    };
}

pub mod input {
    pub use _input::*;
    pub use _input_integrator::{
        InputIntegrator,
        InputIntegratorGameData,
        InputIntegratorState
    };
    pub use _input_integrator::input_integrator as integrator;
}

pub mod debug {
    pub use _debugger::{
        Debugger,
        DebuggerGameData
    };
}

pub mod random {
    pub use _random::{
        Random,
        RandomData
    };
}

pub mod loader {
    pub use _loader::Obj;
}

pub mod timer {
    pub use _timer::{Timer, Phase};
}

pub mod networking {
    pub use _network::{Server, Client};
}

pub mod config {
    use std::sync::Arc;
    use std::io::timer::Timer;
    use std::time::Duration;

    use opencl::hl::{Device, get_platforms};
    use opencl::hl::DeviceType::{GPU, CPU};
    use glfw::{self, Glfw};

    use super::input::{Event, EventGroup, DisplayConfig};
    use super::core;
    use super::render;
    use super::input;

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
        let mut glfw = glfw::init(glfw::LOG_ERRORS).ok().unwrap();

        glfw.window_hint(glfw::WindowHint::OpenglForwardCompat(true));
        glfw.window_hint(glfw::WindowHint::Visible(false));
        glfw.window_hint(glfw::WindowHint::DepthBits(24));
        glfw.window_hint(glfw::WindowHint::StencilBits(8));
        glfw.window_hint(glfw::WindowHint::Decorated(false));

        glfw
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
        pub fn start<GameData: Clone+input::GetIoState,
                     Game: core::Game<GameData, Event>,
                     R: render::Render<GameData>,
                     RF: render::RenderFactory<GameData, R>>
                     (self,
                      render: Box<RF>,
                      mut game: Game,
                      mut gd: GameData) {
            let mut im = input::IOManager::new(setup_glfw());

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
                timer_port.recv().ok().expect("failed to recv");
                im.poll();
                loop {
                    match im.next_event(&ih) {
                        EventGroup::Game(evt) => gd = game.step(evt, gd),
                        EventGroup::Window(evt) => { gd.window_action(evt) },
                        EventGroup::Nop => break
                    }
                }

                gd = game.step(Event::Cadance(candance_scale), gd);

                let next_title = gd.get_io_state().window_title.clone();
                im.set_title(&ih, next_title);
                render.update(gd.clone());
            }
        }
    }
}
