
use gl;
use std::mem;
use collections::{TreeMap, TreeSet};

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

fn format_to_gl_storage(depth: i32) -> u32 {
    match depth {
        1 => gl::R8,
        2 => gl::RG8,
        3 => gl::RGB8,
        4 => gl::RGBA8,
        _ => fail!("depth is to large {}", depth)
    }
}

fn format_to_gl_value(depth: i32) -> u32 {
    match depth {
        1 => gl::RED,
        2 => gl::RG,
        3 => gl::RGB,
        4 => gl::RGBA,
        _ => fail!("depth is to large {}", depth)
    }
}

impl TextureArray {
    pub fn new(width: i32, height: i32, format: i32, depth: i32) -> TextureArray {
        let format = format_to_gl_storage(format);

        let textures = &mut [0];
        unsafe {
            gl::GenTextures(textures.len() as i32, textures.unsafe_mut_ref(0));
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, textures[0]);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexStorage3D(gl::TEXTURE_2D_ARRAY, 8, format, width, height, depth);
            assert!(0 == gl::GetError());
        };

        TextureArray {
            size: (width, height, depth),
            format: format,
            free: range(0, depth).collect(),
            texture: textures[0]
        }
    }

    pub fn matches(&self, text: &Texture) -> bool {
        let (width, height, _) = self.size;

        text.width() == width as uint &&
        text.height() == height as uint &&
        format_to_gl_storage(text.depth() as i32) == self.format   
    }

    pub fn load(&mut self, id: uint, text: &Texture) {
        if !self.matches(text) {
            println!("Texture is not the correct size for array {} {}", text.width(), text.height());
        }

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.texture);
            gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0,
                              id as i32,
                              text.width() as i32,
                              text.height() as i32, 1,
                              format_to_gl_value(text.depth() as i32),
                              gl::UNSIGNED_BYTE,
                              mem::transmute(&text.data()[0]));
            gl::GenerateMipmap(gl::TEXTURE_2D_ARRAY);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
            assert!(0 == gl::GetError());
        }
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
    arrays: TreeMap<uint, TextureArray>,
    loaded: TreeSet<(uint, uint)>
}

impl TextureAtlas {
    pub fn new() -> TextureAtlas {
        TextureAtlas {
            arrays: TreeMap::new(),
            loaded: TreeSet::new()
        }
    }

    pub fn load(&mut self,
                texture_atlas: uint,
                texture_index: uint,
                depth: uint,
                text: &Texture) {

        if self.loaded.contains(&(texture_atlas, texture_index)) {
            return;
        }

        if self.arrays.find(&texture_atlas).is_none() {
            self.arrays.insert(texture_atlas,
                TextureArray::new(
                    text.width() as i32,
                    text.height() as i32,
                    text.depth() as i32,
                    depth as i32
                )
            );
        }

        let array = self.arrays.find_mut(&texture_atlas)
                .expect("could not find textuer array");

        array.load(texture_index, text);
        self.loaded.insert((texture_atlas, texture_index));

    }

    pub fn textures(&self) -> Vec<u32> {
        self.arrays.iter().map(|(_, a)| a.texture()).collect()
    }
}