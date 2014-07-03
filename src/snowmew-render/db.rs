use cow::btree::BTreeMap;
use graphics::{Graphics};
use snowmew::common::{Common};
use snowmew::common::ObjectKey;

use render_data::RenderData;
use vertex_buffer::VertexBuffer;
use shader::Shader;
use texture::TextureAtlas;

use Config;

static VERTEX_PASS: &'static str = include_str!("shaders/pass_vertex.glsl");
static GEO_PASS_VERTEX: &'static str = include_str!("shaders/geometry_pass_vertex.glsl");
static GEO_PASS_FRAG: &'static str = include_str!("shaders/geometry_pass_fragment.glsl");
static DEFERED_POINT_LIGHT: &'static str = include_str!("shaders/defered_point_light_fragment.glsl");
static CULL_SHADER: &'static str = include_str!("shaders/compute_cull.glsl");

static HEADER_410: &'static str = "#version 410\n";
static HEADER_430: &'static str = "#version 430\n#define USE_SSBO 1\n";

#[deriving(Clone)]
pub struct GlState {
    pub vertex: BTreeMap<ObjectKey, VertexBuffer>,
    pub geometry_no_ssbo: Option<Shader>,
    pub geometry_ssbo_drawid: Option<Shader>,
    pub flat_bindless_shader: Option<Shader>,
    pub defered_shader_point_light: Option<Shader>,
    pub ovr_shader: Option<Shader>,
    pub compute_cull: Option<Shader>,
    pub texture: TextureAtlas
}

impl GlState {
    pub fn new() -> GlState {
        GlState {
            vertex: BTreeMap::new(),
            geometry_no_ssbo: None,
            geometry_ssbo_drawid: None,
            flat_bindless_shader: None,
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
                        .expect("Can't find texture");
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

    fn load_shaders(&mut self, _: &RenderData, cfg: &Config) {
        if self.defered_shader_point_light.is_none() {
            self.defered_shader_point_light = Some(Shader::new(VERTEX_PASS, DEFERED_POINT_LIGHT,
                &[],
                &[(0, "color")],
                Some(HEADER_410)
            ));
        }
        if self.geometry_no_ssbo.is_none() {
            self.geometry_no_ssbo = Some(
                Shader::new(GEO_PASS_VERTEX, GEO_PASS_FRAG, 
                    &[(0, "in_position"), (1, "in_texture"), (2, "in_normal")],
                    &[(0, "out_uv"), (1, "out_normal"), (2, "out_material"), (3, "out_dxdt")],
                    Some(HEADER_410)
            )); 
        }
        if cfg.ssbo() && self.geometry_ssbo_drawid.is_none() {
            self.geometry_ssbo_drawid = Some(
                Shader::new(GEO_PASS_VERTEX, GEO_PASS_FRAG, 
                    &[(0, "in_position"), (1, "in_texture"), (2, "in_normal")],
                    &[(0, "out_uv"), (1, "out_normal"), (2, "out_material"), (3, "out_dxdt")],
                    Some(HEADER_430)
            )); 
        }
        if cfg.compute() && self.compute_cull.is_none() {
            self.compute_cull = Some(Shader::compute(CULL_SHADER, None));
        }
    }

    pub fn load(&mut self, db: &RenderData, cfg: &Config) {
        self.load_shaders(db, cfg);
        self.load_vertex(db, cfg);
        self.load_textures(db, cfg);
    }
}