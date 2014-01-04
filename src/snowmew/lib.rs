#[crate_id = "github.com/csherratt/snowmews#snowmew:0.1"];
#[license = "ASL2"];
#[crate_type = "lib"];

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
pub mod render;
pub mod db;