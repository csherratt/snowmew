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

#![crate_name = "snowmew-render-data"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "A game engine in rust"]
#![allow(dead_code)]

extern crate serialize;
extern crate "snowmew-core" as core;
extern crate "snowmew-position" as position;
extern crate "snowmew-graphics" as graphics;

use serialize::Encodable;

#[deriving(Clone, Encodable, Decodable)]
pub struct RenderData {
    camera: Option<core::ObjectKey>,
    scene: Option<core::ObjectKey>
}

impl RenderData {
    pub fn new() -> RenderData {
        RenderData {
            camera: None,
            scene: None
        }
    }
}

impl<'b, C: Renderable> Renderable for &'b DerefMut<C> + 'b {
    fn get_render_data<'a>(&'a self) -> &'a RenderData { self.get_render_data() }
    fn get_render_data_mut<'a>(&'a mut self) -> &'a mut RenderData { self.get_render_data_mut() }
}

pub trait Renderable: graphics::Graphics + position::Positions {
    fn get_render_data(&self) -> &RenderData;
    fn get_render_data_mut(&mut self) -> &mut RenderData;

    /// set the camera for the render
    fn set_camera(&mut self, camera: core::ObjectKey) {
        self.get_render_data_mut().camera = Some(camera);
    }

    /// set the scene to be rendered
    fn set_scene(&mut self, scene: core::ObjectKey) {
        self.get_render_data_mut().scene = Some(scene);
    }

    /// get the camera for rendering
    fn camera(&self) -> Option<core::ObjectKey> {
        self.get_render_data().camera
    }

    /// get the scene for rendering
    fn scene(&self) -> Option<core::ObjectKey> {
        self.get_render_data().scene
    }
}