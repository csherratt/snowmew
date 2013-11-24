use gl;

use core;
use geometry::Geometry;
use shader::Shader;
use render::Context;

use cgmath;
use cgmath::ptr::*;

pub struct FrameBuffer {
    id: uint,
    width: uint,
    height: uint
}

pub struct DrawTarget {
    width: uint,
    height: uint
}


pub trait Uniforms {
    fn bind(&self, idx: i32);
}

impl Uniforms for cgmath::matrix::Mat4<f32> {
    fn bind(&self, idx: i32) {
        unsafe {
            gl::UniformMatrix4fv(idx, 1, gl::FALSE, self.ptr());
        }
    }
}

pub struct Texture {
    id: uint
}

impl core::DrawTarget for DrawTarget  {
    fn draw(&mut self, ctx: &mut Context, s: &Shader, g: &Geometry, uni: &[(i32, &Uniforms)], _: &[&Texture])
    {
        ctx.shader(s);
        for uni in uni.iter() {
            let (name, u) = *uni;
            u.bind(name);
        }
        g.draw(ctx);
    }
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
    fn viewport(&mut self,
                ctx: &mut Context,
                offset: (uint, uint), size: (uint, uint),
                f: &fn(&mut core::DrawTarget, ctx: &mut Context))
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

        /* set new values */
        let (old_offset, old_size) = ctx.get_viewport();
        ctx.viewport((x, y), (w, h));

        f(&mut draw_target as &mut core::DrawTarget, ctx);

        /* restore */
        ctx.viewport(old_offset, old_size);
    }
}