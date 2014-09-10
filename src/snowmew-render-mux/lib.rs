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

#![crate_name = "snowmew-render-mux"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "A game engine in rust"]
#![allow(dead_code)]

extern crate sync;

extern crate opencl;
extern crate "snowmew-render-gfx" as gfx;
extern crate "snowmew-core" as snowmew;
extern crate "snowmew-render" as azdo;
extern crate "snowmew-position" as position;
extern crate "snowmew-graphics" as graphics;
extern crate "snowmew-render-data" as render_data;

use std::os;

use opencl::hl::Device;
use sync::Arc;

use snowmew::common::ObjectKey;
use snowmew::io::Window;

use render_data::RenderData;

impl<'r, RD: RenderData+Send> snowmew::Render<RD> for RenderMux<'r, RD> {
    fn update(&mut self, db: RD, scene: ObjectKey, camera: ObjectKey) {
        self.render.update(db, scene, camera)
    }
}

pub struct RenderMux<'r, RD> {
    render: Box<snowmew::Render<RD> + 'r>
}

impl<'r, RD: RenderData+Send> snowmew::RenderFactory<RD, RenderMux<'r, RD>> for RenderFactory {
    fn init(self: Box<RenderFactory>,
            io: &snowmew::IOManager,
            window: Window,
            size: (i32, i32),
            cl: Option<Arc<Device>>) -> RenderMux<RD> {

        let s = match os::getenv("GFX") {
            Some(s) => s,
            None => "false".to_string()
        };
        let s: String = s.as_slice().chars().map(|c| c.to_lowercase()).collect();

        let use_gfx = match s.as_slice() {
            "true" => Some(true),
            "enabled" => Some(true),
            "1" => Some(true),
            "false" => Some(false),
            "disabled" => Some(false),
            "0" => Some(false),
            _ => None
        };

        let rm: RenderMux<RD> = if use_gfx.is_some() && use_gfx.unwrap() {
            let rf: Box<snowmew::RenderFactory<RD, gfx::RenderManager>> = box gfx::RenderFactory::new();
            let render: Box<gfx::RenderManager> = box rf.init(io, window, size, cl);
            RenderMux {
                render: render as Box<snowmew::Render<RD>>
            }
        } else {
            let rf: Box<snowmew::RenderFactory<RD, azdo::RenderManager>> = box azdo::RenderFactory::new();
            let render: Box<azdo::RenderManager> = box rf.init(io, window, size, cl);
            RenderMux {
                render: render as Box<snowmew::Render<RD>>
            }
        };
        rm
    }
}

pub struct RenderFactory;

impl RenderFactory {
    pub fn new() -> RenderFactory { RenderFactory }
}