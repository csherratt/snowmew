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
static VS_PASS_SRC: &'static str = include_str!("shaders/pass_vertex.glsl");
static FS_FLAT_SRC: &'static str = include_str!("shaders/flat_fragment.glsl");
static FS_AMBIENT_SRC: &'static str = include_str!("shaders/defered_ambient_fragment.glsl");
static FS_POINT_LIGHT_SRC: &'static str = include_str!("shaders/defered_point_light_fragment.glsl");
static CS_CULL_SRC: &'static str = include_str!("shaders/compute_cull.glsl");

#[deriving(Clone)]
pub struct GlState {
    pub vertex: BTreeMap<ObjectKey, VertexBuffer>,
    pub flat_shader: Option<Shader>,
    pub flat_instance_shader: Option<Shader>,
    pub flat_bindless_shader: Option<Shader>,
    pub defered_shader_ambient: Option<Shader>,
    pub defered_shader_point_light: Option<Shader>,
    pub ovr_shader: Option<Shader>,
    pub compute_cull: Option<Shader>,
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
            compute_cull: None,
            texture: TextureAtlas::new()

        }
    }

    fn load_textures(&mut self, db: &RenderData, _: &Config) {
        for (atlas_idx, atlas) in db.texture_atlas_iter().enumerate() {
            for (oid, idx) in atlas.texture_iter() {
                let texture = db.get_texture(*oid)
                        .expect("Can't find textuer");
                self.texture.load(atlas_idx, *idx, atlas.max_layers(), texture);   
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

    fn load_shaders(&mut self, _: &RenderData, _: &Config) {
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
        if self.compute_cull.is_none() {
            self.compute_cull = Some(Shader::compute(CS_CULL_SRC));
        }
    }

    pub fn load(&mut self, db: &RenderData, cfg: &Config) {
        self.load_shaders(db, cfg);
        self.load_vertex(db, cfg);
        self.load_textures(db, cfg);
    }
}