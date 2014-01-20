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

pub mod core;
pub mod geometry;
pub mod shader;
