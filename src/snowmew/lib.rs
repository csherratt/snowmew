#[link(name = "snowmew",
       vers = "0.1",
       author = "Colin Sherratt",
       url = "https://github.com/csherratt/snowmew")];

#[comment = "A game engine in rust"];
#[crate_type = "lib"];

#[feature(macro_rules)];
#[feature(globs)];

extern mod std;
extern mod glfw;
extern mod gl;
extern mod cgmath;

pub mod core;
pub mod coregl;
pub mod geometry;
pub mod shader;
