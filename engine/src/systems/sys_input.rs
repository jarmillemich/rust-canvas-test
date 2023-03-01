use crate::resources::TickCoordinator;
use specs::prelude::*;
extern crate web_sys;

pub struct SysInput;

impl<'a> System<'a> for SysInput {
    type SystemData = (WriteExpect<'a, TickCoordinator>,);

    fn run(&mut self, (mut tc): Self::SystemData) {}
}
