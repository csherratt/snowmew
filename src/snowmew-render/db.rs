use cow::btree::BTreeMap;
use snowmew::position::{Positions, PositionData};
use snowmew::graphics::{Graphics, GraphicsData};
use snowmew::core::{Common, CommonData};
use snowmew::core::ObjectKey;
use {RenderData};

use vertex_buffer::VertexBuffer;
use shader::Shader;

use ovr;
use Config;

static VS_SRC: &'static str =
"#version 150
uniform mat4 mat_model;
uniform mat4 mat_proj_view;
uniform uint object_id;
uniform uint material_id;

in vec3 in_position;
in vec2 in_texture;
in vec3 in_normal;

out vec3 fs_position;
out vec2 fs_texture;
out vec3 fs_normal;
flat out uint fs_object_id;
flat out uint fs_material_id;

void main() {
    gl_Position = mat_proj_view * mat_model * vec4(in_position, 1.);
    vec4 pos = mat_model * vec4(in_position, 1.);
    fs_position = pos.xyz / pos.w;
    fs_texture = in_texture;
    fs_normal = in_normal;
    fs_object_id = object_id;
    fs_material_id = material_id;
}
";

static VS_INSTANCE_SRC: &'static str =
"#version 400
uniform int instance_offset;
uniform mat4 mat_proj_view;

uniform samplerBuffer mat_model0;
uniform samplerBuffer mat_model1;
uniform samplerBuffer mat_model2;
uniform samplerBuffer mat_model3;

uniform usamplerBuffer info;

in vec3 in_position;
in vec2 in_texture;
in vec3 in_normal;

out vec3 fs_position;
out vec2 fs_texture;
out vec3 fs_normal;
flat out uint fs_object_id;
flat out uint fs_material_id;

void main() {
    int instance = gl_InstanceID + instance_offset;
    uvec4 info = texelFetch(info, instance);

    int matrix_id = int(info.y);
    fs_material_id = info.z;
    fs_object_id = info.x;


    mat4 mat_model = mat4(texelFetch(mat_model0, matrix_id),
                          texelFetch(mat_model1, matrix_id),
                          texelFetch(mat_model2, matrix_id),
                          texelFetch(mat_model3, matrix_id));


    vec4 pos = mat_model * vec4(in_position, 1.);
    gl_Position = mat_proj_view * pos;
    fs_position = pos.xyz / pos.w;
    fs_texture = in_texture;
    fs_normal = in_normal;
}
";

static BINDLESS_VS_INSTANCED_SRC: &'static str =
"#version 430
layout(location = 0) uniform mat4 mat_proj_view;
layout(location = 1) uniform int instance[512];

layout(std430, binding = 3) buffer MyBuffer
{
    mat4 model_matrix[];
};

in vec3 in_position;
in vec2 in_texture;
in vec3 in_normal;

out vec3 fs_position;
out vec2 fs_texture;
out vec3 fs_normal;

void main() {
    int id = instance[gl_InstanceID];
    gl_Position = mat_proj_view * model_matrix[id] * vec4(in_position, 1.);
    fs_position = model_matrix[id] * vec4(in_position, 1.);
    fs_texture = in_texture;
    fs_normal = in_normal;
    fs_material_id = material_id;
    fs_object_id = object_id;
}
";

static VS_PASS_SRC: &'static str =
"#version 400
in vec3 pos;

out vec2 TexPos;

void main() {
    gl_Position = vec4(pos.x, pos.y, 0.5, 1.);
    TexPos = vec2((pos.x+1)/2, (pos.y+1)/2); 
}
";

static VR_FS_SRC: &'static str = ovr::SHADER_FRAG_CHROMAB;

static FS_FLAT_SRC: &'static str =
"#version 400

in vec3 fs_position;
in vec2 fs_texture;
in vec3 fs_normal;
flat in uint fs_object_id;
flat in uint fs_material_id;

out vec4 out_position;
out vec2 out_uv;
out vec3 out_normal;
out vec4 out_material;

void main() {
    uint mask = 0xFFFF;
    out_position = vec4(fs_position, gl_FragCoord.z);
    out_uv = fs_texture;
    out_normal = fs_normal;
    out_material = vec4(float(fs_material_id) / 65535., float(fs_object_id) / 65535., 1., 1.);
}
";

static FS_DEFERED_SRC: &'static str =
"#version 400

uniform vec3 mat_color[128];

uniform sampler2D position;
uniform sampler2D uv;
uniform sampler2D normal;
uniform sampler2D pixel_drawn_by;

in vec2 TexPos;
out vec4 color;

void main() {
    ivec2 material = ivec2(texture(pixel_drawn_by, TexPos).xy * 65536.);
    bool edge = 
            (material != ivec2(textureOffset(pixel_drawn_by, TexPos, ivec2( 0,  1)).xy * 65536.)) ||
            (material != ivec2(textureOffset(pixel_drawn_by, TexPos, ivec2( 0, -1)).xy * 65536.)) ||
            (material != ivec2(textureOffset(pixel_drawn_by, TexPos, ivec2( 1,  0)).xy * 65536.)) ||
            (material != ivec2(textureOffset(pixel_drawn_by, TexPos, ivec2(-1,  0)).xy * 65536.)) ||
            (material != ivec2(textureOffset(pixel_drawn_by, TexPos, ivec2( 1,  1)).xy * 65536.)) ||
            (material != ivec2(textureOffset(pixel_drawn_by, TexPos, ivec2(-1, -1)).xy * 65536.)) ||
            (material != ivec2(textureOffset(pixel_drawn_by, TexPos, ivec2( 1, -1)).xy * 65536.)) ||
            (material != ivec2(textureOffset(pixel_drawn_by, TexPos, ivec2(-1,  1)).xy * 65536.));

    if (material.x == 0) {
        color = vec4(0., 0., 0., 1.);
    } else {
        color = vec4(mat_color[material.x], 1.);
    }

    if (edge) {
        color *= 0.5;
    }
}
";

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
            println!("{:?}, {:?}", oid, vbo);
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