#[link(name = "snowmew",
       vers = "0.1",
       author = "Colin Sherratt",
       url = "https://github.com/csherratt/snowmew")];

#[comment = "A game engine in rust"];
#[crate_type = "lib"];

extern mod glfw;
extern mod gl;

pub mod core;
pub mod coregl;