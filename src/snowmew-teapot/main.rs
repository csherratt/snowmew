#[crate_id = "snowmew-teapot"];
#[feature(macro_rules)];
#[feature(globs)];

extern mod snowmew;
extern mod loader = "snowmew-loader";
extern mod render = "snowmew-render";
extern mod cgmath;
extern mod native;
extern mod glfw = "glfw-rs";

use snowmew::core::Database;
use snowmew::display::Display;

use cgmath::transform::*;
use cgmath::vector::*;
use cgmath::rotation::*;
use cgmath::angle::{ToRad, deg};
use cgmath::point::*;
use cgmath::quaternion::*;


use render::RenderManager;

use loader::Obj;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    snowmew::start_managed_input(proc(im) {
        let mut db = Database::new();
        let teapot = Obj::load(&Path::new("assets/teapot.obj")).unwrap();

        let import = db.add_dir(None, ~"import");
        teapot.import(import, &mut db);

        let scene = db.add_dir(None, ~"scene");
        let geo = db.find("import/Teapot01").unwrap();
        let shader = db.find("core/shaders/rainbow_normal").unwrap();
        let teapot = db.new_object(Some(scene), ~"teapot");

        db.set_draw(teapot, geo, shader);
        db.update_location(teapot,
            Transform3D::new(1f32,
                             Rotation3::from_euler(deg(0f32).to_rad(), deg(0f32).to_rad(), deg(0f32).to_rad()),
                             Vec3::new(0f32, 0f32, 0f32)));

        let camera = db.new_object(Some(scene), ~"camera");
        db.update_location(camera,
            Transform3D::new(1f32,
                             Rotation3::from_euler(deg(0f32).to_rad(), deg(0f32).to_rad(), deg(0f32).to_rad()),
                             Vec3::new(0f32, 0f32, -25f32)));


        let (mut display, mut display_input) = Display::new_window(im, (1280, 800)).unwrap();
        let mut ren = RenderManager::new(db.clone());
        ren.load();

        let (mut rot_x, mut rot_y) = (0_f64, 0_f64);
        let mut pos = Point3::new(0f32, 0., -25.);

        let mut last_input = display_input.get();
        while !last_input.should_close() {
            let input = display_input.get();
            match input.is_focused() {
                true => {
                    display.set_cursor_mode(glfw::CursorHidden);
                    match input.cursor_delta(last_input.time()) {
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
                    display.set_cursor_mode(glfw::CursorNormal);
                }
            }

            let rot: Quat<f32> =  Rotation3::from_axis_angle(&Vec3::new(0f32, 1f32, 0f32), deg(-rot_x as f32).to_rad());
            let rot = rot.mul_q(&Rotation3::from_axis_angle(&Vec3::new(1f32, 0f32, 0f32), deg(rot_y as f32).to_rad()));

            let head_trans = Transform3D::new(1f32, rot, pos.to_vec());
            db.update_location(camera, head_trans);

            ren.update(db.clone());
            ren.render(scene, camera, &mut display);
            last_input = input;
        }

    });
}