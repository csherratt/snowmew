use std::mem;
use std::ptr;
use std::slice::raw::mut_buf_as_slice;
use libc::{c_void};
use collections::treemap::TreeMap;

use cow::join::join_maps;
use OpenCL::hl::{CommandQueue, Context, Device, EventList};
use OpenCL::mem::CLBuffer;
use OpenCL::CL::CL_MEM_READ_WRITE;
use sync::Arc;
use cgmath::matrix::Matrix4;
use cgmath::vector::Vector4;
use cgmath::ptr::Ptr;
use gl;
use gl::types::{GLint, GLuint, GLsizeiptr};
use gl_cl;

use graphics::material::Material;
use snowmew::common::{ObjectKey};
use position::{ComputedPosition, Positions, CalcPositionsCl, MatrixManager};
use graphics::Graphics;


use db::GlState;
use Config;

pub trait Drawlist {
    // done on the context manager before, Graphics is owned by
    // the draw list. If there was already a bound scene this
    // needs to be replaces with the current scene
    fn bind_scene(&mut self, db: GlState, scene: ObjectKey);

    // done first on an external thread
    fn setup_scene_async(&mut self);

    // setup on the render thread, called after setup_scene_async
    fn setup_scene(&mut self);

    // done many times on the render thread
    fn render(&mut self, camera: Matrix4<f32>);

    // get materials
    fn materials(&self) -> Vec<Material>;
}

pub struct DrawlistStandard {
    db: Option<GlState>,
    scene: ObjectKey,
    position: Option<ComputedPosition>,

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

    pub cl: Option<(CalcPositionsCl, Arc<CommandQueue>, [CLBuffer<Vector4<f32>>, ..4])>
}

