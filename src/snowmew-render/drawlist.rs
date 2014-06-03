use sync::{TaskPool, Arc};
use libc::{c_void};
use time::precise_time_s;

use OpenCL::hl::{CommandQueue, Context, Device};
use OpenCL::mem::{Buffer};
use cgmath::matrix::Matrix4;
use cgmath::array::Array2;
use gl;
use gl::types::GLint;

use snowmew::common::{ObjectKey};

use position::{Positions, PositionData};
use graphics::{Graphics, GraphicsData};
use snowmew::common::{Common, CommonData};

use db::GlState;
use {Config, RenderData};
use material::MaterialBuffer;
use light::LightsBuffer;
use model::ModelInfoBuffer;
use matrix::MatrixBuffer;

pub trait Drawlist: RenderData {
    // This is done on the OpenGL thread, this will map and setup
    // any OpenGL objects that are required for setup to start/
    fn setup_begin(&mut self);

    // This is performed on a worker thread, the worker thread can copy
    // data from the scene graph into the any mapped buffers. This can also
    // spawn multiple workers. One of the threads must send the drawlist
    // back to the server
    fn setup_compute(~self, db: &RenderData, tp: &mut TaskPool<Sender<Box<Drawlist:Send>>>);

    // setup on the OpenGL thread, this will unmap and sync anything that
    // is needed to be done
    fn setup_complete(&mut self, db: &mut GlState, cfg: &Config);

    // setup is complete, render. This is done on the OpenGL thread.
    fn render(&mut self, db: &GlState, view: &Matrix4<f32>, projection: &Matrix4<f32>);

    // get materials
    fn model_buffer(&self) -> u32;
    fn material_buffer(&self) -> u32;
    fn lights_buffer(&self) -> u32;
    fn start_time(&self) -> f64;
}

impl Common for DrawlistInstanced {
    fn get_common<'a>(&'a self) -> &'a CommonData { &self.data.common }
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData { &mut self.data.common }
}

impl Graphics for DrawlistInstanced {
    fn get_graphics<'a>(&'a self) -> &'a GraphicsData { &self.data.graphics }
    fn get_graphics_mut<'a>(&'a mut self) -> &'a mut GraphicsData { &mut self.data.graphics }
}

impl Positions for DrawlistInstanced {
    fn get_position<'a>(&'a self) -> &'a PositionData { &self.data.position }
    fn get_position_mut<'a>(&'a mut self) -> &'a mut PositionData { &mut self.data.position }
}

impl RenderData for DrawlistInstanced {}

#[deriving(Clone)]
struct DrawlistGraphicsData {
    common: CommonData,
    graphics: GraphicsData,
    position: PositionData,    
}

impl Common for DrawlistGraphicsData {
    fn get_common<'a>(&'a self) -> &'a CommonData { &self.common }
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData { &mut self.common }
}

impl Graphics for DrawlistGraphicsData {
    fn get_graphics<'a>(&'a self) -> &'a GraphicsData { &self.graphics }
    fn get_graphics_mut<'a>(&'a mut self) -> &'a mut GraphicsData { &mut self.graphics }
}

impl Positions for DrawlistGraphicsData {
    fn get_position<'a>(&'a self) -> &'a PositionData { &self.position }
    fn get_position_mut<'a>(&'a mut self) -> &'a mut PositionData { &mut self.position }
}

impl RenderData for DrawlistGraphicsData {}

pub struct DrawlistInstanced {
    data: DrawlistGraphicsData,

    materials: MaterialBuffer,
    lights: LightsBuffer,
    model: ModelInfoBuffer,
    matrix: MatrixBuffer,

    size: uint,
    start: f64
}

impl DrawlistInstanced {
    pub fn from_config(cfg: &Config,
                       cl: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) -> DrawlistInstanced {

        DrawlistInstanced {
            data: DrawlistGraphicsData {
                common: CommonData::new(),
                graphics: GraphicsData::new(),
                position: PositionData::new()
            },
            size: cfg.max_size(),
            materials: MaterialBuffer::new(512),
            lights: LightsBuffer::new(),
            model: ModelInfoBuffer::new(cfg),
            matrix: MatrixBuffer::new(cfg, cl),
            start: 0.
        }
    }
}

impl Drawlist for DrawlistInstanced {
    fn setup_begin(&mut self) {
        self.materials.map();
        self.lights.map();
        self.model.map();
        self.matrix.map();

    }

