use crate::{
    action::{Action, Direction},
    input::{EventQueue, InputEvent},
    resources::TickCoordinator,
};
use bevy::prelude::*;
extern crate web_sys;

pub fn sys_input(mut tc: NonSendMut<TickCoordinator>, eq: ResMut<EventQueue>) {
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
                " " => Action::Fire,
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

        // Request that the action be scheduled
        tc.add_action(action);
    }
}