impl DrawlistStandard {
    pub fn from_config(cfg: &Config,
                       cl: Option<(Arc<Context>, Arc<CommandQueue>, Arc<Device>)>) -> DrawlistStandard {
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

        DrawlistStandard {
            db: None,
            scene: 0,
            position: None,
            size: cfg.max_size(),
            model_matrix: [buffer[0], buffer[1], buffer[2], buffer[3]],
            model_info: buffer[4],
            text_model_matrix: [texture[0], texture[1], texture[2], texture[3]],
            text_model_info: texture[4],
            ptr_model_matrix: [ptr::mut_null(), ptr::mut_null(), ptr::mut_null(), ptr::mut_null()],
            ptr_model_info: ptr::mut_null(),
            material_to_id: TreeMap::new(),
            id_to_material: TreeMap::new(),
            cl: clpos
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


impl Drawlist for DrawlistStandard {
    fn bind_scene(&mut self, db: GlState, scene: ObjectKey) {
        self.db = Some(db);
        self.scene = scene;

        if self.cl.is_none() {
            for i in range(0u, 4) {
                gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_matrix[i]);
                self.ptr_model_matrix[i] = gl::MapBufferRange(gl::TEXTURE_BUFFER, 0, 
                        (mem::size_of::<Vector4<f32>>()*self.size) as GLsizeiptr,
                        gl::MAP_WRITE_BIT | gl::MAP_READ_BIT
                ) as *mut Vector4<f32>;
                assert!(0 == gl::GetError());
            }
        }

        gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_info);
        self.ptr_model_info = gl::MapBufferRange(gl::TEXTURE_BUFFER, 0, 
                (mem::size_of::<(u32, u32, u32, u32)>()*self.size) as GLsizeiptr,
                gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
        ) as *mut (u32, u32, u32, u32);
        assert!(0 == gl::GetError());
    }

    fn setup_scene_async(&mut self) {
        let db = self.db.as_ref().unwrap();
        let (evt, position) = unsafe {
            match self.cl {
                None => {
                    mut_buf_as_slice(self.ptr_model_matrix[0], self.size, |mat0| {
                    mut_buf_as_slice(self.ptr_model_matrix[1], self.size, |mat1| {
                    mut_buf_as_slice(self.ptr_model_matrix[2], self.size, |mat2| {
                    mut_buf_as_slice(self.ptr_model_matrix[3], self.size, |mat3| {
                        let mut mat = GLMatrix {
                            x: mat0, y: mat1, z: mat2, w: mat3
                        };
                        (None, db.to_positions(&mut mat))
                    })})})})               
                }
                Some((ref mut ctx, ref cq, ref buf)) => {
                    let (evt, pos) = db.to_positions_cl(cq.deref(), ctx, buf);
                    (Some(evt), pos)
                }
            }
        };
        self.position = Some(position);

        self.material_to_id.clear();

        for (id, (key, _)) in db.material_iter().enumerate() {
            self.material_to_id.insert(*key, (id+1) as u32);
            self.id_to_material.insert((id+1) as u32, *key);
        }

        unsafe {
            mut_buf_as_slice(self.ptr_model_info, self.size, |info| {
                for (idx, (id, (draw, pos))) in join_maps(db.drawable_iter(), db.location_iter()).enumerate() {
                    info[idx] = (id.clone(),
                                 self.position.as_ref().unwrap().get_loc(*pos) as u32,
                                 self.material_to_id.find(&draw.material).unwrap().clone(),
                                 0u32);
                }
            });
        }

        match evt {
            Some(e) => e.wait(),
            None => ()
        }
    }

    fn setup_scene(&mut self)
    {
        if self.cl.is_none() {
            for i in range(0u, 4) {
                gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_matrix[i]);
                gl::UnmapBuffer(gl::TEXTURE_BUFFER);
                assert!(0 == gl::GetError());
                self.ptr_model_matrix[i] = ptr::mut_null();
            }
        }

        gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_info);
        gl::UnmapBuffer(gl::TEXTURE_BUFFER);
        assert!(0 == gl::GetError());
        self.ptr_model_info = ptr::mut_null();;
    }

    fn render(&mut self, camera: Matrix4<f32>)
    {
        let db = self.db.as_ref().unwrap();
        let shader = db.flat_instance_shader.unwrap();
        shader.bind();

        unsafe {
            gl::UniformMatrix4fv(shader.uniform("mat_proj_view"), 1, gl::FALSE, camera.ptr());

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
        for (idx, (_, draw)) in db.drawable_iter().enumerate() {
            if last_geo.is_some() {
                let (start, end) = range;
                if last_geo.unwrap() == draw.geometry {
                    range = (start, idx);
                } else {
                    let draw_geo = db.geometry(last_geo.unwrap()).unwrap();
                    let draw_vbo = db.vertex.find(&draw_geo.vb).unwrap();

                    draw_vbo.bind();

                    unsafe {
                        gl::Uniform1i(shader.uniform("instance_offset"), start as i32);
                        gl::DrawElementsInstanced(gl::TRIANGLES,
                            draw_geo.count as GLint,
                            gl::UNSIGNED_INT,
                            (draw_geo.offset * 4) as *c_void,
                            (end - start + 1) as GLint
                        );
                    }

                    range = (idx, idx);
                    last_geo = Some(draw.geometry);
                }
            } else {
                range = (idx, idx);
                last_geo = Some(draw.geometry);
            }
        }

        if last_geo.is_some() {
            let draw_geo = db.geometry(last_geo.unwrap()).unwrap();
            let draw_vbo = db.vertex.find(&draw_geo.vb).unwrap();
            let (start, end) = range;

            draw_vbo.bind();

            unsafe {
                gl::Uniform1i(shader.uniform("instance_offset"), start as i32);
                gl::DrawElementsInstanced(gl::TRIANGLES,
                    draw_geo.count as GLint,
                    gl::UNSIGNED_INT,
                    (draw_geo.offset * 4) as *c_void,
                    (end - start + 1) as GLint
                );
            }
        }

    }

    fn materials(&self) -> Vec<Material> {
        let mut mats = Vec::new();
        for (_, key) in self.id_to_material.iter() {
            mats.push(self.db.as_ref().unwrap().material(*key).unwrap().clone());
        }
        mats
    }
}

