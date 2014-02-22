
use cgmath::transform::Transform3D;
use cgmath::quaternion::Quat;
use cgmath::vector::Vec3;
use cgmath::matrix::{Mat4, ToMat4, Matrix};

pub struct Position
{
    priv parent: u32,
    priv transform: Transform3D<f32>
}

impl Default for Position
{
    fn default() -> Position
    {
        Position {
            parent: 0,
            transform: Transform3D::new(1f32, Quat::zero(), Vec3::zero())
        }
    }
}

impl Clone for Position
{
    fn clone(&self) -> Position
    {
        let tras = self.transform.get();
        Position {
            parent: self.parent.clone(),
            transform: Transform3D::new(tras.scale.clone(),
                                    tras.rot.clone(),
                                    tras.disp.clone())
        }
    }
}

pub struct Positions
{
    priv dirty: bool,
    priv delta: ~[~[Position]],
    priv mat4: ~[~[Mat4<f32>]]
}

pub struct Id(u32, u32);


impl Positions
{
    pub fn new() -> Positions
    {
        Positions {
            dirty: true,
            delta: ~[~[Default::default()]],
            mat4:  ~[~[Mat4::identity()]]
        }
    }

    pub fn root() -> Id {Id(0, 0)}

    pub fn calc(&mut self)
    {
        if self.dirty == false {
            return
        }

        for (gen, gen_delta) in self.delta.slice_from(1).iter().enumerate()
        {
            for (idx, delta) in gen_delta.iter().enumerate() {
                let old = self.mat4[gen][delta.parent];
                self.mat4[gen+1][idx] = old.mul_m(&delta.transform.to_mat4());
            }
        }

        self.dirty = false;
    }

    pub fn insert(&mut self, parent: Id, delta: Transform3D<f32>) -> Id
    {
        let Id(gen, id) = parent;

        assert!(gen < self.delta.len() as u32);
        assert!(id < self.delta[gen].len() as u32);

        if self.delta.len() as u32 == gen + 1 {
            self.delta.push(~[]);
        }

        let nid = self.delta[gen+1].len() as u32;
        self.delta[gen+1].push(Position {
            parent: id,
            transform: delta
        });

        if self.mat4.len() as u32 == gen + 1 {
            self.mat4.push(~[]);
        }

        self.mat4[gen+1].push(Mat4::identity());

        self.dirty = true;

        Id(gen+1, nid)
    }

    pub fn get(&self, id :Id) -> Mat4<f32>
    {
        assert!(false == self.dirty);
        let Id(gen, id) = id;

        self.mat4[gen][id].clone()
    }
}