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

use std::sync::TaskPool;
use sync::Arc;
use time::precise_time_s;
use libc::c_void;
use render_data::{Renderable, RenderData};

use opencl::hl::{CommandQueue, Context, Device};
use opencl::mem::Buffer;
use cgmath::{Matrix4, Matrix};
use cgmath::Array2;
use gl;

use position::{Positions, PositionData};
use graphics::{Graphics, GraphicsData};
use snowmew::common::{Common, CommonData};
use snowmew::ObjectKey;

use db::GlState;
use Config;
use material::MaterialBuffer;
use light::LightsBuffer;
use model::{ModelInfoTextureBuffer, ModelInfoSSBOBuffer};
use matrix::{MatrixSSBOBuffer, MatrixTextureBuffer};
use command::{CommandBufferIndirect, CommandBufferEmulated};

pub trait Drawlist: Renderable {
    // This is done on the OpenGL thread, this will map and setup
    // any OpenGL objects that are required for setup to start/
    fn setup_begin(&mut self);

    // This is performed on a worker thread, the worker thread can copy
    // data from the scene graph into the any mapped buffers. This can also
    // spawn multiple workers. One of the threads must send the drawlist
    // back to the server
    fn setup_compute(self: Box<Self>, db: &Renderable, tp: &mut TaskPool<Sender<Box<Drawlist+Send>>>, scene: ObjectKey);

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

impl Common for DrawlistSSBOCompute {
    fn get_common<'a>(&'a self) -> &'a CommonData { &self.data.common }
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData { &mut self.data.common }
}

impl Graphics for DrawlistSSBOCompute {
    fn get_graphics<'a>(&'a self) -> &'a GraphicsData { &self.data.graphics }
    fn get_graphics_mut<'a>(&'a mut self) -> &'a mut GraphicsData { &mut self.data.graphics }
}

impl Positions for DrawlistSSBOCompute {
    fn get_position<'a>(&'a self) -> &'a PositionData { &self.data.position }
    fn get_position_mut<'a>(&'a mut self) -> &'a mut PositionData { &mut self.data.position }
}
impl Renderable for DrawlistSSBOCompute {
    fn get_render_data<'a>(&'a self) -> &'a RenderData { &self.data.render }
    fn get_render_data_mut<'a>(&'a mut self) -> &'a mut RenderData { &mut self.data.render } 
}

impl Common for DrawlistNoSSBO {
    fn get_common<'a>(&'a self) -> &'a CommonData { &self.data.common }
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData { &mut self.data.common }
}

impl Graphics for DrawlistNoSSBO {
    fn get_graphics<'a>(&'a self) -> &'a GraphicsData { &self.data.graphics }
    fn get_graphics_mut<'a>(&'a mut self) -> &'a mut GraphicsData { &mut self.data.graphics }
}

impl Positions for DrawlistNoSSBO {
    fn get_position<'a>(&'a self) -> &'a PositionData { &self.data.position }
    fn get_position_mut<'a>(&'a mut self) -> &'a mut PositionData { &mut self.data.position }
}

impl Renderable for DrawlistNoSSBO {
    fn get_render_data<'a>(&'a self) -> &'a RenderData { &self.data.render }
    fn get_render_data_mut<'a>(&'a mut self) -> &'a mut RenderData { &mut self.data.render } 
}

#[deriving(Clone)]
struct DrawlistGraphicsData {
    common: CommonData,
    graphics: GraphicsData,
    position: PositionData,
    render: RenderData
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

impl Renderable for DrawlistGraphicsData {
    fn get_render_data<'a>(&'a self) -> &'a RenderData { &self.render }
    fn get_render_data_mut<'a>(&'a mut self) -> &'a mut RenderData { &mut self.render } 
}

pub struct DrawlistNoSSBO {
    data: DrawlistGraphicsData,

    materials: MaterialBuffer,
    lights: LightsBuffer,
    model: ModelInfoTextureBuffer,
    matrix: MatrixTextureBuffer,
    command: CommandBufferEmulated,

    size: uint,
    start: f64,
    instanced_is_enabled: bool
}

