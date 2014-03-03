use std::mem;

use cow::btree::BTreeMap;
use snowmew::core::{Database, object_key};

use vertex_buffer::VertexBuffer;
use shader::Shader;

use ovr;
use Config;

static VS_SRC: &'static str =
"#version 150
uniform mat4 mat_model;
uniform mat4 mat_proj_view;

in vec3 position;
in vec2 in_texture;
in vec3 in_normal;

out vec2 fs_texture;
out vec3 fs_normal;

void main() {
    gl_Position = mat_proj_view * mat_model * vec4(position, 1.);
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

in vec3 position;
in vec2 in_texture;
in vec3 in_normal;

out vec2 fs_texture;
out vec3 fs_normal;

void main() {
    int id = instance[gl_InstanceID];
    gl_Position = mat_proj_view * model_matrix[id] * vec4(position, 1.);
    fs_texture = in_texture;
    fs_normal = in_normal;
}
";

static FS_RAINBOW_NORMAL_SRC: &'static str =
"#version 400

in vec2 fs_texture;
in vec3 fs_normal;

out vec4 color;

void main() {
    color = vec4(fs_normal, 1);
}
";

static FS_RAINBOW_TEXTURE_SRC: &'static str =
"#version 400

in vec2 fs_texture;
in vec3 fs_normal;

out vec4 color;

void main() {
    color = vec4(fs_texture, 0.5, 1);
}
";

static VR_VS_SRC: &'static str =
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
in vec2 fs_texture;
in vec3 fs_normal;

uniform vec3 ambient;

out vec4 color;

void main() {
    color = vec4(ambient, 1);
}
";

#[deriving(Clone)]
pub struct Graphics
{
    last: Database,
    current: Database,
    vertex: BTreeMap<object_key, VertexBuffer>,

    rainbow_normal: Option<Shader>,
    rainbow_texture: Option<Shader>,

    flat_shader: Option<Shader>,
    flat_instanced_shader: Option<Shader>,

    ovr_shader: Option<Shader>,
}

impl Graphics
{
    pub fn new(db: Database) -> Graphics
    {
        Graphics {
            current: db.clone(),
            last: db,
            vertex: BTreeMap::new(),
            rainbow_normal: None,
            rainbow_texture: None,
            flat_shader: None,
            ovr_shader: None,
            flat_instanced_shader: None
        }
    }

    pub fn update(&mut self, db: Database) -> Database
    {
        let mut db = db;
        mem::swap(&mut self.last, &mut self.current);
        mem::swap(&mut self.current, &mut db);
        db

    }

    fn load_vertex(&mut self, _: &Config)
    {
        for (oid, vbo) in self.current.walk_vertex_buffers()
        {
            match self.vertex.find(oid) {
                Some(_) => (),
                None => {
                    let vb = VertexBuffer::new(&vbo.vertex, vbo.index.as_slice());
                    self.vertex.insert(*oid, vb);
                }
            }
        }        
    }

    fn load_shaders(&mut self, cfg: &Config)
    {
        if self.rainbow_normal.is_none() {
            self.rainbow_normal = Some(Shader::new(VS_SRC, FS_RAINBOW_NORMAL_SRC));
        }
        if self.rainbow_texture.is_none() {
            self.rainbow_texture = Some(Shader::new(VS_SRC, FS_RAINBOW_TEXTURE_SRC));
        }
        if self.ovr_shader.is_none() {
            self.ovr_shader = Some(Shader::new(VR_VS_SRC, VR_FS_SRC));
        }
        if self.flat_shader.is_none() {
            self.flat_shader = Some(Shader::new(VS_SRC, FS_FLAT_SRC));
        }
        if cfg.use_bindless() {
            if self.flat_instanced_shader.is_none() {
                self.flat_instanced_shader = Some(Shader::new(BINDLESS_VS_INSTANCED_SRC, FS_FLAT_SRC));
            }
        }
    }

    pub fn load(&mut self, cfg: &Config)
    {
        self.load_vertex(cfg);
        self.load_shaders(cfg);
    }
}