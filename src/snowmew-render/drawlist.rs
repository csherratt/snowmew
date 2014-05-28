use std::mem;
use std::ptr;
use std::slice::raw::mut_buf_as_slice;
use sync::{TaskPool, Arc};
use libc::{c_void};
use collections::treemap::TreeMap;
use time::precise_time_s;

use cow::join::join_maps;

use OpenCL::hl::{CommandQueue, Context, Device, Event, EventList};
use OpenCL::mem::{Buffer, CLBuffer};
use OpenCL::CL::CL_MEM_READ_WRITE;
use cgmath::matrix::Matrix4;
use cgmath::vector::Vector4;
use cgmath::ptr::Ptr;
use gl;
use gl::types::{GLint, GLuint, GLsizeiptr};
use gl_cl;
use gl_cl::AcquireRelease;

use graphics::material::Material;
use snowmew::common::{ObjectKey};
use position::{ComputedPosition, CalcPositionsCl, MatrixManager};

use position::{Positions, PositionData};
use graphics::{Graphics, GraphicsData};
use snowmew::common::{Common, CommonData};

use db::GlState;
use {Config, RenderData};
use material::MaterialBuffer;

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
    fn materials(&self) -> Vec<Material>;
    fn material_buffer(&self) -> u32;
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

fn map_material_to_ids(db: &RenderData) ->(TreeMap<ObjectKey, u32>, TreeMap<u32, ObjectKey>) {
    let mut material_to_id = TreeMap::new();
    let mut id_to_material = TreeMap::new();

    for (id, (key, _)) in db.material_iter().enumerate() {
        material_to_id.insert(*key, (id+1) as u32);
        id_to_material.insert((id+1) as u32, *key);
    }

    (material_to_id, id_to_material)
}

impl RenderData for DrawlistGraphicsData {}

pub struct DrawlistInstanced {
    data: DrawlistGraphicsData,

    computed_position: Option<ComputedPosition>,

    materials: MaterialBuffer,
    material_to_id: TreeMap<ObjectKey, u32>,
    id_to_material: TreeMap<u32, ObjectKey>,

    size: uint,

    // one array for each component
    model_matrix: [GLuint, ..4],
    model_info: GLuint,

    text_model_matrix: [GLuint, ..4],
    text_model_info: GLuint,

    ptr_model_matrix: [*mut Vector4<f32>, ..4],
    ptr_model_info: *mut (u32, u32, u32, u32),

    event: Option<Event>,
    cl: Option<(CalcPositionsCl, Arc<CommandQueue>, [CLBuffer<Vector4<f32>>, ..4])>,

    start: f64
}

impl DrawlistInstanced {
    pub fn from_config(cfg: &Config,
                       cl: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) -> DrawlistInstanced {
        let buffer = &mut [0, 0, 0, 0, 0];
        let texture = &mut [0, 0, 0, 0, 0];

        unsafe {
            gl::GenBuffers(buffer.len() as i32, buffer.unsafe_mut_ref(0));
            gl::GenTextures(buffer.len() as i32, texture.unsafe_mut_ref(0));
      
            for i in range(0u, 4) {
                gl::BindBuffer(gl::TEXTURE_BUFFER, buffer[i]);
                gl::BindTexture(gl::TEXTURE_BUFFER, texture[i]);
                gl::TexBuffer(gl::TEXTURE_BUFFER, gl::RGBA32F, buffer[i]);
                gl::BufferData(gl::TEXTURE_BUFFER,
                               (mem::size_of::<Vector4<f32>>()*cfg.max_size()) as GLsizeiptr,
                               ptr::null(), gl::DYNAMIC_DRAW);
            }

            gl::BindBuffer(gl::TEXTURE_BUFFER, buffer[4]);
            gl::BindTexture(gl::TEXTURE_BUFFER, texture[4]);
            gl::TexBuffer(gl::TEXTURE_BUFFER, gl::RGBA32UI, buffer[4]);
            assert!(0 == gl::GetError());
            gl::BufferData(gl::TEXTURE_BUFFER,
                           (mem::size_of::<(u32, u32, u32, u32)>()*cfg.max_size()) as GLsizeiptr,
                           ptr::null(), gl::DYNAMIC_DRAW);
            assert!(0 == gl::GetError());
        }

        let clpos = match cl {
            Some((ctx, cq, dev)) => {
                let calc = CalcPositionsCl::new(ctx.deref(), dev.deref());
                let buffers = [gl_cl::create_from_gl_buffer(ctx.deref(), buffer[0], CL_MEM_READ_WRITE),
                               gl_cl::create_from_gl_buffer(ctx.deref(), buffer[1], CL_MEM_READ_WRITE),
                               gl_cl::create_from_gl_buffer(ctx.deref(), buffer[2], CL_MEM_READ_WRITE),
                               gl_cl::create_from_gl_buffer(ctx.deref(), buffer[3], CL_MEM_READ_WRITE)];

                Some((calc, cq, buffers))
            },
            None => None
        };

        DrawlistInstanced {
            data: DrawlistGraphicsData {
                common: CommonData::new(),
                graphics: GraphicsData::new(),
                position: PositionData::new()
            },
            computed_position: None,
            size: cfg.max_size(),
            model_matrix: [buffer[0], buffer[1], buffer[2], buffer[3]],
            model_info: buffer[4],
            text_model_matrix: [texture[0], texture[1], texture[2], texture[3]],
            text_model_info: texture[4],
            ptr_model_matrix: [ptr::mut_null(), ptr::mut_null(), ptr::mut_null(), ptr::mut_null()],
            ptr_model_info: ptr::mut_null(),
            material_to_id: TreeMap::new(),
            id_to_material: TreeMap::new(),
            materials: MaterialBuffer::new(2048),
            cl: clpos,
            event: None,
            start: 0.
        }
    }
}

