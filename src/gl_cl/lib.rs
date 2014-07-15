#![crate_name = "gl_cl"]
#![comment = "An OpenGL OpenCL bridge utility library"]
#![license = "ASL2"]
#![crate_type = "lib"]

extern crate gl;
extern crate OpenCL;
extern crate libc;

use std::ptr;

use OpenCL::CL::{cl_mem, cl_mem_flags, cl_context, cl_int, cl_uint, cl_event};
use OpenCL::CL::{cl_command_queue};
use OpenCL::hl::{Context, Device, EventList, Event, CommandQueue};
use OpenCL::hl::create_context_with_properties;
use OpenCL::mem::{CLBuffer, Buffer};
use OpenCL::error;

type CGLContextObj = libc::intptr_t;
type CGLShareGroupObj = libc::intptr_t;

#[cfg(target_os = "macos")]
static CL_CONTEXT_PROPERTY_USE_CGL_SHAREGROUP_APPLE: libc::intptr_t = 0x10000000;

#[cfg(target_os = "linux")]
static CL_GL_CONTEXT_KHR: libc::intptr_t = 0x2008;
static CL_GLX_CONTEXT_KHR: libc::intptr_t = 0x200A;

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

pub fn create_from_gl_buffer<T>(ctx: &Context, buf: gl::types::GLuint, flags: cl_mem_flags) -> OpenCL::mem::CLBuffer<T> {
    unsafe {
        let mut status = 0;
        let mem = clCreateFromGLBuffer(ctx.ctx, flags, buf, &mut status);
        assert!(status == 0);
        OpenCL::mem::CLBuffer{cl_buffer: mem}
    }
}

pub trait AcquireRelease {
    fn acquire_gl_objects<T, E: EventList>(&self, mem: &[CLBuffer<T>], events: E) -> Event;
    fn release_gl_objects<T, E: EventList>(&self, mem: &[CLBuffer<T>], events: E) -> Event;
}

impl AcquireRelease for CommandQueue {
    fn acquire_gl_objects<T, E: EventList>(&self, mem: &[CLBuffer<T>], events: E) -> Event {
        let mem: Vec<cl_mem> = mem.iter().map(|m| m.id()).collect();
        let mut event: cl_event = ptr::null();
        let check = events.as_event_list(|evt, evt_len| {
            unsafe {
                clEnqueueAcquireGLObjects(self.cqueue,
                                          mem.len() as cl_uint,
                                          mem.get(0),
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
        let mut event: cl_event = ptr::null();
        let check = events.as_event_list(|evt, evt_len| {
            unsafe {
                clEnqueueReleaseGLObjects(self.cqueue,
                                          mem.len() as cl_uint,
                                          mem.get(0),
                                          evt_len,
                                          evt,
                                          &mut event as *mut cl_event)

            }
        });
        error::check(check, "Could not release_gl_objects");
        Event{ event: event }
    }
}