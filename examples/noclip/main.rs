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

#![crate_name = "noclip"]
#![feature(old_path, env, core)]

extern crate cgmath;
extern crate snowmew;

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

use snowmew::common::*;

use cgmath::*;
use core::Game;
use graphics::light;
use graphics::{Graphics};
use input::{integrator, InputIntegratorState};
use loader::Obj;
use position::{Positions};
use render::{Renderable, DefaultRender, Camera};

use gamedata::GameData;

mod gamedata;

fn main() {
    let sc = config::SnowmewConfig::new();

    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        println!("Please supply a path to an obj to load");
        return;
    }
    let path = Path::new(&args[1]);
    let scale: f32 = if args.len() >= 3 {
        FromStr::from_str(&args[2]).unwrap()
    } else {
        1.0
    };

    let mut db = GameData::new();
    let loader = Obj::load(&path).ok().expect("Failed to load OBJ");
    let objs = loader.import(&mut db);

    let scene = db.new_scene();
    for (_, &id) in objs.iter() {
        match db.get_draw(id) {
            Some(d) => {
                let obj = db.new_object(Some(scene.to_entity()));
                db.set_draw(obj, d.geometry, d.material);
                db.set_scale(obj, scale);
            }
            None => ()
        }
    }

    let camera_loc = db.new_object(None);
    db.set_to_identity(camera_loc);

    let pos = if args.len() >= 6 {
        let x = FromStr::from_str(&args[3]).unwrap();
        let y = FromStr::from_str(&args[4]).unwrap();
        let z = FromStr::from_str(&args[5]).unwrap();
        Point3::new(x, y, z)
    } else {
        Point3::new(0f32, 0f32, 0f32)
    };

    let head_trans = Decomposed{scale: 1f32,
                                rot:   Quaternion::identity(),
                                disp:  pos.to_vec()};
    db.set_delta(camera_loc, None, head_trans);

    let sun = light::Directional::new(Vector3::new(0.05f32, 1., 0.05),
                                      Vector3::new(1f32, 1., 1.), 1.);
    db.set_scene(scene);
    db.set_camera(camera_loc);
    db.new_light(light::Light::Directional(sun));

    let (game, gd) = integrator(Noclip, db);
    sc.start(Box::new(DefaultRender::new()), game, gd);
}

struct Noclip;

impl Game<GameData, InputIntegratorState> for Noclip {
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