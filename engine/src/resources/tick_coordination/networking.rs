use std::sync::{Arc, Mutex};

use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::RtcDataChannel;

use super::types::NetworkMessage;

/// An abstraction around some channel that we can transceive NetworkMessages on
pub trait NetworkChannel {
    fn send(&mut self, messages: &[NetworkMessage]);
    fn drain(&mut self) -> Vec<NetworkMessage>;
}

pub struct RtcNetworkChannel {
    channel: RtcDataChannel,
    message_queue: Arc<Mutex<Vec<NetworkMessage>>>,
}

// TODO we are single threaded but...
unsafe impl Send for RtcNetworkChannel {}

impl RtcNetworkChannel {
    pub fn new(channel: RtcDataChannel) -> Self {
        let message_queue = Self::attach_message_queue(&channel);

        Self {
            channel,
            message_queue,
        }
    }

    fn attach_message_queue(channel: &RtcDataChannel) -> Arc<Mutex<Vec<NetworkMessage>>> {
        let message_queue = Arc::new(Mutex::new(Vec::new()));

        // Set up onmessage callback
        let message_queue_clone = message_queue.clone();
        let on_message = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            let buffer = event.data();
            let array = Uint8Array::new(&buffer);
            let bytes = array.to_vec();

            // Deserialize
            let de = flexbuffers::Reader::get_root(bytes.as_slice())
                .expect("Message from host should be a Flexbuffer");
            let mut messages: Vec<NetworkMessage> = Deserialize::deserialize(de)
                .expect("Message from host should be a Vec<NetworkMessage>");

            // Add to our inbound queue
            message_queue_clone.lock().unwrap().append(&mut messages);
        }) as Box<dyn FnMut(_)>);
        channel.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();

        message_queue
    }
}

impl NetworkChannel for RtcNetworkChannel {
    fn send(&mut self, messages: &[NetworkMessage]) {
        let mut s = flexbuffers::FlexbufferSerializer::new();
        messages.serialize(&mut s).unwrap();

        self.channel
            .send_with_u8_array(s.view())
            .expect("Should be able to send data on RTC channel");
    }

    fn drain(&mut self) -> Vec<NetworkMessage> {
        self.message_queue.lock().unwrap().drain(..).collect()
    }
}
