#![crate_id = "demo-cubes"]
#![feature(macro_rules)]
#![feature(globs)]

extern crate snowmew;
extern crate render = "snowmew-render";
extern crate position = "snowmew-position";
extern crate graphics = "snowmew-graphics";
extern crate cgmath;
extern crate native;
extern crate OpenCL;
extern crate sync;
extern crate glfw;

use std::io::timer::Timer;
use std::from_str::FromStr;
use sync::Arc;

use cgmath::quaternion::*;
use cgmath::transform::*;
use cgmath::vector::*;
use cgmath::point::*;
use cgmath::matrix::*;
use cgmath::rotation::*;
use cgmath::angle::{ToRad, deg};

use OpenCL::hl::{Device, get_platforms, GPU, CPU};

use snowmew::camera::Camera;
use position::Positions;
use graphics::Graphics;
use graphics::light;


use render::RenderManager;
use snowmew::common::Common;

use gamedata::GameData;

mod gamedata;

fn get_cl() -> Option<Arc<Device>> {
    let platforms = get_platforms();

    // find a gpu
    for platform in platforms.iter() {
        let devices = platform.get_devices_by_types(&[GPU]);
        if devices.len() != 0 {
            return Some(Arc::new(*devices.get(0)));
        } 
    }

    // use cpu if no gpu was found
    for platform in platforms.iter() {
        let devices = platform.get_devices_by_types(&[CPU, GPU]);
        if devices.len() != 0 {
            return Some(Arc::new(*devices.get(0)));
        } 
    }

    None
}


#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    snowmew::start_manual_input(proc(im) {
        let args = std::os::args();
        let count = if args.len() >= 2 {
            FromStr::from_str(args.get(1).as_slice()).expect("Could not parse int")
        } else {
            10
        };

        println!("Starting");
        let display = im.hmd().unwrap_or_else(|| {
            im.window((1280, 800)).expect("Failed to create window")
        });

        let mut db = GameData::new();
        let scene = db.new_scene("scene");

        let cube = db.find("core/geometry/cube").expect("cube not found");
        let red = db.find("core/material/flat/red").expect("red not found");

        for x in range(-count, count) {
            for y in range(-count, count) {
                for z in range(-count, count) {
                    let new = db.new_object(Some(scene), format!("cube_{}_{}_{}", x, y, z).as_slice());
                    let x = x as f32 * 2.5;
                    let y = y as f32 * 2.5;
                    let z = z as f32 * 2.5;
                    db.update_location(new,
                        Transform3D::new(0.25f32,
                                         Rotation3::from_euler(deg(0f32).to_rad(), deg(0f32).to_rad(), deg(0f32).to_rad()),
                                         Vector3::new(x, y, z))
                    );
                    db.set_draw(new, cube, red);
                }
            }
        }

        let ovr_camera = display.is_hmd();
        let camera_loc = db.new_object(None, "camera");
        db.update_location(camera_loc,
            Transform3D::new(1f32,
                             Rotation3::from_euler(deg(0f32).to_rad(), deg(0f32).to_rad(), deg(0f32).to_rad()),
                             Vector3::new(0f32, 0f32, 0f32)));

        let ih = display.handle();
        let last_input = im.get(&ih);
        let (wx, wy) = last_input.screen_size();

        let mut ren = match get_cl() {
            Some(dev) => RenderManager::new_cl(display, (wx, wy), dev),
            None => RenderManager::new(display, (wx, wy))
        };

        let (mut rot_x, mut rot_y) = (0_f64, 0_f64);
        let mut pos = if args.len() >= 5 {
            let x = FromStr::from_str(args.get(2).as_slice());
            let y = FromStr::from_str(args.get(3).as_slice());
            let z = FromStr::from_str(args.get(4).as_slice());
            match (x, y, z) {
                (Some(x), Some(y), Some(z)) => {
                    Point3::new(x, y, z)
                }
                _ => Point3::new(0f32, 0., 0.)
            }
        } else {
            Point3::new(0f32, 0f32, 0f32)
        };

        let sun = light::Directional::new(Vector3::new(0.5f32, 1., 0.5),
                                          Vector3::new(1f32, 1., 1.), 0.25);
        db.new_light(scene, "sun", light::Directional(sun));

        let mut last_input = im.get(&ih);
        let mut timer = Timer::new().unwrap();
        let timer_port = timer.periodic(10);
        while !last_input.should_close() {
            im.poll();
            let input_state = im.get(&ih);
            timer_port.recv();

            match input_state.is_focused() {
                true => {
                    match input_state.cursor_delta(last_input.time()) {
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
                if input_state.key_down(glfw::KeyA) {0.05f32} else {0f32} +
                if input_state.key_down(glfw::KeyD) {-0.05f32} else {0f32}, 
                0f32,
                if input_state.key_down(glfw::KeyW) {0.05f32} else {0f32} +
                if input_state.key_down(glfw::KeyS) {-0.05f32} else {0f32}
            );

            let rot: Quaternion<f32> = Rotation3::from_axis_angle(&Vector3::new(0f32, 1f32, 0f32), deg(-rot_x as f32).to_rad());
            let rot = if !ovr_camera {
                rot.mul_q(&Rotation3::from_axis_angle(&Vector3::new(1f32, 0f32, 0f32), deg(-rot_y as f32).to_rad()))
            } else {
                rot
            };

            let camera = Camera::new(Transform3D::new(1f32, rot, pos.to_vec()).to_matrix4());
            pos = camera.move(&input_vec.mul_s(-1f32));
            let head_trans = Transform3D::new(1f32, rot, pos.to_vec());
            db.update_location(camera_loc, head_trans);
            ren.update(box db.clone(), scene, camera_loc);
            last_input = input_state;
        }
    });
}