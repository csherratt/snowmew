use std::mem;
use std::ptr;
use std::cast;
use std::libc::{c_void, };
use std::slice::raw::mut_buf_as_slice;

use cgmath::matrix::{Mat4, Matrix};
use cgmath::vector::{Vec4, Vector};
use cgmath::ptr::Ptr;
use db::Graphics;

use snowmew::material::Material;
use snowmew::core::{object_key};
use snowmew::geometry::Geometry;
use snowmew::core::Drawable;
use snowmew::CalcPositionsCl;
use snowmew::position::{Delta, PositionsGL, Positions};

use gl;
use gl::types::{GLint, GLuint, GLsizeiptr};
use cow::join::join_maps;

use OpenCL::hl::{Device, Context, CommandQueue};

use collections::treemap::TreeMap;

use compute_accelerator::PositionGlAccelerator;

use time::precise_time_ns;

use Config;

pub struct ObjectCull<IN>
{
    priv input: IN,
    priv camera: Mat4<f32>
}

impl<'a, IN: Iterator<(object_key, Mat4<f32>)>> ObjectCull<IN>
{
    pub fn new(input: IN, camera: Mat4<f32>) -> ObjectCull<IN>
    {
        ObjectCull {
            input: input,
            camera: camera
        }
    }
}

impl<'a, IN: Iterator<(object_key, Mat4<f32>)>>
     Iterator<(object_key, Mat4<f32>)> for ObjectCull<IN>
{
    #[inline]
    fn next(&mut self) -> Option<(object_key, Mat4<f32>)>
    {
        static cube_points: &'static [Vec4<f32>] = &'static [
            Vec4{x:1., y:  1., z:  1., w: 1.}, Vec4{x:-1., y:  1., z:  1., w: 1.},
            Vec4{x:1., y: -1., z:  1., w: 1.}, Vec4{x:-1., y: -1., z:  1., w: 1.},
            Vec4{x:1., y:  1., z: -1., w: 1.}, Vec4{x:-1., y:  1., z: -1., w: 1.},
            Vec4{x:1., y: -1., z: -1., w: 1.}, Vec4{x:-1., y: -1., z: -1., w: 1.},
        ];

        loop {
            match self.input.next() {
                Some((oid, mat)) => {
                    let proj = self.camera.mul_m(&mat);

                    let mut behind_camera = true;
                    let mut right_of_camera = true;
                    let mut left_of_camera = true;
                    let mut above_camera = true;
                    let mut below_camera = true;

                    for p in cube_points.iter() {
                        let point = proj.mul_v(p);
                        let point = point.mul_s(1./point.w);

                        behind_camera &= point.z > 1.;
                        right_of_camera &= point.x > 1.;
                        left_of_camera &= point.x < -1.;
                        above_camera &= point.y > 1.;
                        below_camera &= point.y < -1.;
                    }

                    if !(behind_camera|right_of_camera|left_of_camera|above_camera|below_camera) {
                        return Some((oid, mat));
                    }
                },
                None => return None
            }
        }
    }
}

pub struct Expand<'a, IN>
{
    priv input: IN,
    priv material_id: object_key,
    priv vb_id: object_key,
    priv last_material_id: object_key,
    priv last_vb_id: object_key,
    priv mat: Option<Mat4<f32>>,
    priv geometry: Option<&'a Geometry>,
    priv db: &'a Graphics
}

