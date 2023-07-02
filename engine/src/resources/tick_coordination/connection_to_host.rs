use super::{
    action_coordinator::ActionScheduler,
    networking::{NetworkChannel, RtcNetworkChannel},
    tick_queue::TickQueue,
    types::NetworkMessage,
};
use crate::action::Action;
use bevy::prelude::*;
use flexbuffers;
use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::RtcDataChannel;

/// On the client side, a connection to the host
#[wasm_bindgen]
pub struct ConnectionToHost {
    channel: Box<dyn NetworkChannel>,
    action_send_buffer: Vec<Action>,
    initial_connection_buffer: Vec<NetworkMessage>,
    has_received_world: bool,
}

#[wasm_bindgen]
impl ConnectionToHost {
    #[wasm_bindgen(constructor)]
    pub fn new(channel: RtcDataChannel) -> Self {
        let mut channel = Box::new(RtcNetworkChannel::new(channel));
        channel.send(&[NetworkMessage::RequestWorldLoad]);

        Self {
            channel,
            action_send_buffer: Vec::new(),
            initial_connection_buffer: Vec::new(),
            has_received_world: false,
        }
    }
}

impl ConnectionToHost {
    pub fn send_messages(&mut self, messages: Vec<NetworkMessage>) {
        self.channel.send(messages.as_slice());
    }

    pub fn mark_world_received(&mut self) {
        self.has_received_world = true;
    }
}

impl ActionScheduler for ConnectionToHost {
    fn add_action(&mut self, _queue: &mut TickQueue, action: Action) {
        self.action_send_buffer.push(action);
    }

    fn synchronize(&mut self, queue: &mut TickQueue, mut commands: Commands) {
        // Send everything in the action buffer
        let schedule_action_container = vec![NetworkMessage::ScheduleAction {
            actions: self.action_send_buffer.drain(..).collect(),
        }];
        self.send_messages(schedule_action_container);
        self.action_send_buffer.clear();

        // Take in everything from the server
        let mut messages = self.channel.drain();

        // XXX Hacky replay
        if self.has_received_world && !self.initial_connection_buffer.is_empty() {
            messages.append(&mut self.initial_connection_buffer);
        }

        while let Some(message) = messages.pop() {
            match message {
                NetworkMessage::FinalizedTick { tick, actions } => {
                    if self.has_received_world {
                        queue.finalize_tick_with_actions(tick, actions);
                    } else {
                        self.initial_connection_buffer
                            .push(NetworkMessage::FinalizedTick { tick, actions });

                        web_sys::console::log_1(
                            &format!("Holding tick finalization for {tick}").into(),
                        );
                    }
                }
                // TODO should we just be add/removing resources willy gilly?
                NetworkMessage::World(world_load) => {
                    web_sys::console::log_1(&"Received world from host".into());
                    commands.insert_resource(world_load);
                }

                _ => panic!("Unexpected message from host"),
            }
        }
    }
}