impl DrawlistNoSSBO {
    pub fn from_config(cfg: &Config,
                       cl: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) -> DrawlistNoSSBO {

        DrawlistNoSSBO {
            data: DrawlistGraphicsData {
                common: CommonData::new(),
                graphics: GraphicsData::new(),
                position: PositionData::new(),
                render: RenderData::new()
            },
            size: cfg.max_size(),
            materials: MaterialBuffer::new(512),
            lights: LightsBuffer::new(),
            model: ModelInfoTextureBuffer::new(cfg),
            matrix: MatrixTextureBuffer::new(cfg, cl),
            command: CommandBufferEmulated::new(cfg),
            start: 0.,
            instanced_is_enabled: cfg.instanced()
        }
    }
}

impl Drawlist for DrawlistNoSSBO {
    fn setup_begin(&mut self) {
        self.materials.map();
        self.lights.map();
        self.model.map();
        self.matrix.map();
        self.command.map();
    }

    fn setup_compute(self: Box<DrawlistNoSSBO>, db: &Renderable, tp: &mut TaskPool<Sender<Box<Drawlist+Send>>>, scene: ObjectKey) {
        let s = *self;
        let DrawlistNoSSBO {
            data: _,
            size,
            materials,
            lights,
            model,
            command,
            matrix,
            instanced_is_enabled,
            start: _
        } = s;

        let data = DrawlistGraphicsData {
            common: db.get_common().clone(),
            graphics: db.get_graphics().clone(),
            position: db.get_position().clone(),
            render: db.get_render_data().clone()
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
            model.build(&db, scene);
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
            command.build(&db, scene, instanced_is_enabled);
            sender.send(command);
        });

        tp.execute(proc(ch) {
            ch.send(
                match (receiver0.recv(), receiver1.recv(),
                       receiver2.recv(), receiver3.recv(),
                       receiver4.recv()) {
                    (matrix, model, lights, materials, command) => {
                        box DrawlistNoSSBO {
                            matrix: matrix,
                            materials: materials,
                            lights: lights,
                            model: model,
                            command: command,

                            // other
                            size: size,
                            start: start,
                            data: data,
                            instanced_is_enabled: instanced_is_enabled
                        } as Box<Drawlist+Send>
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

    fn cull(&mut self, _: &GlState, _: &Matrix4<f32>, _: &Matrix4<f32>) {}

    fn render(&mut self, db: &GlState, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        let shader = db.geometry_no_ssbo.as_ref().unwrap();
        shader.bind();

        let mut base_index;
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);

            base_index = shader.uniform("base_index");

            gl::UniformMatrix4fv(shader.uniform("mat_proj"), 1, gl::FALSE, projection.ptr());
            gl::UniformMatrix4fv(shader.uniform("mat_view"), 1, gl::FALSE, view.ptr());

            let text = self.matrix.ids();
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_BUFFER, text[0]);
            gl::Uniform1i(shader.uniform("model_matrix0"), 0);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_BUFFER, text[1]);
            gl::Uniform1i(shader.uniform("model_matrix1"), 1);

            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_BUFFER, text[2]);
            gl::Uniform1i(shader.uniform("model_matrix2"), 2);

            gl::ActiveTexture(gl::TEXTURE3);
            gl::BindTexture(gl::TEXTURE_BUFFER, text[3]);
            gl::Uniform1i(shader.uniform("model_matrix3"), 3);

            gl::ActiveTexture(gl::TEXTURE4);
            gl::BindTexture(gl::TEXTURE_BUFFER, self.model.id());
            gl::Uniform1i(shader.uniform("info_buffer"), 4);
        }

        let cmds = self.command.commands();
        for b in self.command.batches().iter() {
            let vbo = db.vertex.find(&b.vbo()).expect("failed to find vertex buffer");
            vbo.bind();
            for d in range(b.offset_int(), b.drawcount() as uint +b.offset_int()) {
                unsafe {
                    gl::Uniform1i(base_index, cmds[d].base_instance as i32);
                    gl::DrawElementsInstanced(
                        gl::TRIANGLES,
                        cmds[d].count as i32,
                        gl::UNSIGNED_INT,
                        (cmds[d].first_index * 4) as *const c_void,
                        cmds[d].instrance_count as i32
                    );
                }
            }
        }
    }

    fn model_buffer(&self) -> u32 { self.model.id() }
    fn material_buffer(&self) -> u32 { self.materials.id() }
    fn lights_buffer(&self) -> u32 { self.lights.id() }
    fn start_time(&self) -> f64 { self.start }
}

