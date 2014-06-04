use sync::{TaskPool, Arc};
use time::precise_time_s;

use OpenCL::hl::{CommandQueue, Context, Device};
use OpenCL::mem::Buffer;
use cgmath::matrix::{Matrix4, Matrix};
use cgmath::array::Array2;
use gl;


use position::{Positions, PositionData};
use graphics::{Graphics, GraphicsData};
use snowmew::common::{Common, CommonData};

use db::GlState;
use {Config, RenderData};
use material::MaterialBuffer;
use light::LightsBuffer;
use model::ModelInfoBuffer;
use matrix::MatrixBuffer;
use command::CommandBuffer;

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
    fn cull(&mut self, db: &GlState, view: &Matrix4<f32>, projection: &Matrix4<f32>);

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
    command: CommandBuffer,

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
            command: CommandBuffer::new(cfg),
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
        self.command.map();

    }

    fn setup_compute(~self, db: &RenderData, tp: &mut TaskPool<Sender<Box<Drawlist:Send>>>) {
        let DrawlistInstanced {
            data: _,
            size: size,
            materials: materials,
            lights: lights,
            model: model,
            matrix: matrix,
            command: command,
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

        let db4 = data.clone();
        let (sender, receiver4) = channel();
        tp.execute(proc(_) {
            let db = db4;
            let mut command = command;
            command.build(&db);
            sender.send(command);
        });

        tp.execute(proc(ch) {
            ch.send(
                match (receiver0.recv(), receiver1.recv(),
                       receiver2.recv(), receiver3.recv(),
                       receiver4.recv()) {
                    (matrix, model, lights, materials, command) => {
                        box DrawlistInstanced {
                            matrix: matrix,
                            materials: materials,
                            lights: lights,
                            model: model,
                            command: command,

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
        self.command.unmap();
    }

    fn cull(&mut self, db: &GlState, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        self.command.cull(self.model.id(), self.matrix.id(), db, &projection.mul_m(view));
    }

    fn render(&mut self, db: &GlState, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        let shader = db.flat_instance_shader.as_ref().unwrap();
        shader.bind();

        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);

        unsafe {
            gl::UniformMatrix4fv(shader.uniform("mat_proj"), 1, gl::FALSE, projection.ptr());
            gl::UniformMatrix4fv(shader.uniform("mat_view"), 1, gl::FALSE, view.ptr());    
        }
        
        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 4, self.model.id());
        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 5, self.matrix.id());

        let instance_offset = shader.uniform("instance_offset");        

        gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, self.command.id());
        gl::Uniform1i(instance_offset, 0);
        for b in self.command.batches().iter() {
            let vbo = db.vertex.find(&b.vbo()).expect("failed to find vertex buffer");
            vbo.bind();
            unsafe {
                gl::MultiDrawElementsIndirect(
                    gl::TRIANGLES,
                    gl::UNSIGNED_INT,
                    b.offset(),
                    b.drawcount(),
                    b.stride()
                );
            }
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