struct GLMatrix<'r> {
    x: &'r mut [Vector4<f32>],
    y: &'r mut [Vector4<f32>],
    z: &'r mut [Vector4<f32>],
    w: &'r mut [Vector4<f32>]
}

impl<'r> MatrixManager for GLMatrix<'r> {
    fn set(&mut self, idx: uint, mat: Matrix4<f32>) {
        assert!(idx < self.x.len());
        unsafe {
            self.x.unsafe_set(idx, mat.x);
            self.y.unsafe_set(idx, mat.y);
            self.z.unsafe_set(idx, mat.z);
            self.w.unsafe_set(idx, mat.w);
        }
    }

    fn get(&self, idx: uint) -> Matrix4<f32> {
        assert!(idx < self.x.len());
        unsafe {
            Matrix4 {
                x: *self.x.unsafe_ref(idx),
                y: *self.y.unsafe_ref(idx),
                z: *self.z.unsafe_ref(idx),
                w: *self.w.unsafe_ref(idx)
            }
        }
    }
}


impl Drawlist for DrawlistInstanced {
    fn setup_begin(&mut self) {
        self.materials.map();
        match self.cl {
            None => {
                for i in range(0u, 4) {
                    gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_matrix[i]);
                    self.ptr_model_matrix[i] = gl::MapBufferRange(gl::TEXTURE_BUFFER, 0, 
                            (mem::size_of::<Vector4<f32>>()*self.size) as GLsizeiptr,
                            gl::MAP_WRITE_BIT | gl::MAP_READ_BIT
                    ) as *mut Vector4<f32>;
                    assert!(0 == gl::GetError());
                }                
            }
            Some((_, ref cq, ref buf)) => {
                cq.acquire_gl_objects(buf.as_slice(), ()).wait()
            }
        }

        gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_info);
        self.ptr_model_info = gl::MapBufferRange(gl::TEXTURE_BUFFER, 0, 
                (mem::size_of::<(u32, u32, u32, u32)>()*self.size) as GLsizeiptr,
                gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
        ) as *mut (u32, u32, u32, u32);
        assert!(0 == gl::GetError());
    }

    fn setup_compute(~self, db: &RenderData, tp: &mut TaskPool<Sender<Box<Drawlist:Send>>>) {
        let DrawlistInstanced {
            data: _,
            cl: cl,
            size: size,
            computed_position: _,
            material_to_id: material_to_id,
            id_to_material: id_to_material,
            model_matrix: model_matrix,
            model_info: model_info,
            text_model_matrix: text_model_matrix,
            text_model_info: text_model_info,
            ptr_model_matrix: ptr_model_matrix,
            ptr_model_info: ptr_model_info,
            materials: materials,
            event: _,
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
            let mut cl = cl;
            let evt = unsafe {
                match cl {
                    None => {
                        mut_buf_as_slice(ptr_model_matrix[0], size, |mat0| {
                        mut_buf_as_slice(ptr_model_matrix[1], size, |mat1| {
                        mut_buf_as_slice(ptr_model_matrix[2], size, |mat2| {
                        mut_buf_as_slice(ptr_model_matrix[3], size, |mat3| {
                            let mut mat = GLMatrix {
                                x: mat0, y: mat1, z: mat2, w: mat3
                            };
                            db.write_positions(&mut mat);
                            None
                        })})})})               
                    }
                    Some((ref mut ctx, ref cq, ref buf)) => {
                        let evt = db.write_positions_cl(cq.deref(), ctx, buf);
                        Some(evt)
                    }
                }
            };

            let position = db.compute_positions();
            sender.send((position, evt, ptr_model_matrix, cl))
        });

        let db1 = data.clone();
        let (sender, receiver1) = channel();
        tp.execute(proc(_) {
            let db = db1;
            let mut material_to_id = material_to_id;
            let mut id_to_material = id_to_material;
            let position = db.compute_positions();

            material_to_id.clear();
            id_to_material.clear();

            for (id, (key, _)) in db.material_iter().enumerate() {
                material_to_id.insert(*key, id as u32);
                id_to_material.insert(id as u32, *key);
            }

            unsafe {
                mut_buf_as_slice(ptr_model_info, size, |info| {
                    for (idx, (id, (draw, pos))) in join_maps(db.drawable_iter(), db.location_iter()).enumerate() {
                        info[idx] = (id.clone(),
                                     position.get_loc(*pos) as u32,
                                     material_to_id.find(&draw.material).unwrap().clone(),
                                     0u32);
                    }
                });
            }
            sender.send((material_to_id, id_to_material, ptr_model_info));
        });

        tp.execute(proc(ch) {
            ch.send(
                match (receiver0.recv(), receiver1.recv()) {
                    ((computed_position, event, ptr_model_matrix, cl),
                     (material_to_id, id_to_material, ptr_model_info)) => {
                        box DrawlistInstanced {
                            // from task 0
                            computed_position: Some(computed_position),
                            event: event,
                            ptr_model_matrix: ptr_model_matrix,
                            cl: cl,

                            // fromt task 1
                            materials: materials,
                            material_to_id: material_to_id,
                            id_to_material: id_to_material,
                            ptr_model_info: ptr_model_info,

                            // other
                            size: size,
                            model_matrix: model_matrix,
                            model_info: model_info,
                            text_model_info: text_model_info,
                            text_model_matrix: text_model_matrix,
                            start: start,
                            data: data
                        } as Box<Drawlist:Send>
                    }
                }
            );
        });
    }

    fn setup_complete(&mut self, db: &mut GlState, cfg: &Config) {
        let event = self.event.take();
        match (&self.cl, event) {
            (&None, None) => {
                for i in range(0u, 4) {
                    gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_matrix[i]);
                    gl::UnmapBuffer(gl::TEXTURE_BUFFER);
                    assert!(0 == gl::GetError());
                    self.ptr_model_matrix[i] = ptr::mut_null();
                }
            }
            (&Some((_, ref cq, ref buf)), Some(ref event)) => {
                cq.release_gl_objects(buf.as_slice(), event).wait();
            }
            _ => fail!("expected both an event and a queue")
        }

        gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_info);
        gl::UnmapBuffer(gl::TEXTURE_BUFFER);
        assert!(0 == gl::GetError());
        self.ptr_model_info = ptr::mut_null();

        let data = self.data.clone();
        db.load(&data, cfg);
        self.materials.build(&data, db);
        self.materials.unmap();
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

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_BUFFER, self.text_model_matrix[0]);
            gl::Uniform1i(shader.uniform("mat_model0"), 0);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_BUFFER, self.text_model_matrix[1]);
            gl::Uniform1i(shader.uniform("mat_model1"), 1);

            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_BUFFER, self.text_model_matrix[2]);
            gl::Uniform1i(shader.uniform("mat_model2"), 2);

            gl::ActiveTexture(gl::TEXTURE3);
            gl::BindTexture(gl::TEXTURE_BUFFER, self.text_model_matrix[3]);
            gl::Uniform1i(shader.uniform("mat_model3"), 3);

            gl::ActiveTexture(gl::TEXTURE4);
            gl::BindTexture(gl::TEXTURE_BUFFER, self.text_model_info);
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

    fn material_buffer(&self) -> u32 {
        self.materials.id()
    }

    fn materials(&self) -> Vec<Material> {
        let mut mats = Vec::new();
        for (_, key) in self.id_to_material.iter() {
            //println!("{:?}", self.material(*key).unwrap().clone());
            mats.push(self.material(*key).unwrap().clone());
        }
        mats
    }

    fn start_time(&self) -> f64 { self.start }
}

