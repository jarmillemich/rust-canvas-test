use crate::{
    action::{Action, Direction},
    input::{EventQueue, InputEvent},
    resources::TickCoordinator,
};
use specs::prelude::*;
extern crate web_sys;

pub struct SysInput;

impl<'a> System<'a> for SysInput {
    type SystemData = (WriteExpect<'a, TickCoordinator>, ReadExpect<'a, EventQueue>);

    fn run(&mut self, (mut tc, eq): Self::SystemData) {
        // TODO does this keep the lock for the entire loop?
        for event in eq.items().lock().unwrap().drain(..) {
            let action: Action = match event {
                InputEvent::KeyDown { key } => match key.as_str() {
                    "w" => Action::StartMoving { dir: Direction::Up },
                    "a" => Action::StartMoving {
                        dir: Direction::Left,
                    },
                    "s" => Action::StartMoving {
                        dir: Direction::Down,
                    },
                    "d" => Action::StartMoving {
                        dir: Direction::Right,
                    },
                    _ => continue,
                },

                InputEvent::KeyUp { key } => match key.as_str() {
                    "w" => Action::StopMoving { dir: Direction::Up },
                    "a" => Action::StopMoving {
                        dir: Direction::Left,
                    },
                    "s" => Action::StopMoving {
                        dir: Direction::Down,
                    },
                    "d" => Action::StopMoving {
                        dir: Direction::Right,
                    },
                    _ => continue,
                },
                // Ignore unhandled events
                _ => continue,
            };

            // For testing purposes, we'll just schedule everything 5 ticks in the future
            let tick = tc.current_tick + 5;
            tc.enqueue_action(action, tick);
        }
    }
}
