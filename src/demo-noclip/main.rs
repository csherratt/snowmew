#![crate_id = "demo-noclip"]
#![feature(macro_rules)]
#![feature(globs)]

extern crate glfw;
extern crate gl;
extern crate snowmew;
extern crate render = "snowmew-render";
extern crate loader = "snowmew-loader";
extern crate position = "snowmew-position";
extern crate graphics = "snowmew-graphics";
extern crate cgmath;
extern crate native;
extern crate green;
extern crate ovr = "oculus-vr";
extern crate OpenCL;
extern crate sync;

use std::io::timer::Timer;
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

use render::RenderManager;
use loader::Obj;
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
        if args.len() == 1 {
            println!("Please supply a path to an obj to load");
            return;
        }
        let path = Path::new(args.get(1).as_slice());
        let loader = Obj::load(&path).expect("Failed to load OBJ");

        println!("Starting");
        let display = im.hmd().unwrap_or_else(|| {
            im.window((1280, 800)).expect("Failed to create window")
        });
        let mut db = GameData::new();

        let import = db.new_object(None, "import");
        loader.import(import, &mut db);

        let scene = db.new_object(None, "scene");
        let geo_dir = db.find("import/objects").expect("geometry not found from import");
        for (name, id) in db.clone().walk_dir(geo_dir) {
            match db.get_draw(id) {
                Some(d) => {
                    let obj = db.new_object(Some(scene), name);
                    db.set_draw(obj, d.geometry, d.material);
                    db.update_location(obj,
                        Transform3D::new(1f32,
                                         Rotation3::from_euler(deg(0f32).to_rad(), deg(0f32).to_rad(), deg(0f32).to_rad()),
                                         Vector3::new(0f32, 0f32, 0f32))
                    );
                }
                None => ()
            }
        }

        let camera_loc = db.new_object(None, "camera");
        db.update_location(camera_loc,
            Transform3D::new(1f32,
                             Rotation3::from_euler(deg(0f32).to_rad(), deg(0f32).to_rad(), deg(0f32).to_rad()),
                             Vector3::new(0f32, 0f32, 0f32)));

        let ih = display.handle();
        let last_input = im.get(&ih);
        let (wx, wy) = last_input.screen_size();

        let mut ren = match get_cl() {
            Some(dev) => RenderManager::new_cl(box db.clone(), display, (wx, wy), dev),
            None => RenderManager::new(box db.clone(), display, (wx, wy))
        };

        let (mut rot_x, mut rot_y) = (0_f64, 0_f64);
        let mut pos = Point3::new(0f32, 0f32, 0f32);
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
            let rot = rot.mul_q(&Rotation3::from_axis_angle(&Vector3::new(1f32, 0f32, 0f32), deg(-rot_y as f32).to_rad()));

            let camera = Camera::new(Transform3D::new(1f32, rot, pos.to_vec()).to_matrix4());
            pos = camera.move(&input_vec.mul_s(-1f32));
            let head_trans = Transform3D::new(1f32, rot, pos.to_vec());
            db.update_location(camera_loc, head_trans);
            ren.update(box db.clone(), scene, camera_loc);
            last_input = input_state;
        }
    });
}