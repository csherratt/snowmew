Snowmew
=======
[![Build Status](https://travis-ci.org/csherratt/snowmew.svg?branch=master)](https://travis-ci.org/csherratt/snowmew)

Snowmew is a game engine written entirely in rust. It is based around a 
copy-on-write game state that can be shared safely with tasks in parallel.

![snowmew-preview](https://s3.amazonaws.com/snowmew/Snowmew_004.png)

Building
--------

Make sure all submodules were cloned first.

    git submodule update --init --recursive

Building is straight forward.

    ./configure
    make


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
