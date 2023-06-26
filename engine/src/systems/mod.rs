use bevy::prelude::*;

mod sys_movement;
pub use sys_movement::sys_movement;

mod sys_renderer;
pub use sys_renderer::sys_renderer;

mod sys_gravity;
pub use sys_gravity::sys_gravity;

mod sys_input;
pub use sys_input::sys_input;

mod sys_tick_coordination;
pub use sys_tick_coordination::sys_tick_coordination;

mod sys_movement_receive;
pub use sys_movement_receive::sys_movement_receive;

mod sys_fire_receive;
pub use sys_fire_receive::sys_fire_receive;

mod sys_client_init;
pub use sys_client_init::sys_client_init;

pub mod set_client_connection;

pub fn setup_systems(app: &mut App) {
    set_client_connection::attach_to_app(app);
}
