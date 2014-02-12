#[crate_id = "github.com/csherratt/snowmew#snowmew:0.1"];
#[license = "ASL2"];
#[crate_type = "lib"];
#[comment = "A game engine in rust"];

#[feature(macro_rules)];
#[feature(globs)];

extern mod std;
extern mod extra;
extern mod glfw = "glfw-rs";
extern mod cgmath;
extern mod cow;
extern mod octtree;
extern mod sync;
extern mod bitmap = "bitmap-set";
extern mod OpenCL;
extern mod native;
extern mod std;
extern mod gl;
extern mod ovr = "ovr-rs";

pub use core::{object_key, Database};
pub use geometry::{VertexBuffer};

pub mod core;
pub mod geometry;
pub mod camera;
pub mod input;
pub mod display;
pub mod material;
mod timing;

mod default;

fn setup_glfw()
{
    glfw::window_hint::context_version(4, 4);
    glfw::window_hint::opengl_profile(glfw::OpenGlAnyProfile);
    glfw::window_hint::opengl_forward_compat(true);
    glfw::window_hint::visible(false);
}

#[cfg(target_os = "win32")]
pub fn start_managed_input(f: proc(&mut input::InputManager))
{
    glfw::start(proc() {
        setup_glfw();
        let f = f;
        let im = input::InputManager::new();
        let (p, c) = std::comm::Chan::new();

        spawn(proc() {
            let mut im = im;
            println!("game- starting")
            f(&mut im);
            println!("game- completed");
            c.send(im);

        });

        
        loop {
            glfw::wait_events();
            match p.try_recv() {
                std::comm::Empty => (),
                std::comm::Disconnected => fail!("Sound not have received Disconnected"),
                std::comm::Data(im) => {
                    im.finish();
                    return
                }
            }
        }
    });
}

// it is faster to do rendering on Thread1
#[cfg(not(target_os = "win32"))]
pub fn start_managed_input(f: proc(&mut input::InputManager))
{
    glfw::start(proc() {
        setup_glfw();
        let f = f;
        let mut im = input::InputManager::new();
        let (p, c): (Port<input::InputManager>, Chan<input::InputManager>) = std::comm::Chan::new();
        
        spawn(proc() {
            loop {
                glfw::wait_events();
                match p.try_recv() {
                    std::comm::Empty => (),
                    std::comm::Disconnected => fail!("Sound not have received Disconnected"),
                    std::comm::Data(im) => {
                        im.finish();
                        println!("done!");
                        return;
                    }
                }
            }
        });

        
        println!("game- starting")
        f(&mut im);
        println!("game- completed");
        c.send(im);

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