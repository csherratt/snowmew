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
#![crate_type = "lib"]


#[cfg(feature="use_opencl")]
extern crate opencl;
extern crate "snowmew-render-gfx" as gfx;
extern crate "snowmew-core" as snowmew;
extern crate "snowmew-position" as position;
extern crate "snowmew-graphics" as graphics;
extern crate "snowmew-render" as render;
extern crate "snowmew-input" as input;

#[cfg(feature="use_opencl")]
use opencl::hl::Device;
#[cfg(feature="use_opencl")]
use std::sync::Arc;

use input::{Window, GetIoState};

use render::Renderable;

impl<'r, RD: Renderable+GetIoState+Send+> render::Render<RD> for RenderMux<'r, RD> {
    fn update(&mut self, db: RD) {
        self.render.update(db)
    }
}

pub struct RenderMux<'r, RD> {
    render: Box<render::Render<RD> + 'r>
}

#[cfg(feature="use_opencl")]
impl<'r, RD: Renderable+GetIoState+Send+'static> render::RenderFactory<RD, RenderMux<'r, RD>> for RenderFactory {
    fn init(self: Box<RenderFactory>,
            io: &input::IOManager,
            window: Window,
            size: (i32, i32),
            cl: Option<Arc<Device>>) -> RenderMux<'r, RD> {

        let rm: RenderMux<RD> = {
            let rf: Box<render::RenderFactory<RD, gfx::RenderManager<RD>>> = Box::new(gfx::RenderFactory::new());
            let render: Box<gfx::RenderManager<RD>> = Box::new(rf.init(io, window, size, cl));
            RenderMux {
                render: render as Box<render::Render<RD>>
            }
        };
        rm
    }
}

#[cfg(not(feature="use_opencl"))]
impl<'r, RD: Renderable+GetIoState+Send+'static> render::RenderFactory<RD, RenderMux<'r, RD>> for RenderFactory {
    fn init(self: Box<RenderFactory>,
            io: &input::IOManager,
            window: Window,
            size: (i32, i32)) -> RenderMux<'r, RD> {

        let rm: RenderMux<RD> = {
            let rf: Box<render::RenderFactory<RD, gfx::RenderManager<RD>>> = Box::new(gfx::RenderFactory::new());
            let render: Box<gfx::RenderManager<RD>> = Box::new(rf.init(io, window, size));
            RenderMux {
                render: render as Box<render::Render<RD>>
            }
        };
        rm
    }
}


#[derive(Copy)]
pub struct RenderFactory;

impl RenderFactory {
    pub fn new() -> RenderFactory { RenderFactory }
}