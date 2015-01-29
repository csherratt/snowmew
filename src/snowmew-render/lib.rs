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
