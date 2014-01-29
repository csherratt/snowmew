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

use std::num::{zero};

use snowmew::core::Database;
use snowmew::shader::Shader;

use render::RenderManager;

use cgmath::quaternion::*;
use cgmath::transform::*;
use cgmath::vector::*;
use cgmath::matrix::*;
use cgmath::angle::{ToRad, deg};

use glfw::Monitor;

use extra::time::precise_time_s;


#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    do glfw::start {
        ovr::init();
        let dm = ovr::DeviceManager::new().unwrap();
        let dev = dm.enumerate().unwrap();
        let info = dev.get_info().unwrap();
        let sf = ovr::SensorFusion::new().unwrap();
        let sensor = dev.get_sensor().unwrap();
        sf.attach_to_sensor(&sensor);

        let (width, height) = info.resolution();

        let monitors = Monitor::get_connected();
        println!("{} {}", width, height);
        glfw::window_hint::context_version(4, 0);
        glfw::window_hint::opengl_profile(glfw::OpenGlCoreProfile);
        glfw::window_hint::opengl_forward_compat(true);

        let window = glfw::Window::create(width as u32, height as u32, "OpenGL", glfw::FullScreen(monitors[info.id()])).unwrap();
        window.make_context_current();
        glfw::set_swap_interval(1);

        gl::load_with(glfw::get_proc_address);

        let mut db = Database::new();

        let camera_dolly = db.new_object(None, ~"dolly");
        let camera = db.new_object(Some(camera_dolly), ~"camera");
        let scene = db.new_object(None, ~"scene");
        let geometry = db.find("core/geometry/cube").unwrap();
        let shader = db.find("core/shaders/rainbow").unwrap();

        let size = 20;

        for y in range(-size, size) { for x in range(-size, size) {for z in range(-size, size) {
            let cube_id = db.new_object(Some(scene), format!("cube_{}_{}_{}", x, y, z));
            let x = x as f32 * 1.5;
            let y = y as f32 * 1.5;
            let z = z as f32 * 1.5;
            db.update_location(cube_id,
                Transform3D::new(0.5f32, Quat::from_euler(deg(15f32).to_rad(), deg(0f32).to_rad(), deg(0f32).to_rad()), Vec3::new(y, x, z)));
            db.set_draw(cube_id, geometry, shader);
        }}}

        db.update_location(camera,
            Transform3D::new(1f32,
                             Quat::from_euler(deg(0f32).to_rad(), deg(0f32).to_rad(), deg(0f32).to_rad()),
                             Vec3::new(0f32, 0f32, 0f32)));


        let mut ren = RenderManager::new(db.clone());
        ren.load();

        glfw::poll_events();

        let (wx, wy) = window.get_size();
        window.set_cursor_pos(wx as f64 /2., wy as f64/2.);

        let (mut rot_x, mut rot_y) = (0_f64, 0_f64);
        let mut pos = Vec3::new(0f32, 0f32, 0f32);

        while !window.should_close() {
            glfw::poll_events();
            let start = precise_time_s();

            match window.is_focused() {
                true => {
                    window.set_cursor_mode(glfw::CursorHidden);
                    let (x, y) = window.get_cursor_pos();
                    let (wx, wy) = window.get_size();
                    let (wx, wy) = (wx as f64, wy as f64);
                    window.set_cursor_pos(wx/2., wy/2.);

                    rot_x += (x - wx/2.) / 3.;
                    rot_y += (y - wy/2.) / 3.;

                    rot_y = rot_y.max(&-90.).min(&90.);
                },
                false => {
                    window.set_cursor_mode(glfw::CursorNormal);
                }
            }

            if window.get_key(glfw::KeySpace) == glfw::Press {
                rot_x = 0.;
                rot_y = 0.;
            }

            let input_vec = Vec4::new(
                if window.get_key(glfw::KeyA) == glfw::Press {0.5f32} else {0f32} +
                if window.get_key(glfw::KeyD) == glfw::Press {-0.5f32} else {0f32}, 
                0f32,
                if window.get_key(glfw::KeyW) == glfw::Press {0.5f32} else {0f32} +
                if window.get_key(glfw::KeyS) == glfw::Press {-0.5f32} else {0f32},
                1f32
            );

            let rot = sf.get_predicted_orientation(None);

            //let rot =  Quat::from_axis_angle(&Vec3::new(0f32, 1f32, 0f32), deg(-rot_x as f32).to_rad()).mul_q(
            //          &Quat::from_axis_angle(&Vec3::new(1f32, 0f32, 0f32), deg(-rot_y as f32).to_rad()));



            let trans = Transform3D::new(1f32,
                                 rot.normalize(),
                                 Vec3::new(0f32, 0f32, 0f32));

            let pos_v = trans.get().rot.to_mat3().to_mat4().mul_v(&input_vec);
            let new_pos = Vec3::new(pos_v.x, pos_v.y, pos_v.z);
            pos = pos.add_v(&new_pos);


            //let rot =  Quat::from_axis_angle(&Vec3::new(1f32, 0f32, 0f32), deg(rot_y as f32).to_rad()).mul_q(
            //          &Quat::from_axis_angle(&Vec3::new(0f32, 1f32, 0f32), deg(rot_x as f32).to_rad())).normalize();


            let dolly_trans = Transform3D::new(1f32, Quat::zero(), pos);
            let camera_trans = Transform3D::new(1f32, rot.normalize(), zero());


            db.update_location(camera, camera_trans);
            db.update_location(camera_dolly, dolly_trans);

            ren.update(db.clone());

            ren.render_vr(scene, camera, &info, &window);
            let end = precise_time_s();

            let time = (end-start);

            print!("\rfps: {:0.2f} time: {:0.3f}ms, budget: {:0.2f}                 ",
                1./time, time*1000., time/(1./60.));
        }
    }
}