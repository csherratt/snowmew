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

#![crate_name = "cubes"]
#![feature(macro_rules)]
#![feature(globs)]

extern crate cgmath;
extern crate opencl;
extern crate glfw;

extern crate "snowmew-core" as snowmew;
extern crate "snowmew-render-mux" as render;
extern crate "snowmew-position" as position;
extern crate "snowmew-graphics" as graphics;
extern crate "snowmew-render-data" as render_data;

use std::str::FromStr;
use std::num::Float;

use cgmath::*;

use snowmew::input;
use snowmew::input_integrator::{input_integrator, InputIntegratorState};
use snowmew::debugger::{debugger};
use snowmew::game::Game;
use snowmew::camera::Camera;
use position::Positions;
use graphics::Graphics;
use graphics::light;

use render_data::Renderable;
use render::RenderFactory;
use snowmew::common::Common;

use gamedata::GameData;

mod gamedata;

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    std::rt::start(argc, argv, main)
}

fn main() {
    let sc = snowmew::SnowmewConfig::new();

    let mut gd = GameData::new();
    let scene = gd.new_scene();

    let cube = gd.standard_graphics().shapes.cube;;
    let red = gd.standard_graphics().materials.flat.red;

    let args = std::os::args();
    let count = if args.len() >= 2 {
        FromStr::from_str(args[1].as_slice()).expect("Could not parse int")
    } else {
        10i
    };

    for x in range(-count, count) {
        for y in range(-count, count) {
            for z in range(-count, count) {
                let new = gd.new_object(Some(scene));
                let x = x as f32 * 2.5;
                let y = y as f32 * 2.5;
                let z = z as f32 * 2.5;
                gd.set_scale(new, 0.25);
                gd.set_displacement(new, Vector3::new(x, y, z));
                gd.set_draw(new, cube, red);
            }
        }
    }

    let sun = light::Directional::new(Vector3::new(0.75f32, 1., 0.7),
                                      Vector3::new(1f32, 1., 1.), 0.25);
    gd.new_light(light::Light::Directional(sun));

    let camera = gd.new_object(None);
    gd.set_to_identity(camera);

    gd.set_scene(scene);
    gd.set_camera(camera);

    let (game, gd) = input_integrator(Cubes, gd);
    let (game, gd) = debugger(game, gd);
    sc.start(box RenderFactory::new(), game, gd);
}

struct Cubes;

impl Game<GameData, InputIntegratorState> for Cubes {
    fn step(&mut self, state: InputIntegratorState, gd: GameData) -> GameData {
        let mut next = gd.clone();

        let camera_key = gd.camera().expect("no camera set");
        let camera = Camera::new(next.position(camera_key));
        let (mut rx, ry, mut rz) = next.get_rotation(camera_key).expect("no rot").to_euler();

        let (x, y) = state.mouse_delta();
        rx = rx.add_a(rad((-x / 120.) as f32));
        rz = rz.add_a(rad((-y / 120.) as f32));

        let max_rot: f32 = Float::frac_pi_2();
        if rz.s > max_rot {
            rz.s = max_rot;
        } else if rz.s < -max_rot {
            rz.s = -max_rot;
        }

        let input_vec = Vector3::new(
            if state.button_down(input::Button::KeyboardA) {0.05f32} else {0f32} +
            if state.button_down(input::Button::KeyboardD) {-0.05f32} else {0f32},
            0f32,
            if state.button_down(input::Button::KeyboardW) {0.05f32} else {0f32} +
            if state.button_down(input::Button::KeyboardS) {-0.05f32} else {0f32}
        ).mul_s(-1f32);

        let head_trans = Decomposed{scale: 1f32,
                                    rot:   Rotation3::from_euler(rx, ry, rz),
                                    disp:  camera.move_with_vector(&input_vec).to_vec()};

        next.set_delta(camera_key, None, head_trans);

        next
    }
}