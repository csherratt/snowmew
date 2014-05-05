#![crate_id = "snowmew-cube"]
#![feature(macro_rules)]
#![feature(globs)]

extern crate glfw;
extern crate gl;
extern crate snowmew;
extern crate render = "snowmew-render";
extern crate loader = "snowmew-loader";
extern crate position = "snowmew-position";
extern crate cgmath;
extern crate native;
extern crate green;
extern crate ovr = "oculus-vr";
extern crate rand;
extern crate OpenCL;
extern crate sync;

use std::io::timer::Timer;
use rand::{StdRng, Rng};
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
use snowmew::graphics::Graphics;

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
            return Some(Arc::new(devices[0]));
        } 
    }

    // use cpu if no gpu was found
    for platform in platforms.iter() {
        let devices = platform.get_devices_by_types(&[CPU, GPU]);
        if devices.len() != 0 {
            return Some(Arc::new(devices[0]));
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
        println!("Starting");
        let display = im.window((1280, 800))
                .expect("Could not create a display");

        let mut db = GameData::new();

        let import = Obj::load(&Path::new("assets/suzanne.obj"))
                .expect("Could not fetch suzanne");

        import.import(db.add_dir(None, "import"), &mut db);

        let scene = db.new_object(None, "scene");
        let geometry = db.find("core/geometry/cube")
                .expect("Could not find Suzanne");

        let dir = db.find("core/material/flat").expect("Could not find flat");

        let mut rng = StdRng::new().unwrap();

        let mut materials = Vec::new();
        for (_, oid) in db.walk_dir(dir) {
            materials.push(*oid);
        }

        let size = 5;

        println!("creating");
        for x in range(-size, size) {
            let x_dir = db.new_object(Some(scene), format!("{}", x));
            db.update_location(x_dir,
                Transform3D::new(1f32, Quaternion::zero(), Vector3::new(x as f32 * 2.5, 0., 0.)));
            for y in range(-size, size) {
                let y_dir = db.new_object(Some(x_dir), format!("{}", y));
                db.update_location(y_dir,
                    Transform3D::new(1f32, Quaternion::zero(), Vector3::new(0., y as f32 * 2.5, 0.)));
                for z in range(-size, size) {
                    let materials = materials.slice(0, materials.len());
                    let material = rng.choose(materials);
                    let cube_id = db.new_object(Some(y_dir), format!("cube_{}", z));
                    let xa = x as f32 * 25.;
                    let ya = y as f32 * 25.;
                    let za = z as f32 * 25.;
                    let z = z as f32 * 2.5;
                    db.update_location(cube_id,
                        Transform3D::new(0.5f32, Rotation3::from_euler(deg(xa).to_rad(), deg(ya).to_rad(), deg(za).to_rad()), Vector3::new(0., 0., z)));
                    db.set_draw(cube_id, geometry, material);
                }
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

        let cl = get_cl();

        let mut ren = match cl {
            Some(dev) => RenderManager::new_cl(~db.clone(), display, (wx, wy), dev),
            None => RenderManager::new(~db.clone(), display, (wx, wy))
        };
        //display_input.set_cursor(wx as f64 /2., wy as f64/2.);

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
                    //display.set_cursor_mode(glfw::CursorDisabled);
                    match input_state.cursor_delta(last_input.time()) {
                        Some((x, y)) => {
                            //let (wx, wy) = input_state.screen_size();
                            //let (wx, wy) = (wx as f64, wy as f64);
                            //display_input.set_cursor(wx/2., wy/2.);

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
                false => {
                    //display.set_cursor_mode(glfw::CursorNormal);
                }
            }

            if input_state.key_down(glfw::KeySpace) {
                rot_x = 0.;
                rot_y = 0.;
                //display_input.reset_ovr();
            }

            let input_vec = Vector3::new(
                if input_state.key_down(glfw::KeyA) {-0.05f32} else {0f32} +
                if input_state.key_down(glfw::KeyD) {0.05f32} else {0f32}, 
                0f32,
                if input_state.key_down(glfw::KeyW) {-0.05f32} else {0f32} +
                if input_state.key_down(glfw::KeyS) {0.05f32} else {0f32}
            );

            let rot: Quaternion<f32> =  Rotation3::from_axis_angle(&Vector3::new(0f32, 1f32, 0f32), deg(-rot_x as f32).to_rad());
            let rot = rot.mul_q(&Rotation3::from_axis_angle(&Vector3::new(1f32, 0f32, 0f32), deg(rot_y as f32).to_rad()));

            let camera = Camera::new(rot.clone(), Transform3D::new(1f32, rot, pos.to_vec()).to_matrix4());
            pos = camera.move(&input_vec.mul_s(-1f32));

            let head_trans = Transform3D::new(1f32, rot, pos.to_vec());

            db.update_location(camera_loc, head_trans);

            ren.update(~db.clone(), scene, camera_loc);

            last_input = input_state;
        }
    });
}