//   Copyright 2014 Colin Sherratt
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

#![crate_name = "snowmew-loader"]
#![license = "ASL2"]
#![crate_type = "lib"]
#![comment = "An asset loader for snowmew"]

extern crate debug;
extern crate collections;
extern crate core;

extern crate cgmath;
extern crate image = "stb_image";

extern crate snowmew  = "snowmew-core";
extern crate graphics = "snowmew-graphics";

pub use obj::Obj;

mod obj;
mod mtl;
mod texture;