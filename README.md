Snowmew
=======
[![Build Status](https://travis-ci.org/csherratt/snowmew.svg?branch=master)](https://travis-ci.org/csherratt/snowmew)

Snowmew is a game engine written entirely in rust. It is based around a 
copy-on-write game state that can be shared safely with tasks in parallel.

![snowmew-preview](https://raw.githubusercontent.com/csherratt/snowmew/master/.screenshot.jpg)

Last Tested Version
-------------------
`rustc 1.0.0-nightly (f4f10dba2 2015-01-17 20:31:08 +0000)`

Building
--------

Snowmew may require some dependencies to build, travis pulls down the following packages to build:

  sudo apt-get install libudev-dev libglfw-dev opencl-headers xorg-dev libglu1-mesa-dev freeglut3 freeglut3-dev libglfw-dev

In addition you will probably need either fglrx or the nvidia drivers. For the nvidia drivers be sure to include nvidia-opencl-dev
    
Now just build using cargo:

    cargo build

