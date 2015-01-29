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
#![crate_type = "lib"]
#![allow(unstable)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate cgmath;
extern crate ovr;

extern crate "snowmew-core" as snowmew;
extern crate "snowmew-position" as position;
extern crate "snowmew-graphics" as graphics;
extern crate "snowmew-input" as input;

#[cfg(feature="use_opencl")]
extern crate opencl;

#[cfg(feature="use_opencl")]
use std::sync::Arc;

/// contains utility functions for managing a camera
pub mod camera;

#[derive(Clone, RustcEncodable, RustcDecodable, Copy)]
pub struct RenderData {
    camera: Option<snowmew::Entity>,
    scene: Option<snowmew::Entity>
}

impl RenderData {
    pub fn new() -> RenderData {
        RenderData {
            camera: None,
            scene: None
        }
    }
}

pub trait Renderable: graphics::Graphics + position::Positions {
    fn get_render_data(&self) -> &RenderData;
    fn get_render_data_mut(&mut self) -> &mut RenderData;

    /// set the camera for the render
    fn set_camera(&mut self, camera: snowmew::Entity) {
        self.get_render_data_mut().camera = Some(camera);
    }

    /// set the scene to be rendered
    fn set_scene(&mut self, scene: snowmew::Entity) {
        self.get_render_data_mut().scene = Some(scene);
    }

    /// get the camera for rendering
    fn camera(&self) -> Option<snowmew::Entity> {
        self.get_render_data().camera
    }

    /// get the scene for rendering
    fn scene(&self) -> Option<snowmew::Entity> {
        self.get_render_data().scene
    }
}

/// Render is a trait that describes the describes how a render is implemented
/// in the engine. `update` is called once per cadence pulse.
pub trait Render<T> {
    fn update(&mut self, db: T);
}

/// RenderFactor is used to create a `Render` object. This is used to pass a configured
/// Window to the Render.
#[cfg(feature="use_opencl")]
pub trait RenderFactory<T, R: Render<T>> {
    fn init(self: Box<Self>,
            im: &input::IOManager,
            window: input::Window,
            size: (i32, i32),
            cl: Option<Arc<opencl::hl::Device>>) -> R;
}

#[cfg(not(feature="use_opencl"))]
pub trait RenderFactory<T, R: Render<T>> {
    fn init(self: Box<Self>,
            im: &input::IOManager,
            window: input::Window,
            size: (i32, i32)) -> R;
}

/// Convert your game into
pub trait IntoRender {
    /// The output render form of your gamestate
    type RenderGameState;

    fn into_render(self) -> Self::RenderGameState;
}

impl<T> IntoRender for T
    where T: position::Positions + graphics::Graphics + Renderable +
             input::GetIoState + snowmew::common::Common {
    
    type RenderGameState = BasicRenderData;

    fn into_render(self) -> BasicRenderData {
        BasicRenderData {
            common: self.get_common().clone(),
            graphics: self.get_graphics().clone(),
            render_data: self.get_render_data().clone(),
            io_state: self.get_io_state().clone(),
            position: self.get_position().clone()
        }
    }

}

#[derive(Clone)]
struct BasicRenderData {
    common: snowmew::common::CommonData,
    graphics: graphics::GraphicsData,
    position: position::PositionData,
    io_state: input::IoState,
    render_data: RenderData
}

impl position::Positions for BasicRenderData {
    fn get_position<'a>(&'a self) -> &'a position::PositionData { &self.position }
    fn get_position_mut<'a>(&'a mut self) -> &'a mut position::PositionData { &mut self.position }
}

impl graphics::Graphics for BasicRenderData {
    fn get_graphics<'a>(&'a self) -> &'a graphics::GraphicsData { &self.graphics }
    fn get_graphics_mut<'a>(&'a mut self) -> &'a mut graphics::GraphicsData { &mut self.graphics }
}

impl Renderable for BasicRenderData {
    fn get_render_data<'a>(&'a self) -> &'a RenderData { &self.render_data }
    fn get_render_data_mut<'a>(&'a mut self) -> &'a mut RenderData { &mut self.render_data }
}

impl input::GetIoState for BasicRenderData {
    fn get_io_state<'a>(&'a self) -> &'a input::IoState { &self.io_state }
    fn get_io_state_mut<'a>(&'a mut self) -> &'a mut input::IoState { &mut self.io_state }
}

impl snowmew::common::Common for BasicRenderData {
    fn get_common<'a>(&'a self) -> &'a snowmew::common::CommonData { &self.common }
    fn get_common_mut<'a>(&'a mut self) -> &'a mut snowmew::common::CommonData { &mut self.common }
}