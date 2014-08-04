#![crate_name = "demo-noclip"]
#![feature(macro_rules)]
#![feature(globs)]

extern crate glfw;
extern crate gl;
extern crate snowmew  = "snowmew-core";
extern crate render = "snowmew-render-mux";
extern crate loader = "snowmew-loader";
extern crate position = "snowmew-position";
extern crate graphics = "snowmew-graphics";
extern crate cgmath;
extern crate native;
extern crate green;
extern crate ovr = "ovr-vr";
extern crate opencl;
extern crate sync;
extern crate render_data = "render-data";

use std::from_str::FromStr;

use cgmath::quaternion::*;
use cgmath::transform::*;
use cgmath::vector::*;
use cgmath::point::*;
use cgmath::matrix::*;
use cgmath::rotation::*;
use cgmath::angle::{ToRad, deg};

use snowmew::camera::Camera;
use position::Positions;
use graphics::{Graphics};
use graphics::light;

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
    let mut sc = snowmew::SnowmewConfig::new();
    sc.render = Some(box RenderFactory::new());

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
    let loader = Obj::load(&path).expect("Failed to load OBJ");
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

    let (mut rot_x, mut rot_y) = (0_f64, 0_f64);
    let mut pos = if args.len() >= 6 {
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

    let sun = light::Directional::new(Vector3::new(0.5f32, 1., 0.5),
                                      Vector3::new(1f32, 1., 1.), 1.);
    db.new_light(scene, "sun", light::Directional(sun));

    sc.start(db, |gd, input_state, last_input| {
        let mut gd = gd;
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
            if input_state.key_down(glfw::KeyA) {0.01f32} else {0f32} +
            if input_state.key_down(glfw::KeyD) {-0.01f32} else {0f32}, 
            0f32,
            if input_state.key_down(glfw::KeyW) {0.01f32} else {0f32} +
            if input_state.key_down(glfw::KeyS) {-0.01f32} else {0f32}
        );

        let rot: Quaternion<f32> = Rotation3::from_axis_angle(&Vector3::new(0f32, 1f32, 0f32), deg(-rot_x as f32).to_rad());

        let camera = Camera::new(Decomposed{scale: 1f32,
                                            rot:   rot,
                                            disp:  pos.to_vec()}.to_matrix4());
        pos = camera.move(&input_vec.mul_s(-1f32));
        let head_trans = Decomposed{scale: 1f32,
                                    rot:   rot,
                                    disp:  pos.to_vec()};
        gd.update_location(camera_loc, head_trans);

        (gd, scene, camera_loc)
    });
}