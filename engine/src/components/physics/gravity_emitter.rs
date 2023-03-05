use specs::prelude::*;

#[derive(Default)]
pub struct GravityEmitter;

impl GravityEmitter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component for GravityEmitter {
    type Storage = NullStorage<Self>;
}