impl<'a, IN: Iterator<(object_key, (Mat4<f32>, &'a Drawable))>> Expand<'a, IN>
{
    pub fn new(input: IN, db: &'a Graphics) -> Expand<'a, IN>
    {
        Expand {
            input: input,
            material_id: 0,
            vb_id: 0,
            last_material_id: 0,
            last_vb_id: 0,
            mat: None,
            geometry: None,
            db: db
        }
    }
}

pub enum DrawCommand
{
    Draw(Geometry),
    BindMaterial(object_key),
    BindVertexBuffer(object_key),
    SetModelMatrix(Mat4<f32>),
    MultiDraw(object_key, u32, u32, u32, u32),
    DrawElements(object_key, u32, i32, u32, i32, u32, i32),
    DrawElements2(object_key, Geometry, u32, ~[u32])
}

impl<'a, IN: Iterator<(object_key, (Mat4<f32>, &'a Drawable))>> Iterator<DrawCommand> for Expand<'a, IN>
{
    #[inline]
    fn next(&mut self) -> Option<DrawCommand>
    {
        loop {
            if self.material_id != self.last_material_id {
                self.last_material_id = self.material_id;
                return Some(BindMaterial(self.material_id));
            }

            if self.vb_id != self.last_vb_id {
                self.last_vb_id = self.vb_id;
                return Some(BindVertexBuffer(self.vb_id));
            }

            match self.mat {
                Some(mat) => {
                    let out = SetModelMatrix(mat);
                    self.mat = None;
                    return Some(out);
                },
                None => ()
            }

            match self.geometry {
                Some(geometry) => {
                    let out = Draw(geometry.clone());
                    self.geometry = None;
                    return Some(out);
                },
                None => ()
            }

            match self.input.next() {
                Some((_, (mat, draw))) => {
                    self.mat = Some(mat);
                    self.material_id = draw.material;
                    self.geometry = self.db.current.geometry(draw.geometry);
                    self.vb_id = self.geometry.unwrap().vb;
                },
                None => return None,
            }
        }
    }
}

pub trait Drawlist
{
    // done on the context manager before, Graphics is owned by
    // the draw list. If there was already a bound scene this
    // needs to be replaces with the current scene
    fn bind_scene(&mut self, db: Graphics, scene: object_key);

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
    priv db: Option<Graphics>,
    priv scene: object_key,
    priv position: Option<Positions>,

    priv material_to_id: TreeMap<object_key, u32>,
    priv id_to_material: TreeMap<u32, object_key>,

    priv size: uint,

    // one array for each component
    priv model_matrix: [GLuint, ..4],
    priv model_info: GLuint,

    priv text_model_matrix: [GLuint, ..4],
    priv text_model_info: GLuint,

    priv ptr_model_matrix: [*mut Vec4<f32>, ..4],
    priv ptr_model_info: *mut (u32, u32, u32, u32),
}

impl DrawlistStandard
{
    pub fn from_config(cfg: &Config) -> ~Drawlist
    {
        let buffer = &mut [0, 0, 0, 0, 0];
        let texture = &mut [0, 0, 0, 0, 0];

        unsafe {
            gl::GenBuffers(buffer.len() as i32, buffer.unsafe_mut_ref(0));
            gl::GenTextures(buffer.len() as i32, texture.unsafe_mut_ref(0));
      
            for i in range(0, 4) {
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
        } as ~Drawlist
    }
}

impl Drawlist for DrawlistStandard
{
    fn bind_scene(&mut self, db: Graphics, scene: object_key)
    {
        self.db = Some(db);
        self.scene = scene;

        unsafe {
            for i in range(0, 4) {
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
    }

    fn setup_scene_async(&mut self)
    {
        self.position = None;
        self.position = Some(self.db.as_ref().unwrap().current.position.deref().to_positions());

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
        
            mut_buf_as_slice(self.ptr_model_info, self.size, |info| {
                for (idx, (id, (draw, pos))) in self.db.as_ref().unwrap().current.walk_drawables_and_pos().enumerate() {
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
        for i in range(0, 4) {
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
        let model_matrix = shader.uniform("mat_model");
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
        for (idx, (id, draw)) in self.db.as_ref().unwrap().current.walk_drawables().enumerate() {
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

struct Indirect {
    vertex_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: u32,
    base_instance: u32
}

pub struct DrawlistBindless
{
    priv model_matrix: GLuint,
    priv model_delta: GLuint,
    priv model_delta_ptr: *mut Delta,
    priv max_size: uint,
    priv bins: TreeMap<Drawable ,~[u32]>,
    priv cmds: ~[DrawCommand],
    priv gl_pos: Option<PositionsGL>
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
    pub fn setup_scene(&mut self, db: &Graphics, scene: object_key, queue: Option<&CommandQueue>)
    {
        let start = precise_time_ns();
        let num_drawable = db.current.drawable_count();
        assert!(self.max_size < num_drawable);

        // clear bins
        for (_, data) in self.bins.mut_iter() {
            unsafe {
                data.set_len(0);
            }
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