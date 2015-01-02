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

#![crate_name = "gears"]
#![feature(macro_rules)]
#![feature(globs)]

extern crate glfw;
extern crate gl;
extern crate "snowmew-core" as snowmew;
extern crate "snowmew-render-mux" as render;
extern crate "snowmew-loader" as loader;
extern crate "snowmew-position" as position;
extern crate "snowmew-graphics" as graphics;
extern crate "snowmew-debugger" as debugger;
extern crate cgmath;
extern crate opencl;
extern crate "rustc-serialize" as rustc_serialize;
extern crate "snowmew-render" as render_data;

use cgmath::*;
use debugger::{Debugger, DebuggerGameData};
use graphics::light;
use graphics::{Graphics, GraphicsData};
use loader::Obj;
use position::{Positions, PositionData};
use render::RenderFactory;
use render_data::{Renderable, RenderData};
use snowmew::common::{Common, CommonData};
use snowmew::game::Game;
use snowmew::input_integrator::{input_integrator, InputIntegratorState};

use gamedata::GameData;

mod gamedata;

fn main() {
    let sc = snowmew::SnowmewConfig::new();

    let mut gd = GameData::new();
    let loader = Obj::load(&Path::new("assets/rust_logo.obj")).ok().expect("Failed to load OBJ");
    let obj = loader.import(&mut gd);

    let scene = gd.new_scene();
    let &logo = obj.get(&"rust_logo".to_string()).expect("geometry not found from import");
    let logo_draw = gd.get_draw(logo).expect("Could not get draw binding");

    let scene_logos = vec!((gd.new_object(Some(scene)), gd.standard_graphics().materials.flat.green),
                           (gd.new_object(Some(scene)), gd.standard_graphics().materials.flat.blue),
                           (gd.new_object(Some(scene)), gd.standard_graphics().materials.flat.red));

    for (idx, &(logo, material)) in scene_logos.iter().enumerate() {
        gd.set_draw(logo, logo_draw.geometry, material);
        gd.set_scale(logo, 1.36);
        gd.set_displacement(logo, Vector3::new((idx as f32 - 1.) * 10., 0f32, 0f32));
        gd.set_rotation(logo, Rotation3::from_euler(rad(0f32),
                                                    deg(90f32).to_rad(),
                                                    deg(90f32).to_rad()));
        gd.gears.push(logo);
    }

    let camera_loc = gd.new_object(None);

    gd.set_delta(camera_loc, None, Decomposed{scale: 1f32,
                                              rot:   Rotation::identity(),
                                              disp:  Vector3::new(0f32, 0f32, 15f32)});

    let sun = light::Directional::new(Vector3::new(0.5f32, 1., 0.5),
                                      Vector3::new(1f32, 1., 1.), 0.25);

    gd.new_light(light::Light::Directional(sun));
    gd.set_scene(scene);
    gd.set_camera(camera_loc);

    let game = GearsInput {
        debugger: Debugger::new(Gears)
    };
    let gd = GearsInputData {
        paused: false,
        inner: DebuggerGameData::new(gd, 100)
    };
    let (game, gd) = input_integrator(game, gd);
    sc.start(box RenderFactory::new(), game, gd);
}

struct Gears;

impl Game<GameData, InputIntegratorState> for Gears {
    fn step(&mut self, state: InputIntegratorState, gd: GameData) -> GameData {
        let mut next = gd.clone();

        for (idx, &logo) in gd.gears.iter().enumerate() {
            let t = state.time() as f32 * 10.;
            let this_gear_rot = if idx % 2 == 0 { t } else { 5.625 - t };
            next.set_rotation(logo, Rotation3::from_euler(deg(0f32).to_rad(),
                                                          deg(this_gear_rot).to_rad(),
                                                          deg(90f32).to_rad()));
        }

        next
    }
}

struct GearsInput {
    debugger: Debugger<Gears>    
}

impl Game<GearsInputData, InputIntegratorState> for GearsInput {
    fn step(&mut self, state: InputIntegratorState, mut gd: GearsInputData) -> GearsInputData {
        if state.button_pressed(snowmew::input::Button::KeyboardSpace) {
            gd.paused ^= true;
        }

        if !gd.paused {
            gd.inner = self.debugger.step(state, gd.inner);
        } else {
            let (_, x) = state.scroll_delta();
            if x < 0. {
                gd.inner = self.debugger.skip_backward(gd.inner);
            } else if x > 0. {
                gd.inner = self.debugger.skip_forward(gd.inner);
            }
        }
        gd
    }
}

#[deriving(Clone)]
struct GearsInputData {
    paused: bool,
    inner: DebuggerGameData<GameData, InputIntegratorState>
}

impl Common for GearsInputData {
    fn get_common<'a>(&'a self) -> &'a CommonData { self.inner.get_common() }
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData { self.inner.get_common_mut() }
}

impl Positions for GearsInputData {
    fn get_position<'a>(&'a self) -> &'a PositionData { self.inner.get_position() }
    fn get_position_mut<'a>(&'a mut self) -> &'a mut PositionData { self.inner.get_position_mut() }
}

impl Graphics for GearsInputData {
    fn get_graphics<'a>(&'a self) -> &'a GraphicsData { self.inner.get_graphics() }
    fn get_graphics_mut<'a>(&'a mut self) -> &'a mut GraphicsData { self.inner.get_graphics_mut() }
}

impl Renderable for GearsInputData {
    fn get_render_data<'a>(&'a self) -> &'a RenderData { self.inner.get_render_data() }
    fn get_render_data_mut<'a>(&'a mut self) -> &'a mut RenderData { self.inner.get_render_data_mut() }
}

