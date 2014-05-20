
use gl;
use std::cast;
use collections::TreeMap;

use cgmath::vector::Vector2;

use snowmew::ObjectKey;

use graphics::Texture;

#[deriving(Clone)]
pub struct TextureValue {
    index: i32,
    scale: Vector2<f32>
}

#[deriving(Clone)]
pub struct TextureAtlas {
    texture: u32,
    mapping: TreeMap<ObjectKey, TextureValue>,
    last: i32

}

impl TextureAtlas {
    pub fn new() -> TextureAtlas {
        let textures = &mut [0];
        unsafe {
            gl::GenTextures(textures.len() as i32, textures.unsafe_mut_ref(0));
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, textures[0]);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexStorage3D(gl::TEXTURE_2D_ARRAY, 1, gl::RGBA8, 2048, 2048, 192);
            assert!(0 == gl::GetError());
        };

        TextureAtlas {
            texture: textures[0],
            mapping: TreeMap::new(),
            last: 0
        }
    }

    pub fn load(&mut self, oid: ObjectKey, text: &Texture) -> i32 {
        let id = self.last;
        self.last += 1;

        let scale = Vector2::new(text.width() as f32 / 2048f32,
                                 text.height() as f32 / 2048f32);
        self.mapping.insert(oid, TextureValue {
            scale: scale,
            index: id
        });
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.texture);
            if text.width() > 2048 || text.height() > 2048 {
                println!("dropping texture to big {} {}", text.width(), text.height());
                return -1;
            }
            println!("x:{} y:{} d:{} len:{} oid:{}->id:{}",
                     text.width(),
                     text.height(),
                     text.depth(),
                     text.data().len(),
                     oid,
                     id);
            gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0,
                              id as i32, text.width() as i32, text.height() as i32, 1,
                              if text.depth() == 4 {gl::RGBA} else {gl::RGB},
                              gl::UNSIGNED_BYTE,
                              cast::transmute(&text.data()[0]));
            assert!(0 == gl::GetError());

        }
        id
    }

    pub fn get_index(&self, id: ObjectKey) -> Option<i32> {
        match self.mapping.find(&id) {
            Some(ref id) => Some(id.index),
            None => None
        }
    }

    pub fn get_scale(&self, id: ObjectKey) -> Option<Vector2<f32>> {
        match self.mapping.find(&id) {
            Some(ref id) => Some(id.scale.clone()),
            None => None
        }
    }

    pub fn texture(&self) -> u32 {
        self.texture
    }
}