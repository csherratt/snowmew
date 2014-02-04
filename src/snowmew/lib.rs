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

pub fn start(f: proc(input::InputManager))
{
    glfw::start(proc() {
        let f = f;
        let im = input::InputManager::new();
        let im_game = im.clone();
        spawn(proc() {
            f(im_game);    
        });
        im.run();
    });
}