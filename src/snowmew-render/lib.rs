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
//use drawlist::Drawlist;
use drawlist::{ObjectCull, Expand};

use cgmath::matrix::{Mat4, ToMat4, Matrix};
use cgmath::vector::{Vec4, Vector};

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

        let mut list = Expand::new(
                        ObjectCull::new(
                            self.db.current.walk_drawables(scene), projection.clone()
                        ),
                        projection.clone(),
                        &self.db
                    );

        for (_, mat, geo, _, shader) in list {
            //shader.bind();
            //vb.bind();
            shader.set_position(&mat);
            //shader.set_projection(&projection);

            unsafe {
                gl::DrawElements(gl::TRIANGLES_ADJACENCY, geo.count as i32, gl::UNSIGNED_INT, ptr::null());
            }
        }
    }
}