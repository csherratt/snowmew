#![crate_id = "github.com/csherratt/snowmew#gl_cl:0.1"]
#![comment = "An OpenGL OpenCL bridge utility library"]
#![license = "ASL2"]
#![crate_type = "lib"]

extern crate gl;
extern crate OpenCL;
extern crate libc;

use OpenCL::CL::{cl_mem, cl_mem_flags, cl_context, cl_int};
use OpenCL::hl::{Context, Device};

#[cfg(target_os = "macos")]
use OpenCL::hl::create_context_with_properties;

type CGLContextObj = libc::intptr_t;
type CGLShareGroupObj = libc::intptr_t;

#[cfg(target_os = "macos")]
static CL_CONTEXT_PROPERTY_USE_CGL_SHAREGROUP_APPLE: libc::intptr_t = 0x10000000;

extern {
    #[cfg(target_os = "macos")]
    fn CGLGetCurrentContext() -> CGLContextObj;
    #[cfg(target_os = "macos")]
    fn CGLGetShareGroup(ctx: CGLContextObj) -> CGLShareGroupObj;

    fn clCreateFromGLBuffer(ctx: cl_context, flags: cl_mem_flags, buf: gl::types::GLuint, err: *mut cl_int) -> cl_mem;
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
pub fn create_context(_: &Device) -> Option<Context> {
    None
}

pub fn create_from_gl_buffer<T>(ctx: &Context, buf: gl::types::GLuint, flags: cl_mem_flags) -> OpenCL::mem::CLBuffer<T> {
    unsafe {
        let mut status = 0;
        let mem = clCreateFromGLBuffer(ctx.ctx, flags, buf, &mut status);
        assert!(status == 0);
        OpenCL::mem::CLBuffer{cl_buffer: mem}
    }
}