
use gl;
use std::cast;
use collections::TreeMap;
use snowmew::ObjectKey;

use graphics::Texture;

#[deriving(Clone)]
pub struct TextureAlmanac {
    texture: u32,
    mapping: TreeMap<ObjectKey, i32>,
    last: i32

}

impl TextureAlmanac {
    pub fn new() -> TextureAlmanac {
        let textures = &mut [0];
        unsafe {
            gl::GenTextures(1, textures.unsafe_mut_ref(0));
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, textures[0]);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexStorage3D(gl::TEXTURE_2D_ARRAY, 1, gl::RGBA8, 1024, 1024, 512);
            assert!(0 == gl::GetError());
        };

        TextureAlmanac {
            texture: textures[0],
            mapping: TreeMap::new(),
            last: 0
        }
    }

    pub fn load(&mut self, oid: ObjectKey, text: &Texture) -> i32 {
        let id = self.last;
        self.last += 1;
        self.mapping.insert(oid, id);
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.texture);
            if text.width() > 1024 || text.height() > 1024 {
                return id;
            }
            gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0,
                              id as i32, text.width() as i32, text.height() as i32, 1,
                              gl::RGBA, gl::UNSIGNED_BYTE,
                              cast::transmute(&text.data()[0]));
            assert!(0 == gl::GetError());

        }
        id
    }

    pub fn get_index(&self, id: ObjectKey) -> Option<i32> {
        match self.mapping.find(&id) {
            Some(id) => Some(*id),
            None => None
        }
    }

    pub fn texture(&self) -> u32 {
        self.texture
    }
}