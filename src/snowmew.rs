#![crate_name = "snowmew"]

extern crate core = "snowmew-core";
extern crate graphics = "snowmew-graphics";
extern crate loader = "snowmew-loader";
extern crate physics = "snowmew-physics";
extern crate position = "snowmew-position";
extern crate render = "snowmew-render-mux";

pub mod config  {
    pub use SnowmewConfig = core::SnowmewConfig;
}

