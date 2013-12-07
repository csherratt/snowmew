use geometry::Geometry;
use shader::Shader;
use coregl::{Uniforms, Texture};
use coregl;
use db;
use render::Context;

use cgmath::vector::*;

pub trait DrawSize {
    fn size(&self) -> (uint, uint);
}

pub trait DrawTarget: DrawSize {
    fn draw(&self, ctx: &mut Context, &Shader, &Geometry, &[(i32, &Uniforms)], &[&Texture]);
}

pub trait FrameBuffer: DrawSize {
    fn viewport(&self, ctx: &mut Context, offset :(uint, uint), size :(uint, uint), f: |&mut DrawTarget, ctx: &mut Context|);
}

pub struct FrameInfo {
    count: uint,  /* unique frame identifier */
    time: f64,    /* current time in seconds */
    delta: f64,   /* time from last frame */
}

pub trait Object  {
    fn draw(&self, ren: &Database, ctx: &mut Context, frame: &FrameInfo, target: &mut DrawTarget);
}

pub struct Database {
    priv objects: ~[~Object],
    priv geometry: ~[Geometry],
    priv shaders: ~[Shader]   
}

impl Database {
    fn new() -> Database
    {
        Database {
            objects: ~[],
            geometry: ~[],
            shaders: ~[]
        }
    }


    fn object_mut<'a>(&'a mut self, id: uint) -> &'a mut ~Object
    {
        &mut self.objects[id]
    }

    fn shader_mut<'a>(&'a mut self, id: uint) -> &'a mut Shader
    {
        &mut self.shaders[id]
    }

    fn geometry_mut<'a>(&'a mut self, id: uint) -> &'a mut Geometry
    {
        &mut self.geometry[id]
    }

    fn object<'a>(&'a self, id: uint) -> &'a ~Object
    {
        &self.objects[id]
    }

    fn shader<'a>(&'a self, id: uint) -> &'a Shader
    {
        &self.shaders[id]
    }

    fn geometry<'a>(&'a self, id: uint) -> &'a Geometry
    {
        &self.geometry[id]
    }
}

pub struct Position {
    position: Vec3<f32>,
    rotation: Vec3<f32>,
    scale:    f32,
    parent:   i32
}

pub struct Render  {
    priv fb: ~FrameBuffer,
    priv root: uint,
    priv ctx: Context,
    priv db: Database
}

impl Render {
    pub fn new() -> Render
    {
        Render {
            fb: ~coregl::FrameBuffer {
                width: 800,
                height: 600,
                id: 0
            } as ~FrameBuffer,
            root: 0,
            ctx: Context::new(),
            db: Database::new()
        }
    }

    pub fn draw(&mut self, fi: &FrameInfo) {
        let (w, h) = self.fb.size();
        self.fb.viewport(&mut self.ctx, (0, 0), (w, h), |viewport, ctx| {
            let root = self.db.object(self.root);
            root.draw(&self.db, ctx, fi, viewport);  
        });
    }

}
