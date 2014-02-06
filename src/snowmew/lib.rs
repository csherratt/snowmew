#[crate_id = "github.com/csherratt/snowmew#snowmew:0.1"];
#[license = "ASL2"];
#[crate_type = "lib"];
#[comment = "A game engine in rust"];

#[feature(macro_rules)];
#[feature(globs)];

extern mod std;
extern mod glfw;
extern mod cgmath;
extern mod cow;
extern mod octtree;
extern mod extra;
extern mod bitmap = "bitmap-set";
extern mod OpenCL;
extern mod native;
extern mod std;
extern mod gl;
extern mod ovr = "ovr-rs";

pub mod core;
pub mod geometry;
pub mod shader;
pub mod camera;
pub mod input;
pub mod display;

mod default;

fn setup_glfw()
{
    glfw::window_hint::context_version(4, 3);
    glfw::window_hint::opengl_profile(glfw::OpenGlCoreProfile);
    glfw::window_hint::opengl_forward_compat(true);

}

// THIS CURRENTLY CAUSES PERFORMANCE ISSUES WITH SOME DRIVERS
// USE WITH CAUTION
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