pub struct DrawlistSSBOCompute {
    data: DrawlistGraphicsData,

    materials: MaterialBuffer,
    lights: LightsBuffer,
    model: ModelInfoSSBOBuffer,
    matrix: MatrixSSBOBuffer,
    command: CommandBufferIndirect,

    size: uint,
    start: f64,

    culling_is_enabled: bool,
    instanced_is_enabled: bool
}

impl DrawlistSSBOCompute {
    pub fn from_config(cfg: &Config,
                       cl: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) -> DrawlistSSBOCompute {

        DrawlistSSBOCompute {
            data: DrawlistGraphicsData {
                common: CommonData::new(),
                graphics: GraphicsData::new(),
                position: PositionData::new(),
                render: RenderData::new()
            },
            size: cfg.max_size(),
            materials: MaterialBuffer::new(512),
            lights: LightsBuffer::new(),
            model: ModelInfoSSBOBuffer::new(cfg),
            matrix: MatrixSSBOBuffer::new(cfg, cl),
            command: CommandBufferIndirect::new(cfg),
            start: 0.,
            culling_is_enabled: cfg.culling(),
            instanced_is_enabled: cfg.instanced()
        }
    }
}

impl Drawlist for DrawlistSSBOCompute {
    fn setup_begin(&mut self) {
        self.materials.map();
        self.lights.map();
        self.model.map();
        self.matrix.map();
        self.command.map();
    }

    fn setup_compute(self: Box<DrawlistSSBOCompute>, db: &Renderable, tp: &mut TaskPool<Sender<Box<Drawlist+Send>>>, scene: ObjectKey) {
        let s = *self;
        let DrawlistSSBOCompute {
            data: _,
            size,
            materials,
            lights,
            model,
            matrix,
            command,
            culling_is_enabled,
            instanced_is_enabled,
            start: _
        } = s;

        let data = DrawlistGraphicsData {
            common: db.get_common().clone(),
            graphics: db.get_graphics().clone(),
            position: db.get_position().clone(),
            render: db.get_render_data().clone()
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
            model.build(&db, scene);
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
            command.build(&db, scene, instanced_is_enabled);
            sender.send(command);
        });

        tp.execute(proc(ch) {
            ch.send(
                match (receiver0.recv(), receiver1.recv(),
                       receiver2.recv(), receiver3.recv(),
                       receiver4.recv()) {
                    (matrix, model, lights, materials, command) => {
                        box DrawlistSSBOCompute {
                            matrix: matrix,
                            materials: materials,
                            lights: lights,
                            model: model,
                            command: command,

                            // other
                            size: size,
                            start: start,
                            data: data,
                            culling_is_enabled: culling_is_enabled,
                            instanced_is_enabled: instanced_is_enabled
                        } as Box<Drawlist+Send>
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
        if self.culling_is_enabled {
            self.command.cull(self.model.id(), self.matrix.id(), db, &projection.mul_m(view));
        }
    }

    fn render(&mut self, db: &GlState, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        let shader = db.geometry_ssbo_drawid.as_ref().unwrap();
        shader.bind();

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);

            gl::UniformMatrix4fv(shader.uniform("mat_proj"), 1, gl::FALSE, projection.ptr());
            gl::UniformMatrix4fv(shader.uniform("mat_view"), 1, gl::FALSE, view.ptr());

            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 4, self.model.id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 5, self.matrix.id());

            gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, self.command.id());
            for b in self.command.batches().iter() {
                let vbo = db.vertex.find(&b.vbo()).expect("failed to find vertex buffer");
                vbo.bind();
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

    fn model_buffer(&self) -> u32 { self.model.id() }
    fn material_buffer(&self) -> u32 { self.materials.id() }
    fn lights_buffer(&self) -> u32 { self.lights.id() }
    fn start_time(&self) -> f64 { self.start }
}

pub fn create_drawlist(cfg: &Config,
                       cl: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) -> Box<Drawlist+Send> {
    if cfg.compute() && cfg.ssbo() {
        box DrawlistSSBOCompute::from_config(cfg, cl) as Box<Drawlist+Send>
    } else {
        box DrawlistNoSSBO::from_config(cfg, cl) as Box<Drawlist+Send>
    }
}
