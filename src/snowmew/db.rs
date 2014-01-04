
use cgmath::vector::*;

pub type key = u32;

#[deriving(Eq, Clone)]
pub struct Object
{
    parent: Option<key>,
    position: Option<key>,
    render: Option<key>,
    physics: Option<key>,
    nick: Option<~str>
}

#[deriving(Eq, Clone)]
pub struct Position
{
    position: Vec3<f32>,
    rotation: Vec3<f32>,
    scale: f32,
}

#[deriving(Eq, Clone)]
pub struct Render
{
    Geometry: key,
    Material: key,
}

#[deriving(Eq, Clone)]
pub struct Physics
{
    velocity: Vec3<f32>,
    mass: f32
}

#[deriving(Clone)]
pub struct Database
{
    objects: ~[Object],
    positions: ~[Position],
    render: ~[Render],
    physics: ~[Physics]
}

impl Database
{
    pub fn new() -> Database
    {
        Database {
            objects: ~[],
            positions: ~[],
            render: ~[],
            physics: ~[]
        }
    }

    fn add_object(&mut self, parent: Option<key>, pos: Option<key>, ren: Option<key>, physics: Option<key>, nick: Option<~str>) -> key
    {
        let index = self.objects.len() as key;

        self.objects.push(
            Object {
                parent: parent,
                position: pos,
                render: ren,
                physics: physics,
                nick: nick
            }
        );

        index
    }


    fn add_position(&mut self, pos: Vec3<f32>, rot: Vec3<f32>, scale: f32) -> key
    {
        let index = self.positions.len() as key;

        self.positions.push(
            Position {
                position: pos,
                rotation: rot,
                scale: scale
            }
        );

        index
    }

    pub fn create_scene(&mut self, nick: Option<~str>) -> key
    {
        self.add_object(None, None, None, None, nick)
    }

    pub fn create_object(&mut self, parent: key, nick: Option<~str>) -> key
    {
        let pos_idx = self.add_position(Vec3::new(0f32, 0f32, 0f32), Vec3::new(0f32, 0f32, 0f32), 1.);

        self.add_object(Some(parent), Some(pos_idx), None, None, nick)
    }

    pub fn get_position(&self, key: key) -> Option<Position>
    {
        if self.objects.len() <= key as uint {
            None
        } else {
            let pos = &self.objects[key];
            if pos.position.is_none() {
                None
            } else {
                Some(self.positions[pos.position.unwrap()].clone())
            }
        }
    }

    pub fn set_position(&mut self, key: key, pos: Position)
    {
        if self.objects[key].position.is_none() {
            let pos_idx = self.add_position(pos.position, pos.rotation, pos.scale);
            self.objects[key].position = Some(pos_idx);
        } else {
            self.positions[self.objects[key].position.unwrap()] = pos;
        }


    }

    fn ifind(&self, node: Option<key>, str_key: &str) -> Option<key>
    {
        for (key, o) in self.objects.iter().enumerate() {
            if o.parent == node && o.nick.is_some() && o.nick.get_ref().as_slice() == str_key {
                return Some(key as key);
            }
        }
        None
    }

    pub fn find(&self, str_key: &str) -> Option<key>
    {
        let mut node = None;
        for s in str_key.split('/') {
            let next = self.ifind(node, s);
            if next == None {
                return None
            }
            node = next;
        }
        node
    }

    pub fn name(&self, key: key) -> ~str
    {
        if self.objects[key].parent.is_some() {
            format!("{}/{}", self.name(self.objects[key].parent.unwrap()), self.objects[key].nick.get_ref().as_slice())
        } else {
            self.objects[key].nick.get_ref().clone()
        }
    }

    pub fn dump(&self)
    {
        for (key, o) in self.objects.iter().enumerate() {
            if o.nick.is_some() {
                print(format!("{} {}", key, self.name(key as key)));
            } else {
                print(format!("{} <Anonymous>", key));
            }
            println("");
        }
    }
}

