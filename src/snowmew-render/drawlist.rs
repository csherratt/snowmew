use std::mem;
use std::ptr;
use libc::{c_void};
use std::slice::raw::mut_buf_as_slice;

use cow::join::join_maps;

use cgmath::matrix::Mat4;
use cgmath::vector::Vec4;
use cgmath::ptr::Ptr;
use db::GlState;

use snowmew::material::Material;
use snowmew::core::{ObjectKey};
use snowmew::position::{Positions, Position};
use snowmew::graphics::Graphics;

use gl;
use gl::types::{GLint, GLuint, GLsizeiptr};

use collections::treemap::TreeMap;

use Config;

pub trait Drawlist
{
    // done on the context manager before, Graphics is owned by
    // the draw list. If there was already a bound scene this
    // needs to be replaces with the current scene
    fn bind_scene(&mut self, db: GlState, scene: ObjectKey);

    // done first on an external thread
    fn setup_scene_async(&mut self);

    // setup on the render thread, called after setup_scene_async
    fn setup_scene(&mut self);

    // done many times on the render thread
    fn render(&mut self, camera: Mat4<f32>);

    // get materials
    fn materials(&self) -> ~[Material];
}

pub struct DrawlistStandard
{
    db: Option<GlState>,
    scene: ObjectKey,
    position: Option<Positions>,

    material_to_id: TreeMap<ObjectKey, u32>,
    id_to_material: TreeMap<u32, ObjectKey>,

    size: uint,

    // one array for each component
    model_matrix: [GLuint, ..4],
    model_info: GLuint,

    text_model_matrix: [GLuint, ..4],
    text_model_info: GLuint,

    ptr_model_matrix: [*mut Vec4<f32>, ..4],
    ptr_model_info: *mut (u32, u32, u32, u32),
}

impl DrawlistStandard
{
    pub fn from_config(cfg: &Config) -> ~DrawlistStandard
    {
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
                               (mem::size_of::<Vec4<f32>>()*cfg.max_size()) as GLsizeiptr,
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

        ~DrawlistStandard {
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
            id_to_material: TreeMap::new()
        }
    }
}

impl Drawlist for DrawlistStandard
{
    fn bind_scene(&mut self, db: GlState, scene: ObjectKey)
    {
        self.db = Some(db);
        self.scene = scene;

        for i in range(0u, 4) {
            gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_matrix[i]);
            self.ptr_model_matrix[i] = gl::MapBufferRange(gl::TEXTURE_BUFFER, 0, 
                    (mem::size_of::<Vec4<f32>>()*self.size) as GLsizeiptr,
                    gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
            ) as *mut Vec4<f32>;
            assert!(0 == gl::GetError());
        }

        gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_info);
        self.ptr_model_info = gl::MapBufferRange(gl::TEXTURE_BUFFER, 0, 
                (mem::size_of::<(u32, u32, u32, u32)>()*self.size) as GLsizeiptr,
                gl::MAP_WRITE_BIT | gl::MAP_INVALIDATE_BUFFER_BIT
        ) as *mut (u32, u32, u32, u32);
        assert!(0 == gl::GetError());
    }

    fn setup_scene_async(&mut self)
    {
        self.position = None;
        self.position = Some(self.db.as_ref().unwrap().current.to_positions());

        self.material_to_id.clear();

        for (id, (key, _)) in self.db.as_ref().unwrap().current.material_iter().enumerate() {
            self.material_to_id.insert(*key, (id+1) as u32);
            self.id_to_material.insert((id+1) as u32, *key);
        }

        unsafe {
            mut_buf_as_slice(self.ptr_model_matrix[0], self.size, |mat0| {
            mut_buf_as_slice(self.ptr_model_matrix[1], self.size, |mat1| {
            mut_buf_as_slice(self.ptr_model_matrix[2], self.size, |mat2| {
            mut_buf_as_slice(self.ptr_model_matrix[3], self.size, |mat3| {
                let mats = self.position.as_ref().unwrap().all_mats();
                for (idx, m) in mats.iter().enumerate() {
                    mat0[idx] = m.x;
                    mat1[idx] = m.y;
                    mat2[idx] = m.z;
                    mat3[idx] = m.w;
                }
            })})})});
        
            let db = &self.db.as_ref().unwrap().current;

            mut_buf_as_slice(self.ptr_model_info, self.size, |info| {
                for (idx, (id, (draw, pos))) in join_maps(db.drawable_iter(), db.location_iter()).enumerate() {
                    info[idx] = (id.clone(),
                                 self.position.as_ref().unwrap().get_loc(*pos) as u32,
                                 self.material_to_id.find(&draw.material).unwrap().clone(),
                                 0u32);
                }
            });
        }
    }

    fn setup_scene(&mut self)
    {
        for i in range(0u, 4) {
            gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_matrix[i]);
            gl::UnmapBuffer(gl::TEXTURE_BUFFER);
            assert!(0 == gl::GetError());
            self.ptr_model_matrix[i] = ptr::mut_null();
        }

        gl::BindBuffer(gl::TEXTURE_BUFFER, self.model_info);
        gl::UnmapBuffer(gl::TEXTURE_BUFFER);
        assert!(0 == gl::GetError());
        self.ptr_model_info = ptr::mut_null();;
    }

    fn render(&mut self, camera: Mat4<f32>)
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

        unsafe {
            let buffers = &[gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1, gl::COLOR_ATTACHMENT2, gl::COLOR_ATTACHMENT3];
            gl::DrawBuffers(4, buffers.unsafe_ref(0));
        }

        let mut range = (0u, 0u);
        let mut last_geo: Option<u32> = None;
        for (idx, (_, draw)) in self.db.as_ref().unwrap().current.drawable_iter().enumerate() {
            if last_geo.is_some() {
                if last_geo.unwrap() == draw.geometry {
                    let (start, _) = range;
                    range = (start, idx);
                } else {
                    let draw_geo = db.current.geometry(last_geo.unwrap()).unwrap();
                    let draw_vbo = db.vertex.find(&draw_geo.vb).unwrap();

                    draw_vbo.bind();
                    let (start, end) = range;

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
            let draw_geo = db.current.geometry(last_geo.unwrap()).unwrap();
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

    fn materials(&self) -> ~[Material]
    {
        let mut mats = ~[];
        for (_, key) in self.id_to_material.iter() {
            mats.push(self.db.as_ref().unwrap().current.material(*key).unwrap().clone());
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
            let size = (mem::size_of::<Mat4<f32>>() * max_size) as GLsizeiptr;
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

    pub fn render<'a>(&'a mut self, db: &Graphics, camera: Mat4<f32>)
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