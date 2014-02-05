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
extern mod SendRecvReply;
extern mod native;
extern mod std;

pub mod core;
pub mod geometry;
pub mod shader;
pub mod camera;
pub mod input;

mod default;

// THIS CURRENTLY CAUSES PERFORMANCE ISSUES WITH SOME DRIVERS
// USE WITH CAUTION
pub fn start_managed_input(f: proc(input::InputManager))
{
    glfw::start(proc() {
        let f = f;
        let im = input::InputManager::new();
        let im_game = im.clone();

        let (p, c) = std::comm::Chan::new();

        spawn(proc() {
            println!("game- starting")
            f(im_game);
            println!("game- completed");
            c.send(());

        });

        loop {
            match p.try_recv() {
                std::comm::Empty => im.wait(),
                std::comm::Disconnected | std::comm::Data(_) => break
            }
        }

        im.finish();
    });
}

pub fn start_manual_input(f: proc(input::InputManager))
{
    glfw::start(proc() {
        let f = f;
        let im = input::InputManager::new();
        let im_game = im.clone();
        f(im_game);
        println!("Cleaning up input manager");
        im.finish();
        println!("done");
    });
}