#[crate_id = "github.com/csherratt/snowmew#snowmew-render:0.1"];
#[license = "ASL2"];
#[crate_type = "lib"];
#[comment = "A game engine in rust"];

extern mod std;
extern mod glfw;
extern mod cgmath;
extern mod snowmew;
extern mod cow;
extern mod gl;

use std::ptr;
use cgmath::matrix::{ToMat4, Matrix};
use drawlist::Drawlist;

mod db;
mod shader;
mod vertex_buffer;
mod drawlist;

pub struct RenderManager {
    //window: glfw::Window,
    db: db::Graphics
}

impl RenderManager
{
    pub fn new(_: &glfw::Window, db: snowmew::core::Database) -> RenderManager
    {
        // todo move!
        gl::Enable(gl::SCISSOR_TEST);
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::LINE_SMOOTH);
        gl::Enable(gl::BLEND);
        gl::CullFace(gl::BACK);

        RenderManager {
            //window: window,
            db: db::Graphics::new(db)
        }
    }

    pub fn load(&mut self)
    {
        self.db.load();
    }

    pub fn update(&mut self, db: snowmew::core::Database)
    {
        self.db.update(db);
    }

    pub fn render(&mut self, scene: i32, camera: i32)
    {
        let projection = cgmath::projection::perspective(
            cgmath::angle::deg(72f32), 1024f32/768f32, 0.01f32, 10000f32
        );

        let camera = self.db.current.location(camera).unwrap();

        let projection = projection.mul_m(&camera.get().to_mat4());

        let mut last_vb = 0;
        let mut last_shader = 0;

        let mut dl = Drawlist::create(&self.db, scene, projection.clone());

        //dl.sort(projection.clone());

        for (_, dat) in dl.iter() {
            let geo = self.db.current.geometry(dat.draw.geometry).unwrap();
            let vb = self.db.vertex.find(&geo.vb).unwrap();
            let shader = self.db.shaders.find(&dat.draw.shader).unwrap();
         
            if last_vb != geo.vb {
                vb.bind();
                last_vb = geo.vb;
            }
            if last_shader != dat.draw.shader {
                shader.bind();
                last_shader = dat.draw.shader;
                shader.set_projection(&projection);
            }
            shader.set_position(&dat.mat);

            unsafe {
                gl::DrawElements(gl::TRIANGLES_ADJACENCY, geo.count as i32, gl::UNSIGNED_INT, ptr::null());
            }
        }
    }
}