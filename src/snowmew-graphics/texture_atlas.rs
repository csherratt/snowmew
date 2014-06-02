
use cow::btree::{BTreeMap};

use snowmew::common::ObjectKey;

use Texture;

#[deriving(Clone)]
pub struct Atlas {
    width: uint,
    height: uint,
    depth: uint,
    max_layers: uint,
    layers: BTreeMap<ObjectKey, uint>,
    free_layers: Vec<uint>
}

impl Atlas {
    pub fn new(width: uint, height: uint, depth: uint) -> Atlas {
        let layers = 100_000_000 / (width * height * depth);

        let mut free_layers = Vec::new();
        for l in range(0, layers) {
            free_layers.push(l);
        }

        Atlas {
            width: width,
            height: height,
            depth: depth,
            max_layers: layers,
            layers: BTreeMap::new(),
            free_layers: free_layers
        }
    }

    pub fn check_texture(&self, text: &Texture) -> bool {
        self.width == text.width()   &&
        self.height == text.height() &&
        self.depth == text.depth()   &&
        self.free_layers.is_empty()
    }

    pub fn add_texture(&mut self, id: ObjectKey, text: &Texture) {
        assert!(!self.check_texture(text));

        let layer = self.free_layers.pop().expect("Failed to get free layer");
        self.layers.insert(id, layer);
    }
}