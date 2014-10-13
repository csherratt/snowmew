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
extern crate native;
extern crate opencl;
extern crate sync;
extern crate glfw;

extern crate "snowmew-core" as snowmew;
extern crate "snowmew-render-mux" as render;
extern crate "snowmew-position" as position;
extern crate "snowmew-graphics" as graphics;
extern crate "snowmew-render-data" as render_data;

use std::from_str::FromStr;

use cgmath::*;

use snowmew::input;
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
    native::start(argc, argv, main)
}

fn main() {
    let mut sc = snowmew::SnowmewConfig::new();

    let mut gd = GameData::new();
    let scene = gd.new_scene("scene");

    let cube = gd.find("core/geometry/cube").expect("cube not found");
    let red = gd.find("core/material/flat/red").expect("red not found");

    let args = std::os::args();
    let count = if args.len() >= 2 {
        FromStr::from_str(args[1].as_slice()).expect("Could not parse int")
    } else {
        10i
    };

    for x in range(-count, count) {
        for y in range(-count, count) {
            for z in range(-count, count) {
                let new = gd.new_object(Some(scene), format!("cube_{}_{}_{}", x, y, z).as_slice());
                let x = x as f32 * 2.5;
                let y = y as f32 * 2.5;
                let z = z as f32 * 2.5;
                gd.set_scale(new, 0.25);
                gd.set_displacement(new, Vector3::new(x, y, z));
                gd.set_draw(new, cube, red);
            }
        }
    }

    let (mut rot_x, mut rot_y) = (0_f64, 0_f64);
    let mut pos = Point3::new(0f32, 0f32, 0f32);

    let sun = light::Directional::new(Vector3::new(0.5f32, 1., 0.5),
                                      Vector3::new(1f32, 1., 1.), 0.25);
    gd.new_light(scene, "sun", light::DirectionalLight(sun));

    let camera = gd.new_object(None, "camera");
    gd.set_to_identity(camera);

    gd.set_scene(scene);
    gd.set_camera(camera);

    sc.start(box RenderFactory::new(), Cubes, gd);

    /*sc.start(gd, |gd, current_input, last_input| {
        let mut gd = gd;
        match current_input.is_focused() {
            true => {
                match current_input.cursor_delta(last_input.time()) {
                    Some((x, y)) => {
                        rot_x += x / 3.;
                        rot_y += y / 3.;

                        rot_y = rot_y.max(-90.).min(90.);
                        if rot_x > 360. {
                            rot_x -= 360.
                        } else if rot_x < -360. {
                            rot_x += 360.
                        }
                    },
                    None => (),
                }

            },
            false => {}
        }

        let input_vec = Vector3::new(
            if current_input.key_down(glfw::KeyA) {0.05f32} else {0f32} +
            if current_input.key_down(glfw::KeyD) {-0.05f32} else {0f32}, 
            0f32,
            if current_input.key_down(glfw::KeyW) {0.05f32} else {0f32} +
            if current_input.key_down(glfw::KeyS) {-0.05f32} else {0f32}
        );

        let rot: Quaternion<f32> = Rotation3::from_axis_angle(&Vector3::new(0f32, 1f32, 0f32), deg(-rot_x as f32).to_rad());
        rot.mul_q(&Rotation3::from_axis_angle(&Vector3::new(1f32, 0f32, 0f32), deg(-rot_y as f32).to_rad()));
        let camera = Camera::new(Decomposed{scale: 1f32,
                                            rot:   rot,
                                            disp:  pos.to_vec()}.to_matrix4());

        pos = camera.move_with_vector(&input_vec.mul_s(-1f32));
        let head_trans = Decomposed{scale: 1f32,
                                    rot:   rot,
                                    disp:  pos.to_vec()};
        gd.update_location(camera_loc, head_trans);

        (gd, scene, camera_loc)
    });*/
}

struct Cubes;

impl Game<GameData, input::Event> for Cubes {
    fn step(&mut self, event: input::Event, gd: GameData) -> GameData {
        let mut next = gd.clone();
        let gears_dir = next.find("scene/gears").unwrap();

        match event {
            input::Cadance(_, time) => {
                for (idx, (_, logo)) in gd.walk_dir(gears_dir).enumerate() {
                    let t = time as f32 * 10.;
                    let this_gear_rot = if idx % 2 == 0 { t } else { 5.625 - t };
                    next.set_rotation(logo, Rotation3::from_euler(deg(0f32).to_rad(),
                                                                  deg(this_gear_rot).to_rad(),
                                                                  deg(90f32).to_rad()));
                }
            }
            _ => ()
        }
        gd
    }
}