
use gl;
use std::cast;
use collections::{TreeSet, TreeMap};

use cgmath::vector::Vector2;

use snowmew::ObjectKey;

use graphics::Texture;

#[deriving(Clone)]
pub struct TextureValue {
    index: i32,
    array: i32
}

#[deriving(Clone)]
pub struct TextureArray {
    size: (i32, i32, i32),
    format: u32,
    free: Vec<i32>,
    texture: u32
}

fn calculate_height(width: i32, height: i32, depth: i32) -> i32 {
    let size = (4096*4096*4) / (width*height*depth);
    if size < 4 {
        4
    } else {
        size
    }
}

fn format_to_gl_storage(depth: i32) -> u32 {
    match depth {
        1 => gl::R8,
        2 => gl::RG8,
        3 => gl::RGB8,
        4 => gl::RGBA8,
        _ => fail!("depth is to larger {}", depth)
    }
}

fn format_to_gl_value(depth: i32) -> u32 {
    match depth {
        1 => gl::RED,
        2 => gl::RG,
        3 => gl::RGB,
        4 => gl::RGBA,
        _ => fail!("depth is to larger {}", depth)
    }
}

impl TextureArray {
    pub fn new(width: i32, height: i32, format: i32) -> TextureArray {
        let depth = calculate_height(width, height, format);
        let format = format_to_gl_storage(format);

        let textures = &mut [0];
        unsafe {
            gl::GenTextures(textures.len() as i32, textures.unsafe_mut_ref(0));
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, textures[0]);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexStorage3D(gl::TEXTURE_2D_ARRAY, 1, format, width, height, depth);
            assert!(0 == gl::GetError());
        };

        TextureArray {
            size: (width, height, depth),
            format: format,
            free: range(0, height).map(|x| height - x - 1).collect(),
            texture: textures[0]
        }
    }

    pub fn matches(&self, text: &Texture) -> bool {
        let (width, height, _) = self.size;

        text.width() == width as uint &&
        text.height() == height as uint &&
        format_to_gl_storage(text.depth() as i32) == self.format   
    }

    pub fn load(&mut self, text: &Texture) -> Option<i32> {
        if !self.matches(text) {
            println!("Texture is not the correct size for array {} {}", text.width(), text.height());
            return None;
        }

        // allocate a new id
        let id = match self.free.pop() {
            None => return None,
            Some(id) => id,
        };

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.texture);
            gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0,
                              id as i32,
                              text.width() as i32,
                              text.height() as i32, 1,
                              format_to_gl_value(text.depth() as i32),
                              gl::UNSIGNED_BYTE,
                              cast::transmute(&text.data()[0]));
            gl::GenerateMipmap(gl::TEXTURE_2D_ARRAY);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
            assert!(0 == gl::GetError());
        }
        Some(id)
    }

    pub fn has_space(&self) -> bool {
        !self.free.is_empty()
    }

    pub fn free(&mut self, id: i32) {
        self.free.push(id)
    }

    pub fn texture(&self) -> u32 {
        self.texture
    }    
}

#[deriving(Clone)]
pub struct TextureAtlas {
    arrays: Vec<TextureArray>,
    mapping: TreeMap<ObjectKey, TextureValue>
}

impl TextureAtlas {
    pub fn new() -> TextureAtlas {
        TextureAtlas {
            arrays: Vec::new(),
            mapping: TreeMap::new()
        }
    }

    pub fn load(&mut self, oid: ObjectKey, text: &Texture) {
        let mut value = None;
        for (idx, a) in self.arrays.iter().enumerate() {
            if a.matches(text) && a.has_space() {
                value = Some(idx);
                break;
            }
        }

        let array = match value {
            None => {
                let id = self.arrays.len();
                self.arrays.push(
                    TextureArray::new(text.width() as i32,
                                      text.height() as i32,
                                      text.depth() as i32
                    )
                );
                id
            }
            Some(id) => id
        };

        let index = self.arrays.get_mut(array)
                .load(text).expect("Expected free space");
        self.mapping.insert(oid, TextureValue {
            array: array as i32,
            index: index
        });
    }

    pub fn get_index(&self, id: ObjectKey) -> Option<(i32, i32)> {
        match self.mapping.find(&id) {
            Some(ref id) => Some((id.array+1, id.index)),
            None => None
        }
    }

    pub fn textures(&self) -> Vec<u32> {
        self.arrays.iter().map(|a| a.texture()).collect()
    }
}