/*

struct Indirect {
    vertex_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: u32,
    base_instance: u32
}

pub struct DrawlistBindless
{
    model_matrix: GLuint,
    model_delta: GLuint,
    model_delta_ptr: *mut Delta,
    max_size: uint,
    bins: TreeMap<Drawable ,~[u32]>,
    gl_pos: Option<PositionsGL>
}

impl DrawlistBindless
{
    pub fn new(max_size: uint) -> DrawlistBindless
    {
        let buffers = &mut [0, 0];

        let delta = unsafe {
            let size = (mem::size_of::<Matrix4<f32>>() * max_size) as GLsizeiptr;
            let flags = gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT;
            gl::GenBuffers(2, buffers.unsafe_mut_ref(0));
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, buffers[0]);
            gl::BufferStorage(gl::SHADER_STORAGE_BUFFER, size, ptr::null(), flags);
 
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, buffers[1]);
            gl::BufferStorage(gl::SHADER_STORAGE_BUFFER, size, ptr::null(), flags);
            let delta = gl::MapBufferRange(gl::SHADER_STORAGE_BUFFER, 0, size, flags) as *mut Delta;

            delta
        };

        DrawlistBindless {
            model_delta: buffers[1],
            model_delta_ptr: delta,
            model_matrix: buffers[0],
            max_size: 0,
            bins: TreeMap::new(),
            cmds: ~[],
            gl_pos: None
        }
    }

    // This downloads the positions to the GPU and bins the objects
    #[inline(never)]
    pub fn setup_scene(&mut self, db: &Graphics, _: ObjectKey, _: Option<&CommandQueue>)
    {
        let start = precise_time_ns();
        let num_drawable = db.current.drawable_count();
        assert!(self.max_size < num_drawable);

        // clear bins
        for (_, data) in self.bins.mut_iter() {
            unsafe {data.set_len(0);}
        }

        self.gl_pos = Some(unsafe {
            mut_buf_as_slice(self.model_delta_ptr, 1024*1024, |vec| {
                db.current.position.deref().to_positions_gl(vec)
            })
        });

        let end = precise_time_ns();
        println!("setup scene {}", end - start);
    }

    pub fn calc_pos(&self, accl: &PositionGlAccelerator)
    {
        match self.gl_pos.as_ref() {
            Some(gl_pos) => {
                accl.calc(gl_pos, self.model_delta, self.model_matrix);
            },
            None => ()
        }  
    }

    pub fn render<'a>(&'a mut self, db: &Graphics, camera: Matrix4<f32>)
    {
        let start = precise_time_ns();
        let shader = db.flat_bindless_shader.unwrap();
        shader.bind();
        unsafe {
            gl::UniformMatrix4fv(shader.uniform("mat_proj_view"), 1, gl::FALSE, camera.ptr());
        }

        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, self.model_matrix);

        let mut buffer = ~[];

        for (geo, vals) in db.current.draw_bins.iter() {
            let geo = db.current.geometry(*geo).unwrap();
            //let mat = db.current.material(draw.material).unwrap();

            let vbo = db.vertex.find(&geo.vb);
            vbo.unwrap().bind();

            //shader.set_material(mat);

            for v in vals.iter() {
                buffer.push(self.gl_pos.as_ref().unwrap().get_loc(*v) as i32);
                if buffer.len() == 512 {
                    unsafe {
                        gl::Uniform1iv(1, buffer.len() as i32, cast::transmute(&buffer[0]));
                        gl::DrawElementsInstancedBaseInstance(gl::TRIANGLES,
                                                              geo.count as i32,
                                                              gl::UNSIGNED_INT,
                                                              (geo.offset * 32) as *c_void,
                                                              buffer.len() as i32, 0);
                        buffer.set_len(0);
                    }
                }
            }
            if buffer.len() > 0 {
                unsafe {
                    gl::Uniform1iv(1, buffer.len() as i32, cast::transmute(&buffer[0]));
                    gl::DrawElementsInstancedBaseInstance(gl::TRIANGLES,
                                                          geo.count as i32,
                                                          gl::UNSIGNED_INT,
                                                          (geo.offset * 32) as *c_void,
                                                          buffer.len() as i32, 0);
                    buffer.set_len(0);
                }
            }
        }

        let end = precise_time_ns();
        println!("render {}", end - start);
    }
}

impl Drop for DrawlistBindless
{
    fn drop(&mut self)
    {
        /* TODO, is dropped dies on a none-gl task bad things happen */
        //let buffers = &[self.model_matrix];
        //gl::UnmapBuffer(self.model_matrix);
        //gl::DeleteBuffers(1, buffers.unsafe_ref(0));
    }
}
*/