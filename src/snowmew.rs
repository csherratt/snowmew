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
#![feature(globs)]

extern crate "snowmew-core"         as _core;
extern crate "snowmew-graphics"     as _graphics;
extern crate "snowmew-loader"       as _loader;
//extern crate "snowmew-physics"      as _physics;
extern crate "snowmew-position"     as _position;
extern crate "snowmew-render-mux"   as _render;
extern crate "snowmew-render"  as _render_data;

pub use _core::table;
pub use _core::common::Entity as Entity;

pub mod render {
    pub use _render::RenderFactory as DefaultRender;
    pub use _render_data::{
        RenderData,
        Renderable
    };
    pub use _core::{
        RenderFactory,
        Render,
    };
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
        texture_atlas,
        VertexBuffer,
        VertexBufferIter,
    };
}

pub mod position {
    pub use _position::{
        MatrixManager,
        PositionData,
        Positions
    };
}

pub mod core {
    pub use _core::{
        DisplayConfig,
        Render,
        IOManager,
    };

    pub use _core::io::{
        InputHandle,
        Window,
    };

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

pub mod config  {
    pub use _core::SnowmewConfig as SnowmewConfig;
}

pub mod input {
    pub use _core::input::*;
    pub use _core::input_integrator::{
        InputIntegrator,
        InputIntegratorGameData,
        InputIntegratorState,
    };
    pub use _core::input_integrator::input_integrator as integrator;
}

pub mod debug {
    pub use _core::debugger::{
        Debugger,
        DebuggerGameData,
        debugger
    };
}