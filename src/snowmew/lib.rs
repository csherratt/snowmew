#[crate_id = "github.com/csherratt/snowmew#snowmew:0.1"];
#[license = "ASL2"];
#[crate_type = "lib"];
#[comment = "A game engine in rust"];

#[feature(macro_rules)];
#[feature(globs)];

extern crate semver;
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
pub mod io;
//pub mod display;
pub mod material;
pub mod position;
mod timing;

mod default;

fn setup_glfw() -> (glfw::Glfw, Receiver<glfw::Error>)
{
    let (glfw, erros) = glfw::init().ok().unwrap();

    glfw.window_hint(glfw::ContextVersion(4, 1));
    glfw.window_hint(glfw::OpenglProfile(glfw::OpenGlCoreProfile));
    glfw.window_hint(glfw::OpenglForwardCompat(true));
    glfw.window_hint(glfw::Visible(false));
    glfw.window_hint(glfw::DepthBits(0));
    glfw.window_hint(glfw::StencilBits(0));

    (glfw, erros)
}


pub fn start_manual_input(f: proc(&mut io::IOManager))
{
    let (glfw, errors) = setup_glfw();

    let f = f;
    let mut im = io::IOManager::new(glfw);
    f(&mut im);
    println!("done");
}