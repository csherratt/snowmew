use cow::btree::BTreeMap;
use graphics::{Graphics};
use snowmew::common::{Common};
use snowmew::common::ObjectKey;
use {RenderData};

use vertex_buffer::VertexBuffer;
use shader::Shader;
use texture::TextureAtlas;

use Config;

static VS_SRC: &'static str = include_str!("shaders/basic_vertex.glsl");
static VS_INSTANCE_SRC: &'static str = include_str!("shaders/instance_vertex.glsl");
static BINDLESS_VS_INSTANCED_SRC: &'static str = include_str!("shaders/bindless_instanced_vertex.glsl");
static VS_PASS_SRC: &'static str = include_str!("shaders/pass_vertex.glsl");
static FS_FLAT_SRC: &'static str = include_str!("shaders/flat_fragment.glsl");
static FS_AMBIENT_SRC: &'static str = include_str!("shaders/defered_ambient_fragment.glsl");
static FS_POINT_LIGHT_SRC: &'static str = include_str!("shaders/defered_point_light_fragment.glsl");

#[deriving(Clone)]
pub struct GlState {
    pub vertex: BTreeMap<ObjectKey, VertexBuffer>,
    pub flat_shader: Option<Shader>,
    pub flat_instance_shader: Option<Shader>,
    pub flat_bindless_shader: Option<Shader>,
    pub defered_shader_ambient: Option<Shader>,
    pub defered_shader_point_light: Option<Shader>,
    pub ovr_shader: Option<Shader>,
    pub texture: TextureAtlas
}

impl GlState {
    pub fn new() -> GlState {
        GlState {
            vertex: BTreeMap::new(),
            flat_shader: None,
            flat_instance_shader: None,
            flat_bindless_shader: None,
            defered_shader_ambient: None,
            defered_shader_point_light: None,
            ovr_shader: None,
            texture: TextureAtlas::new()

        }
    }

    fn load_textures(&mut self, db: &RenderData, _: &Config) {
        for (id, texture) in db.texture_iter() {
            if self.texture.get_index(*id).is_none() {
                self.texture.load(*id, texture);
            }
        }
    }

    fn load_vertex(&mut self, db: &RenderData, _: &Config) {
        let mut vertex = self.vertex.clone();

        for (oid, vbo) in db.vertex_buffer_iter() {
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

    fn load_shaders(&mut self, _: &RenderData, cfg: &Config) {
        if self.flat_shader.is_none() {
            self.flat_shader = Some(
                Shader::new(VS_SRC, FS_FLAT_SRC,
                    &[(0, "in_position"), (1, "in_texture"), (2, "in_normal")],
                    &[(0, "out_uv"), (1, "out_normal"), (2, "out_material"), (3, "out_dxdt")]
            ));
        }
        if self.defered_shader_ambient.is_none() {
            self.defered_shader_ambient = Some(Shader::new(VS_PASS_SRC, FS_AMBIENT_SRC,
                &[],
                &[(0, "color")]
            ));
        }
        if self.defered_shader_point_light.is_none() {
            self.defered_shader_point_light = Some(Shader::new(VS_PASS_SRC, FS_POINT_LIGHT_SRC,
                &[],
                &[(0, "color")]
            ));
        }
        if self.flat_instance_shader.is_none() {
            self.flat_instance_shader = Some(
                Shader::new(VS_INSTANCE_SRC, FS_FLAT_SRC, 
                    &[(0, "in_position"), (1, "in_texture"), (2, "in_normal")],
                    &[(0, "out_uv"), (1, "out_normal"), (2, "out_material"), (3, "out_dxdt")]
            )); 
        }
        if cfg.use_bindless() {
            if self.flat_bindless_shader.is_none() {
                self.flat_bindless_shader = Some(
                    Shader::new(BINDLESS_VS_INSTANCED_SRC, FS_FLAT_SRC,
                        &[(0, "in_position"), (1, "in_texture"), (2, "in_normal")],
                        &[(0, "out_uv"), (1, "out_normal"), (2, "out_material")]
                ));
            }
        }
    }

    pub fn load(&mut self, db: &RenderData, cfg: &Config) {
        self.load_vertex(db, cfg);
        self.load_textures(db, cfg);
        self.load_shaders(db, cfg);
    }
}