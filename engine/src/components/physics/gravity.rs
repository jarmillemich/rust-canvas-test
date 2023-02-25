use specs::prelude::*;

pub struct Gravity;

impl Component for Gravity {
    type Storage = VecStorage<Self>;
}
