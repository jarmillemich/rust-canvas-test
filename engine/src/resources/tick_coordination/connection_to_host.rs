use std::sync::Arc;

use bevy::prelude::*;
use flexbuffers;
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast,
};
use web_sys::{RtcDataChannel, RtcPeerConnection};

use crate::action::Action;

use super::{action_coordinator::ActionScheduler, tick_queue::TickQueue, types::NetworkMessage};

/// On the client side, a connection to the host
#[wasm_bindgen]
pub struct ConnectionToHost {
    connection: RtcPeerConnection,
    channel: RtcDataChannel,
    action_send_buffer: Vec<Action>,
    message_receive_queue: Arc<Mutex<Vec<Vec<u8>>>>,
    initial_connection_buffer: Vec<NetworkMessage>,
    has_received_world: bool,
}

#[wasm_bindgen]
impl ConnectionToHost {
    #[wasm_bindgen(constructor)]
    pub fn new(connection: RtcPeerConnection, channel: RtcDataChannel) -> Self {
        let message_receive_queue = Self::attach_message_queue(&channel);

        Self {
            connection,
            channel,
            action_send_buffer: Vec::new(),
            message_receive_queue,
            initial_connection_buffer: Vec::new(),
            has_received_world: false,
        }
    }
}

impl ConnectionToHost {
    /// Creates and attaches a message queue to the given data channel
    fn attach_message_queue(channel: &RtcDataChannel) -> Arc<Mutex<Vec<Vec<u8>>>> {
        let message_queue = Arc::new(Mutex::new(Vec::new()));
        let message_queue_clone = message_queue.clone();
        let on_message = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            web_sys::console::log_1(&"Received message from network".into());

            let buffer = event.data();
            let array = Uint8Array::new(&buffer);
            let bytes = array.to_vec();
            message_queue_clone.lock().unwrap().push(bytes);
        }) as Box<dyn FnMut(_)>);
        channel.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();
        web_sys::console::log_1(&"Attached to network channel".into());
        channel.send_with_str("ready").unwrap();
        message_queue
    }

    pub fn send_messages(&mut self, messages: Vec<NetworkMessage>) {
        let mut s = flexbuffers::FlexbufferSerializer::new();
        messages.serialize(&mut s).unwrap();
        self.channel
            .send_with_u8_array(s.view())
            .expect("Should be able to send to host");
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
        let mut s = flexbuffers::FlexbufferSerializer::new();
        let schedule_action_container = vec![NetworkMessage::ScheduleAction {
            actions: self.action_send_buffer.drain(..).collect(),
        }];
        schedule_action_container.serialize(&mut s).unwrap();
        self.channel
            .send_with_u8_array(s.view())
            .expect("Should be able to send to host");
        self.action_send_buffer.clear();

        // Take in everything from the server
        let things: Vec<Vec<u8>> = self
            .message_receive_queue
            .lock()
            .unwrap()
            .drain(..)
            .collect();

        for message in things {
            let de = flexbuffers::Reader::get_root(message.as_slice())
                .expect("Message from host should be a Flexbuffer");
            let mut messages: Vec<NetworkMessage> = Deserialize::deserialize(de)
                .expect("Message from host should be a Vec<NetworkMessage>");

            // XXX Hacky replay
            if self.has_received_world && !self.initial_connection_buffer.is_empty() {
                messages.append(&mut self.initial_connection_buffer);
            }

            while let Some(message) = messages.pop() {
                match message {
                    NetworkMessage::FinalizedTick { tick, actions } => {
                        web_sys::console::log_1(
                            &format!("Got tick finalization for {tick}").into(),
                        );

                        if self.has_received_world {
                            queue.finalize_tick_with_actions(tick, actions);
                        } else {
                            self.initial_connection_buffer
                                .push(NetworkMessage::FinalizedTick { tick, actions });
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
}