pub struct DrawlistSimple {
    data: DrawlistGraphicsData,
    material_to_id: TreeMap<ObjectKey, u32>,
    id_to_material: TreeMap<u32, ObjectKey>,
    start: f64 
}

impl Common for DrawlistSimple {
    fn get_common<'a>(&'a self) -> &'a CommonData { &self.data.common }
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData { &mut self.data.common }
}

impl Graphics for DrawlistSimple {
    fn get_graphics<'a>(&'a self) -> &'a GraphicsData { &self.data.graphics }
    fn get_graphics_mut<'a>(&'a mut self) -> &'a mut GraphicsData { &mut self.data.graphics }
}

impl Positions for DrawlistSimple {
    fn get_position<'a>(&'a self) -> &'a PositionData { &self.data.position }
    fn get_position_mut<'a>(&'a mut self) -> &'a mut PositionData { &mut self.data.position }
}

impl RenderData for DrawlistSimple {}

impl DrawlistSimple {
    pub fn from_config(_: &Config,
                       _: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) -> DrawlistSimple {
        DrawlistSimple {
            data: DrawlistGraphicsData {
                common: CommonData::new(),
                graphics: GraphicsData::new(),
                position: PositionData::new()
            },
            material_to_id: TreeMap::new(),
            id_to_material: TreeMap::new(),
            start: 0.
        }
    }
}

impl Drawlist for DrawlistSimple {
    fn setup_begin(&mut self) {}

    fn setup_compute(~self, db: &RenderData, tp: &mut TaskPool<Sender<Box<Drawlist:Send>>>) {
        let mut s = *self;
        s.start = precise_time_s();
        s.data = DrawlistGraphicsData {
            common: db.get_common().clone(),
            graphics: db.get_graphics().clone(),
            position: db.get_position().clone()
        };
        tp.execute(proc(send) {
            let mut s = s;
            let (material_to_id, id_to_material) = map_material_to_ids(&s);
            s.material_to_id = material_to_id;
            s.id_to_material = id_to_material;
            send.send(box s as Box<Drawlist:Send>)
        });
    }

    fn setup_complete(&mut self, db: &mut GlState, cfg: &Config) {
        db.load(self, cfg);
    }

    fn render(&mut self, db: &GlState, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        let shader = db.flat_shader.unwrap();
        shader.bind();

        let mat_model = shader.uniform("mat_model");
        let object_id = shader.uniform("object_id");
        let material_id = shader.uniform("material_id");

        unsafe {
            gl::UniformMatrix4fv(shader.uniform("mat_proj"), 1, gl::FALSE, projection.ptr());
            gl::UniformMatrix4fv(shader.uniform("mat_view"), 1, gl::FALSE, view.ptr());
        }

        for (id, draw) in self.drawable_iter() {   
            let draw_geo = self.geometry(draw.geometry).expect("geometry not found");
            let draw_vbo = db.vertex.find(&draw_geo.vb).expect("vertex not found");
            draw_vbo.bind();
            unsafe {
                let material = self.material_to_id.find(&draw.material).expect("material not found");
                let matrix = self.position(*id);
                gl::UniformMatrix4fv(mat_model, 1, gl::FALSE, matrix.ptr());
                gl::Uniform1ui(object_id, *id as u32);
                gl::Uniform1ui(material_id, *material);
                gl::DrawElements(gl::TRIANGLES,
                    draw_geo.count as GLint,
                    gl::UNSIGNED_INT,
                    (draw_geo.offset * 4) as *c_void
                );
            }
        }
    }

    fn material_buffer(&self) -> u32 {0}

    fn materials(&self) -> Vec<Material> {
        let mut mats = Vec::new();
        for (_, key) in self.id_to_material.iter() {
            mats.push(self.material(*key).unwrap().clone());
        }
        mats
    }

    fn start_time(&self) -> f64 { self.start }    
}

pub fn create_drawlist(cfg: &Config,
                       cl: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) -> Box<Drawlist:Send> {
    if cfg.instanced() {
        box DrawlistInstanced::from_config(cfg, cl) as Box<Drawlist:Send>
    } else {
        box DrawlistSimple::from_config(cfg, cl) as Box<Drawlist:Send>
    }
}
