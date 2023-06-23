use std::sync::Arc;

use bevy::prelude::World;
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
        }
    }
}

impl ConnectionToHost {
    /// Creates and attaches a message queue to the given data channel
    fn attach_message_queue(channel: &RtcDataChannel) -> Arc<Mutex<Vec<Vec<u8>>>> {
        let message_queue = Arc::new(Mutex::new(Vec::new()));
        let message_queue_clone = message_queue.clone();
        let on_message = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            let mut lock = message_queue_clone.lock().unwrap();
            let buffer = event.data();
            let array = Uint8Array::new(&buffer);
            let bytes = array.to_vec();
            lock.push(bytes);
        }) as Box<dyn FnMut(_)>);
        channel.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();
        message_queue
    }
}

impl ActionScheduler for ConnectionToHost {
    fn add_action(&mut self, queue: &mut TickQueue, action: Action) {
        self.action_send_buffer.push(action);
    }

    fn synchronize(&mut self, queue: &mut TickQueue, world: &mut World) {
        // Send everything in the action buffer
        let mut s = flexbuffers::FlexbufferSerializer::new();
        self.action_send_buffer.serialize(&mut s).unwrap();
        self.channel
            .send_with_u8_array(s.view())
            .expect("Should be able to send to host");
        self.action_send_buffer.clear();

        // Take in everything from the server
        let mut lock = self.message_receive_queue.lock().unwrap();
        // Just a Vec<Action> for now
        for message in lock.drain(..) {
            let de = flexbuffers::Reader::get_root(message.as_slice())
                .expect("Message from host should be a Flexbuffer");
            let messages: Vec<NetworkMessage> =
                Deserialize::deserialize(de).expect("Message from host should be a Vec<Action>");

            for message in messages {
                match message {
                    NetworkMessage::FinalizedTick { tick, actions } => {
                        queue.finalize_tick_with_actions(tick, actions);
                    }

                    NetworkMessage::World {
                        scene,
                        last_finalized_tick,
                    } => queue.load_world(world, scene, last_finalized_tick),

                    _ => panic!("Unexpected message from host"),
                }
            }
        }
    }
}
