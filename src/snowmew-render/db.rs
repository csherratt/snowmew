use cow::btree::BTreeMap;
use position::{Positions, PositionData};
use graphics::{Graphics, GraphicsData};
use snowmew::common::{Common, CommonData};
use snowmew::common::ObjectKey;
use {RenderData};

use vertex_buffer::VertexBuffer;
use shader::Shader;

use ovr;
use Config;

static VS_SRC: &'static str = include_str!("shaders/basic_vertex.glsl");
static VS_INSTANCE_SRC: &'static str = include_str!("shaders/instance_vertex.glsl");
static BINDLESS_VS_INSTANCED_SRC: &'static str = include_str!("shaders/bindless_instanced_vertex.glsl");
static VS_PASS_SRC: &'static str = include_str!("shaders/pass_vertex.glsl");
static VR_FS_SRC: &'static str = ovr::SHADER_FRAG_CHROMAB;
static FS_FLAT_SRC: &'static str = include_str!("shaders/flat_fragment.glsl");
static FS_DEFERED_SRC: &'static str = include_str!("shaders/flat_defered_fragment.glsl");

#[deriving(Clone)]
pub struct GlState {
    pub common: CommonData,
    pub position: PositionData,
    pub graphics: GraphicsData,

    pub vertex: BTreeMap<ObjectKey, VertexBuffer>,

    pub flat_shader: Option<Shader>,
    pub flat_instance_shader: Option<Shader>,
    pub flat_bindless_shader: Option<Shader>,

    pub defered_shader: Option<Shader>,

    pub ovr_shader: Option<Shader>,
}

impl Common for GlState {
    fn get_common<'a>(&'a self) -> &'a CommonData { &self.common }
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData { &mut self.common }
}

impl Graphics for GlState {
    fn get_graphics<'a>(&'a self) -> &'a GraphicsData { &self.graphics }
    fn get_graphics_mut<'a>(&'a mut self) -> &'a mut GraphicsData { &mut self.graphics }
}

impl Positions for GlState {
    fn get_position<'a>(&'a self) -> &'a PositionData { &self.position }
    fn get_position_mut<'a>(&'a mut self) -> &'a mut PositionData { &mut self.position }
}

impl GlState {
    pub fn new(db: &RenderData) -> GlState {
        GlState {
            common: db.get_common().clone(),
            position: db.get_position().clone(),
            graphics: db.get_graphics().clone(),
            vertex: BTreeMap::new(),
            flat_shader: None,
            flat_instance_shader: None,
            flat_bindless_shader: None,
            defered_shader: None,
            ovr_shader: None,

        }
    }

    pub fn update(&mut self, db: &RenderData) {
        self.common = db.get_common().clone();
        self.position = db.get_position().clone();
        self.graphics = db.get_graphics().clone();
    }

    fn load_vertex(&mut self, _: &Config) {
        let mut vertex = self.vertex.clone();

        for (oid, vbo) in self.vertex_buffer_iter() {
            match vertex.find(oid) {
                Some(_) => (),
                None => {
                    let vb = VertexBuffer::new(&vbo.vertex, vbo.index.as_slice());
                    vertex.insert(*oid, vb);
                }
            }
        }

        self.vertex = vertex; 
    }

    fn load_shaders(&mut self, cfg: &Config) {
        if self.ovr_shader.is_none() {
            self.ovr_shader = Some(
                Shader::new(VS_PASS_SRC, VR_FS_SRC,
                    &[(0, "pos")],
                    &[(0, "color")]
            ));
        }
        if self.flat_shader.is_none() {
            self.flat_shader = Some(
                Shader::new(VS_SRC, FS_FLAT_SRC,
                    &[(0, "in_position"), (1, "in_texture"), (2, "in_normal")],
                    &[(0, "out_position"), (1, "out_uv"), (2, "out_normal"), (3, "out_material")]
            ));
        }
        if self.defered_shader.is_none() {
            self.defered_shader = Some(Shader::new(VS_PASS_SRC, FS_DEFERED_SRC, &[], &[(0, "color")]));
        }
        if self.flat_instance_shader.is_none() {
            self.flat_instance_shader = Some(
                Shader::new(VS_INSTANCE_SRC, FS_FLAT_SRC, 
                    &[(0, "in_position"), (1, "in_texture"), (2, "in_normal")],
                    &[(0, "out_position"), (1, "out_uv"), (2, "out_normal"), (3, "out_material")]
            )); 
        }
        if cfg.use_bindless() {
            if self.flat_bindless_shader.is_none() {
                self.flat_bindless_shader = Some(
                    Shader::new(BINDLESS_VS_INSTANCED_SRC, FS_FLAT_SRC,
                        &[(0, "in_position"), (1, "in_texture"), (2, "in_normal")],
                        &[(0, "out_position"), (1, "out_uv"), (2, "out_normal"), (3, "out_material")]
                ));
            }
        }
    }

    pub fn load(&mut self, cfg: &Config) {
        self.load_vertex(cfg);
        self.load_shaders(cfg);
    }
}