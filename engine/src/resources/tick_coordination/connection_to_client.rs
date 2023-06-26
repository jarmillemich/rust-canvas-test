use std::sync::{Arc, Mutex};

use js_sys::Uint8Array;
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use web_sys::{RtcDataChannel, RtcPeerConnection};

use super::{tick_queue::TickQueue, types::NetworkMessage};

enum ConnectionState {
    NeedsWorldLoad,
    CatchingUp,
    Connected,
    Disconnected,
}

/// On the host side, a connection to a client
#[wasm_bindgen]
pub struct ConnectionToClient {
    connection: RtcPeerConnection,
    channel: RtcDataChannel,
    message_queue: Arc<Mutex<Vec<Vec<u8>>>>,
    last_sync_tick: usize,
    state: ConnectionState,
}

#[wasm_bindgen]
impl ConnectionToClient {
    #[wasm_bindgen(constructor)]
    pub fn new(connection: RtcPeerConnection, channel: RtcDataChannel) -> Self {
        let message_queue = Self::attach_message_queue(&channel);

        Self {
            connection,
            channel,
            message_queue,
            last_sync_tick: 0,
            state: ConnectionState::NeedsWorldLoad,
        }
    }
}

impl ConnectionToClient {
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

    pub fn set_sync(&mut self, tick: usize) {
        self.last_sync_tick = tick;
    }

    pub fn take_current_messages(&mut self) -> Vec<NetworkMessage> {
        let mut lock = self.message_queue.lock().unwrap();
        let mut ret = Vec::new();
        for message in lock.drain(..) {
            let de = flexbuffers::Reader::get_root(message.as_slice())
                .expect("Message from host should be a Flexbuffer");
            let mut messages: Vec<NetworkMessage> = Deserialize::deserialize(de)
                .expect("Message from host should be a Vec<NetworkMessage>");
            ret.append(&mut messages);
        }
        ret
    }

    pub fn synchronize_to_queue(&mut self, queue: &TickQueue) {
        // Send all ticks between this clients last synced tick and the latest available finalized tick
        let (last_sync_tick, messages) =
            queue.make_tick_finalization_messages(self.last_sync_tick + 1);
        // Just kind of move things forward, we'll want to get ACKs from the client later
        self.last_sync_tick = last_sync_tick;

        // Serialize messages
        let serialized =
            flexbuffers::to_vec(messages).expect("Messages should serialize to Flexbuffer");
        self.channel
            .send_with_u8_array(&serialized)
            .expect("Should be able to send messages");
    }

    pub fn send_message(&mut self, message: Vec<u8>) {
        self.channel
            .send_with_u8_array(&message)
            .expect("Should be able to send messages");
    }
}
