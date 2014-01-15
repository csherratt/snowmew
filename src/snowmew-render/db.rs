use std::util;

use cow::btree::BTreeMap;
use snowmew::core::{Database, object_key};

use vertex_buffer::VertexBuffer;
use shader::Shader;

#[deriving(Clone)]
pub struct Graphics
{
    last: Database,
    current: Database,
    vertex: BTreeMap<object_key, VertexBuffer>,
    shaders: BTreeMap<object_key, Shader>
}

impl Graphics
{
    pub fn new(db: Database) -> Graphics
    {
        Graphics {
            current: db.clone(),
            last: db,
            vertex: BTreeMap::new(),
            shaders: BTreeMap::new()
        }
    }

    pub fn update(&mut self, db: Database)
    {
        util::swap(&mut self.last, &mut self.current);
        self.current = db;

    }

    fn load_vertex(&mut self)
    {
        for (oid, vbo) in self.current.walk_vertex_buffers()
        {
            match self.vertex.find(oid) {
                Some(_) => (),
                None => {
                    let vb = VertexBuffer::new(vbo.vertex.as_slice(), vbo.index.as_slice());
                    self.vertex.insert(*oid, vb);
                }
            }
        }        
    }

    fn load_shaders(&mut self)
    {
        for (oid, shader) in self.current.walk_shaders()
        {
            match self.shaders.find(oid) {
                Some(_) => (),
                None => {
                    let s = match(shader.geometry) {
                        Some(ref geo) => {
                            Shader::new_geo(shader.vertex.as_slice(), geo.as_slice(), shader.frag.as_slice())
                        },
                        None => {
                            Shader::new(shader.vertex.as_slice(), shader.frag.as_slice())
                        }
                    };

                    self.shaders.insert(*oid, s);
                }
            }
        }        
    }

    pub fn load(&mut self)
    {
        self.load_vertex();
        self.load_shaders();
    }
}