use gl;
use glfw;

use core;

pub struct FrameBuffer {
    id: uint,
    width: uint,
    height: uint
}

pub struct DrawTarget {
    width: uint,
    height: uint
}

impl core::DrawTarget for DrawTarget {
    fn draw(&mut self, t: core::DrawType) {}
}

impl core::DrawSize for DrawTarget {
    fn size(&self) -> (uint, uint)
    {
        (self.width, self.height)
    }
}

impl core::DrawSize for FrameBuffer {
    fn size(&self) -> (uint, uint)
    {
        (self.width, self.height)
    }
}

impl core::FrameBuffer for FrameBuffer {
    fn viewport(&mut self, offset: (uint, uint), size: (uint, uint), f: &fn(&mut core::DrawTarget))
    {
        let (w, h) = size;
        let (x, y) = offset;
        let mut draw_target = DrawTarget {
            width: w,
            height: h
        };

        let w = w as i32;
        let h = h as i32;
        let x = x as i32;
        let y = y as i32;

        let old = &mut [0i32, 0i32, 0i32, 0i32];
        unsafe {
            do old.as_mut_buf |ptr, _| {
                gl::GetIntegerv(gl::VIEWPORT, ptr);
            }
        }

        /* set new values */
        gl::Viewport(x, y, w, h);
        
        f(&mut draw_target as &mut core::DrawTarget);

        /* restore */
        gl::Viewport(old[0], old[1], old[2], old[3]);
    }
}