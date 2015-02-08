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
#![feature(os)]
#![feature(core)]


extern crate cgmath;
extern crate snowmew;
extern crate "rustc-serialize" as rustc_serialize;

pub use snowmew::{
    core,
    render,
    loader,
    graphics,
    position,
    input,
    config,
    debug
};

use std::str::FromStr;
use std::f32;

use cgmath::*;
use core::Game;
use graphics::light;
use graphics::{Graphics};
use input::{integrator, InputIntegratorState};
use position::{Positions};
use render::{Renderable, DefaultRender, Camera};
use snowmew::common::{Common};

use gamedata::GameData;
mod gamedata;

fn main() {
    let sc = config::SnowmewConfig::new();

    let mut gd = GameData::new();
    let scene = gd.new_scene();

    let cube = gd.standard_graphics().shapes.cube;;
    let red = gd.standard_graphics().materials.flat.red;

    let args = std::os::args();
    let count: i32 = if args.len() >= 2 {
        FromStr::from_str(&args[1][]).unwrap()
    } else {
        10i32
    };

    for x in (-count..count) {
        for y in (-count..count) {
            for z in (-count..count) {
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

    let (game, gd) = integrator(Cubes, gd);
    sc.start(Box::new(DefaultRender::new()), game, gd);
}

struct Cubes;

impl Game<GameData, InputIntegratorState> for Cubes {
    fn step(&mut self, state: InputIntegratorState, gd: GameData) -> GameData {
        let mut next = gd.clone();

        //let (w, h) = gd.io_state().size;
        let (w, h) = (800, 600);

        let camera_key = gd.camera().expect("no camera set");
        let camera = Camera::new(w, h, next.position(camera_key));
        let (mut rx, ry, mut rz) = next.get_rotation(camera_key).expect("no rot").to_euler();

        let (x, y) = state.mouse_delta();
        rx = rx.add_a(rad((-x / 120.) as f32));
        rz = rz.add_a(rad((-y / 120.) as f32));

        let max_rot: f32 = f32::consts::FRAC_PI_2;
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