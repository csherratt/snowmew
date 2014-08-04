Snowmew
=======
[![Build Status](https://travis-ci.org/csherratt/snowmew.svg?branch=master)](https://travis-ci.org/csherratt/snowmew)

Snowmew is a game engine written entirely in rust. It is based around a 
copy-on-write game state that can be shared safely with tasks in parallel.

![snowmew-preview](https://s3.amazonaws.com/snowmew/Snowmew_june_5.png)

Last Tested Version
-------------------
`rustc 0.11.0-pre-nightly (94343da1bdf4de84d0ece90d920400697ad7e143 2014-06-30 16:36:33 +0000)`

Building
--------

Snowmew may require some dependencies to build, travis pulls down the following packages to build:

  sudo apt-get install libudev-dev libglfw-dev opencl-headers xorg-dev libglu1-mesa-dev freeglut3 freeglut3-dev libglfw-dev

In addition you will probably need either fglrx or the nvidia drivers. For the nvidia drivers be sure to include nvidia-opencl-dev
    
Now just build using cargo:

    cargo build


Dependencies
------------

All required dependencies are included as submodules.

| Dependency  |
| ----------- |
| [cgmath-rs](https://github.com/bjz/cgmath-rs) |
| [collision-rs](https://github.com/csherratt/collision-rs) |
| [cow-rs](https://github.com/csherratt/cow-rs) |
| [gl-rs](https://github.com/bjz/gl-rs) |
| [glfw-rs](https://github.com/bjz/glfw-rs) |
| [rust-opencl](https://github.com/luqmana/rust-opencl) |
| [rust-stb-image](https://github.com/mozilla-servo/rust-stb-image/) |
| [vr-rs](https://github.com/csherratt/vr-rs) |
