#[crate_id = "snowmew-cube"];

#[feature(macro_rules)];
#[feature(globs)];

extern mod glfw;
extern mod gl;
extern mod snowmew;
extern mod render = "snowmew-render";
extern mod cgmath;
extern mod native;
extern mod extra;
extern mod ovr = "ovr-rs";

use snowmew::core::Database;
use snowmew::display::Display;
use snowmew::camera::Camera;

use render::RenderManager;

use cgmath::quaternion::*;
use cgmath::transform::*;
use cgmath::vector::*;
use cgmath::point::*;
use cgmath::matrix::*;
use cgmath::rotation::*;
use cgmath::angle::{ToRad, deg};

use extra::time::precise_time_s;


#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    snowmew::start_managed_input(proc(im) {
        let (mut display, mut display_input) = Display::new_ovr(im).unwrap();

        let mut db = Database::new();
        let camera_loc = db.new_object(None, ~"camera");

        let scene = db.new_object(None, ~"scene");
        let geometry = db.find("core/geometry/cube").unwrap();
        let shader = db.find("core/shaders/rainbow").unwrap();

        let size = 20;

        for y in range(-size, size) { for x in range(-size, size) {for z in range(-size, size) {
            let cube_id = db.new_object(Some(scene), format!("cube_{}_{}_{}", x, y, z));
            let x = x as f32 * 2.5;
            let y = y as f32 * 2.5;
            let z = z as f32 * 2.5;
            db.update_location(cube_id,
                Transform3D::new(0.5f32, Rotation3::from_euler(deg(15f32).to_rad(), deg(0f32).to_rad(), deg(0f32).to_rad()), Vec3::new(y, x, z)));
            db.set_draw(cube_id, geometry, shader);
        }}}

        db.update_location(camera_loc,
            Transform3D::new(1f32,
                             Rotation3::from_euler(deg(0f32).to_rad(), deg(0f32).to_rad(), deg(0f32).to_rad()),
                             Vec3::new(0f32, 0f32, 0f32)));

        let mut ren = RenderManager::new(db.clone());
        ren.load();

        let (wx, wy) = display.size();
        display_input.set_cursor(wx as f64 /2., wy as f64/2.);

        let (mut rot_x, mut rot_y) = (0_f64, 0_f64);
        let mut pos = Point3::new(0f32, 0f32, 0f32);

        let mut last_input = display_input.get();

        while !last_input.should_close() {
            let input_state = display_input.get();
            let start = precise_time_s();

            match input_state.is_focused() {
                true => {
                    //display.set_cursor_mode(glfw::CursorNormal);
                    match input_state.cursor_delta(last_input.time()) {
                        Some((x, y)) => {
                            let (wx, wy) = display.size();
                            let (wx, wy) = (wx as f64, wy as f64);
                            display_input.set_cursor(wx/2., wy/2.);

                            rot_x += x / 3.;
                            rot_y += y / 3.;

                            rot_y = rot_y.max(&-90.).min(&90.);
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
                //sf.reset();
            }

            let input_vec = Vec3::new(
                if input_state.key_down(glfw::KeyA) {0.5f32} else {0f32} +
                if input_state.key_down(glfw::KeyD) {-0.5f32} else {0f32}, 
                0f32,
                if input_state.key_down(glfw::KeyW) {0.5f32} else {0f32} +
                if input_state.key_down(glfw::KeyS) {-0.5f32} else {0f32}
            );

            //let rift = sf.get_predicted_orientation(None);
            let rift = input_state.predicted.clone();
            let rot: Quat<f32> =  Rotation3::from_axis_angle(&Vec3::new(0f32, 1f32, 0f32), deg(-rot_x as f32).to_rad());
            let rot = rot.mul_q(&Rotation3::from_axis_angle(&Vec3::new(1f32, 0f32, 0f32), deg(-rot_y as f32).to_rad()));

            let rift = rift.mul_q(&Rotation3::from_axis_angle(&Vec3::new(0f32, 1f32, 0f32), deg(180 as f32).to_rad()));

            let camera = Camera::new(rot.clone(), Transform3D::new(1f32, rot, pos.to_vec()).to_mat4());
            pos = camera.move(&input_vec.mul_s(-1f32));

            let head_trans = Transform3D::new(1f32, rot.mul_q(&rift), pos.to_vec());

            db.update_location(camera_loc, head_trans);

            ren.update(db.clone());
            //ren.render_vr(scene, camera_loc,  &info, &mut display);
            ren.render(scene, camera_loc, /*&info,*/ &mut display);

            let end = precise_time_s();

            let time = (end-start);

            print!("\rfps: {:0.2f} time: {:0.3f}ms, budget: {:0.2f}                 ",
                1./time, time*1000., time/(1./60.));

            last_input = input_state;
        }
    });
}