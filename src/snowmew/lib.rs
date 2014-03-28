#[crate_id = "github.com/csherratt/snowmew#snowmew:0.1"];
#[license = "ASL2"];
#[crate_type = "lib"];
#[comment = "A game engine in rust"];

#[feature(macro_rules)];
#[feature(globs)];

extern crate std;
extern crate time;
extern crate glfw;
extern crate cgmath;
extern crate cow;
extern crate octtree;
extern crate sync;
extern crate OpenCL;
extern crate native;
extern crate std;
extern crate gl;
extern crate green;
extern crate collections;
extern crate ovr = "oculus-vr";

pub use core::{object_key, Database};
pub use geometry::{VertexBuffer};
pub use position::{Positions, Deltas, CalcPositionsCl};

pub mod core;
pub mod geometry;
pub mod camera;
pub mod input;
pub mod display;
pub mod material;
pub mod position;
mod timing;

mod default;

fn setup_glfw()
{
    glfw::window_hint(glfw::ContextVersion(4, 1));
    glfw::window_hint(glfw::OpenglProfile(glfw::OpenGlCoreProfile));
    glfw::window_hint(glfw::OpenglForwardCompat(true));
    glfw::window_hint(glfw::Visible(false));
    glfw::window_hint(glfw::DepthBits(0));
    glfw::window_hint(glfw::StencilBits(0));
    glfw::set_swap_interval(0);
}

pub fn start_managed_input(f: proc(&mut input::InputManager))
{
    glfw::start(proc() {
        setup_glfw();
        let f = f;
        let im = input::InputManager::new();
        let (send, recv) = channel();

        let task = std::task::task().named("game task");

        task.spawn(proc() {
            green::run(proc() {
                let mut im = im;
                println!("game- starting")
                f(&mut im);
                println!("game- completed");
                send.send(im);
            });
        });

        loop {
            glfw::wait_events();
            match recv.try_recv() {
                std::comm::Empty => (),
                std::comm::Disconnected => fail!("Should not have received Disconnected"),
                std::comm::Data(im) => {
                    im.finish();
                    return
                }
            }
        }
    });
}

pub fn start_manual_input(f: proc(&mut input::InputManager))
{
    glfw::start(proc() {
        setup_glfw();

        let f = f;
        let mut im = input::InputManager::new();
        f(&mut im);
        println!("Cleaning up input manager");
        im.finish();
        println!("done");
    });
}