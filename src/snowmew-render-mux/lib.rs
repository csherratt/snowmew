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

extern crate opencl;
extern crate "snowmew-render-gfx" as gfx;
extern crate "snowmew-core" as snowmew;
extern crate "snowmew-position" as position;
extern crate "snowmew-graphics" as graphics;
extern crate "snowmew-render" as render_data;

use opencl::hl::Device;
use std::sync::Arc;

use snowmew::io::Window;

use render_data::Renderable;

impl<'r, RD: Renderable+Send> snowmew::Render<RD> for RenderMux<'r, RD> {
    fn update(&mut self, db: RD) {
        self.render.update(db)
    }
}

pub struct RenderMux<'r, RD> {
    render: Box<snowmew::Render<RD> + 'r>
}

impl<'r, RD: Renderable+Send> snowmew::RenderFactory<RD, RenderMux<'r, RD>> for RenderFactory {
    fn init(self: Box<RenderFactory>,
            io: &snowmew::IOManager,
            window: Window,
            size: (i32, i32),
            cl: Option<Arc<Device>>) -> RenderMux<'r, RD> {

        let rm: RenderMux<RD> = {
            let rf: Box<snowmew::RenderFactory<RD, gfx::RenderManager<RD>>> = box gfx::RenderFactory::new();
            let render: Box<gfx::RenderManager<RD>> = box rf.init(io, window, size, cl);
            RenderMux {
                render: render as Box<snowmew::Render<RD>>
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