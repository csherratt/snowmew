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

#![crate_name = "gl_cl"]
#![crate_type = "lib"]

extern crate gl;
extern crate opencl;
extern crate libc;

use std::ptr;

use opencl::cl::{cl_mem, cl_mem_flags, cl_context, cl_int, cl_uint, cl_event};
use opencl::cl::{cl_command_queue};
use opencl::hl::{Context, Device, EventList, Event, CommandQueue};
use opencl::hl::create_context_with_properties;
use opencl::mem::{CLBuffer, Buffer};
use opencl::error;

type CGLContextObj = libc::intptr_t;
type CGLShareGroupObj = libc::intptr_t;

#[cfg(target_os = "macos")]
const CL_CONTEXT_PROPERTY_USE_CGL_SHAREGROUP_APPLE: libc::intptr_t = 0x10000000;

#[cfg(target_os = "linux")]
const CL_GL_CONTEXT_KHR: libc::intptr_t = 0x2008;
const CL_GLX_CONTEXT_KHR: libc::intptr_t = 0x200A;

extern {
    #[cfg(target_os = "macos")]
    fn CGLGetCurrentContext() -> CGLContextObj;

    #[cfg(target_os = "macos")]
    fn CGLGetShareGroup(ctx: CGLContextObj) -> CGLShareGroupObj;

    #[cfg(target_os="linux")]
    fn glXGetCurrentContext() -> libc::intptr_t;

    #[cfg(target_os="linux")]
    fn glXGetCurrentDisplay() -> libc::intptr_t;

    fn clCreateFromGLBuffer(ctx: cl_context,
                            flags: cl_mem_flags,
                            buf: gl::types::GLuint,
                            err: *mut cl_int) -> cl_mem;

    fn clEnqueueAcquireGLObjects(cq: cl_command_queue,
                                 num_object: cl_uint,
                                 mem_objects: *const cl_mem,
                                 num_events_in_wait_list: cl_uint,
                                 event_wait_list: *const cl_event,
                                 event: *mut cl_event) -> cl_int;

    fn clEnqueueReleaseGLObjects(cq: cl_command_queue,
                                 num_object: cl_uint,
                                 mem_objects: *const cl_mem,
                                 num_events_in_wait_list: cl_uint,
                                 event_wait_list: *const cl_event,
                                 event: *mut cl_event) -> cl_int;
}

#[cfg(target_os = "macos")]
pub fn create_context(dev: &Device) -> Option<Context> {
    unsafe {
        let ctx = CGLGetCurrentContext();
        let grp = CGLGetShareGroup(ctx);

        let properties = &[CL_CONTEXT_PROPERTY_USE_CGL_SHAREGROUP_APPLE, grp, 0];

        Some(create_context_with_properties(&[*dev], properties))
    }
}

#[cfg(target_os = "linux")]
pub fn create_context(dev: &Device) -> Option<Context> {
    unsafe {
        let ctx = glXGetCurrentContext();
        let disp = glXGetCurrentDisplay();

        println!("opencl {} {}", ctx, disp);

        let properties = &[CL_GL_CONTEXT_KHR, ctx as libc::intptr_t,
                           CL_GLX_CONTEXT_KHR, disp as libc::intptr_t,
                           0];

        Some(create_context_with_properties(&[*dev], properties))
    }
}

pub fn create_from_gl_buffer<T>(ctx: &Context, buf: gl::types::GLuint, flags: cl_mem_flags) -> opencl::mem::CLBuffer<T> {
    unsafe {
        let mut status = 0;
        let mem = clCreateFromGLBuffer(ctx.ctx, flags, buf, &mut status);
        assert!(status == 0);
        opencl::mem::CLBuffer{cl_buffer: mem}
    }
}

pub trait AcquireRelease {
    fn acquire_gl_objects<T, E: EventList>(&self, mem: &[CLBuffer<T>], events: E) -> Event;
    fn release_gl_objects<T, E: EventList>(&self, mem: &[CLBuffer<T>], events: E) -> Event;
}

impl AcquireRelease for CommandQueue {
    fn acquire_gl_objects<T, E: EventList>(&self, mem: &[CLBuffer<T>], events: E) -> Event {
        let mem: Vec<cl_mem> = mem.iter().map(|m| m.id()).collect();
        let mut event: cl_event = ptr::null_mut();
        let check = events.as_event_list(|evt, evt_len| {
            unsafe {
                clEnqueueAcquireGLObjects(self.cqueue,
                                          mem.len() as cl_uint,
                                          &mem[0],
                                          evt_len,
                                          evt,
                                          &mut event as *mut cl_event)

            }
        });
        error::check(check, "Could not acquire_gl_objects");
        Event{ event: event }
    }

    fn release_gl_objects<T, E: EventList>(&self, mem: &[CLBuffer<T>], events: E) -> Event {
        let mem: Vec<cl_mem> = mem.iter().map(|m| m.id()).collect();
        let mut event: cl_event = ptr::null_mut();
        let check = events.as_event_list(|evt, evt_len| {
            unsafe {
                clEnqueueReleaseGLObjects(self.cqueue,
                                          mem.len() as cl_uint,
                                          &mem[0],
                                          evt_len,
                                          evt,
                                          &mut event as *mut cl_event)

            }
        });
        error::check(check, "Could not release_gl_objects");
        Event{ event: event }
    }
}