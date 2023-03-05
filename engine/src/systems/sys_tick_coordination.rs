use crate::resources::TickCoordinator;
use specs::prelude::*;
extern crate web_sys;

pub struct SysTickCoordinator;

impl<'a> System<'a> for SysTickCoordinator {
    type SystemData = (WriteExpect<'a, TickCoordinator>,);

    fn run(&mut self, mut tc: Self::SystemData) {
        tc.0.advance();
    }
}
