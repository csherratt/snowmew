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
#![feature(macro_rules)]
#![feature(globs)]

extern crate glfw;
extern crate gl;
extern crate "snowmew-core" as snowmew;
extern crate "snowmew-render-mux" as render;
extern crate "snowmew-loader" as loader;
extern crate "snowmew-position" as position;
extern crate "snowmew-graphics" as graphics;
extern crate cgmath;
extern crate native;
extern crate "snowmew-render-data" as render_data;

use std::from_str::FromStr;

use cgmath::*;

use snowmew::input;
use snowmew::debugger::debugger;
use snowmew::input_integrator::{input_integrator, InputIntegratorState};
use snowmew::game::Game;
use snowmew::camera::Camera;
use position::Positions;
use graphics::Graphics;
use graphics::light;

use render_data::Renderable;
use render::RenderFactory;
use loader::Obj;
use snowmew::common::Common;

use gamedata::GameData;

mod gamedata;

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    let sc = snowmew::SnowmewConfig::new();

    let args = std::os::args();
    if args.len() == 1 {
        println!("Please supply a path to an obj to load");
        return;
    }
    let path = Path::new(args[1].as_slice());
    let scale: f32 = if args.len() >= 3 {
        match FromStr::from_str(args[2].as_slice()) {
            Some(v) => v,
            None => 1.0
        }
    } else {
        1.0
    };

    let mut db = GameData::new();
    let loader = Obj::load(&path).ok().expect("Failed to load OBJ");
    let import = db.new_object(None, "import");
    loader.import(import, &mut db);
    let scene = db.new_scene("scene");
    let geo_dir = db.find("import/objects").expect("geometry not found from import");
    for (name, id) in db.clone().walk_dir(geo_dir) {
        match db.get_draw(id) {
            Some(d) => {
                let obj = db.new_object(Some(scene), name);
                db.set_draw(obj, d.geometry, d.material);
                db.set_scale(obj, scale);
            }
            None => ()
        }
    }

    let camera_loc = db.new_object(None, "camera");
    db.set_to_identity(camera_loc);

    let pos = if args.len() >= 6 {
        let x = FromStr::from_str(args[3].as_slice());
        let y = FromStr::from_str(args[4].as_slice());
        let z = FromStr::from_str(args[5].as_slice());
        match (x, y, z) {
            (Some(x), Some(y), Some(z)) => {
                Point3::new(x, y, z)
            }
            _ => Point3::new(0f32, 0., 0.)
        }
    } else {
        Point3::new(0f32, 0f32, 0f32)
    };

    let head_trans = Decomposed{scale: 1f32,
                                rot:   Quaternion::identity(),
                                disp:  pos.to_vec()};
    db.update_location(camera_loc, head_trans);

    let sun = light::Directional::new(Vector3::new(0.05f32, 1., 0.05),
                                      Vector3::new(1f32, 1., 1.), 1.);
    db.set_scene(scene);
    db.set_camera(camera_loc);
    db.new_light(scene, "sun", light::DirectionalLight(sun));

    let (game, gd) = input_integrator(Noclip, db);
    let (game, gd) = debugger(game, gd);
    sc.start(box RenderFactory::new(), game, gd);
}

struct Noclip;

impl Game<GameData, InputIntegratorState> for Noclip {
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
            if state.button_down(input::KeyboardA) {0.05f32} else {0f32} +
            if state.button_down(input::KeyboardD) {-0.05f32} else {0f32}, 
            0f32,
            if state.button_down(input::KeyboardW) {0.05f32} else {0f32} +
            if state.button_down(input::KeyboardS) {-0.05f32} else {0f32}
        ).mul_s(-1f32);

        let head_trans = Decomposed{scale: 1f32,
                                    rot:   Rotation3::from_euler(rx, ry, rz),
                                    disp:  camera.move_with_vector(&input_vec).to_vec()};
        next.update_location(camera_key, head_trans);

        next
    }
}