    fn setup_compute(~self, db: &RenderData, tp: &mut TaskPool<Sender<Box<Drawlist:Send>>>) {
        let DrawlistInstanced {
            data: _,
            size: size,
            materials: materials,
            lights: lights,
            model: model,
            matrix: matrix,
            start: _
        } = *self;

        let data = DrawlistGraphicsData {
            common: db.get_common().clone(),
            graphics: db.get_graphics().clone(),
            position: db.get_position().clone()
        };

        let start = precise_time_s();
        let db0 = data.clone();
        let (sender, receiver0) = channel();
        tp.execute(proc(_) {
            let db = db0;
            let mut matrix = matrix;
            matrix.build(&db);
            sender.send(matrix)
        });

        let db1 = data.clone();
        let (sender, receiver1) = channel();
        tp.execute(proc(_) {
            let db = db1;
            let mut model = model;
            model.build(&db);
            sender.send(model);
        });

        let db2 = data.clone();
        let (sender, receiver2) = channel();
        tp.execute(proc(_) {
            let db = db2;
            let mut lights = lights;
            lights.build(&db);
            sender.send(lights);
        });

        let db3 = data.clone();
        let (sender, receiver3) = channel();
        tp.execute(proc(_) {
            let db = db3;
            let mut materials = materials;
            materials.build(&db);
            sender.send(materials);
        });

        tp.execute(proc(ch) {
            ch.send(
                match (receiver0.recv(), receiver1.recv(),
                       receiver2.recv(), receiver3.recv()) {
                    (matrix, model, lights, materials) => {
                        box DrawlistInstanced {
                            matrix: matrix,
                            materials: materials,
                            lights: lights,
                            model: model,

                            // other
                            size: size,
                            start: start,
                            data: data
                        } as Box<Drawlist:Send>
                    }
                }
            );
        });
    }

    fn setup_complete(&mut self, db: &mut GlState, cfg: &Config) {
        let data = self.data.clone();
        db.load(&data, cfg);
        self.materials.unmap();
        self.lights.unmap();
        self.model.unmap();
        self.matrix.unmap();
    }

    fn render(&mut self, db: &GlState, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        let shader = db.flat_instance_shader.unwrap();
        shader.bind();

        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);

        unsafe {
            gl::UniformMatrix4fv(shader.uniform("mat_proj"), 1, gl::FALSE, projection.ptr());
            gl::UniformMatrix4fv(shader.uniform("mat_view"), 1, gl::FALSE, view.ptr());

            let text = self.matrix.ids();
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_BUFFER, text[0]);
            gl::Uniform1i(shader.uniform("mat_model0"), 0);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_BUFFER, text[1]);
            gl::Uniform1i(shader.uniform("mat_model1"), 1);

            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_BUFFER, text[2]);
            gl::Uniform1i(shader.uniform("mat_model2"), 2);

            gl::ActiveTexture(gl::TEXTURE3);
            gl::BindTexture(gl::TEXTURE_BUFFER, text[3]);
            gl::Uniform1i(shader.uniform("mat_model3"), 3);

            gl::ActiveTexture(gl::TEXTURE4);
            gl::BindTexture(gl::TEXTURE_BUFFER, self.model.id());
            gl::Uniform1i(shader.uniform("info"), 4);
        }

        let mut range = (0u, 0u);
        let mut last_geo: Option<u32> = None;
        let mut bound_vbo = None;
        let instance_offset = shader.uniform("instance_offset");        

        let instance_draw = |draw_geo: ObjectKey, offset: uint, len: uint| {
            let draw_geo = self.geometry(draw_geo).expect("geometry not found");
            if Some(draw_geo.vb) != bound_vbo {
                let draw_vbo = db.vertex.find(&draw_geo.vb).expect("vbo not found");
                draw_vbo.bind();
                bound_vbo = Some(draw_geo.vb);
            }
            unsafe {
                gl::Uniform1i(instance_offset, offset as GLint);
                gl::DrawElementsInstanced(gl::TRIANGLES,
                    draw_geo.count as GLint,
                    gl::UNSIGNED_INT,
                    (draw_geo.offset * 4) as *c_void,
                    len as GLint
                );
            }
        };

        for (idx, (_, draw)) in self.drawable_iter().enumerate() {
            if last_geo.is_some() {
                let (start, end) = range;
                if last_geo.unwrap() == draw.geometry {
                    range = (start, idx);
                } else {
                    instance_draw(last_geo.unwrap(), start, end - start + 1);
                    range = (idx, idx);
                    last_geo = Some(draw.geometry);
                }
            } else {
                range = (idx, idx);
                last_geo = Some(draw.geometry);
            }
        }

        if last_geo.is_some() {
            let (start, end) = range;
            instance_draw(last_geo.unwrap(), start, end - start + 1);
        }

    }

    fn model_buffer(&self) -> u32 {
        self.model.id()
    }

    fn material_buffer(&self) -> u32 {
        self.materials.id()
    }

    fn lights_buffer(&self) -> u32 {
        self.lights.id()
    }

    fn start_time(&self) -> f64 { self.start }
}

pub fn create_drawlist(cfg: &Config,
                       cl: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) -> Box<Drawlist:Send> {
    box DrawlistInstanced::from_config(cfg, cl) as Box<Drawlist:Send>
}
