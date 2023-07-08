use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bevy::prelude::{ResMut, NonSendMut};
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::RtcDataChannel;

use super::{types::NetworkMessage, ResNetworkQueue};

/// An abstraction around some channel that we can transceive NetworkMessages on
pub trait NetworkChannel {
    fn send(&mut self, messages: Vec<NetworkMessage>);
    fn drain(&mut self) -> Vec<NetworkMessage>;
}

pub struct RtcNetworkChannel {
    channel: RtcDataChannel,
    message_queue: Arc<Mutex<Vec<NetworkMessage>>>,
}

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
    fn send(&mut self, messages: Vec<NetworkMessage>) {
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

// Suppose we have a single non-send resource that owns our network channels
// And other things will just have a id-reference to a particular channel
#[derive(Default)]
pub struct ResChannelManager {
    channels: HashMap<ChannelId, Box<dyn NetworkChannel>>,
    last_id: usize,
}

#[derive(Eq, PartialEq, Hash, Serialize, Deserialize, Copy, Clone)]
pub struct ChannelId(usize);

impl ResChannelManager {
    pub fn register_channel(&mut self, channel: Box<dyn NetworkChannel>) -> ChannelId {
        let id = ChannelId(self.last_id + 1);
        self.last_id += 1;

        assert!(self.channels.get(&id).is_none(), "Channel id collision");

        self.channels.insert(id, channel);

        id
    }

    pub fn get_channel(&self, id: ChannelId) -> &Box<dyn NetworkChannel> {
        self.channels.get(&id).unwrap()
    }

    pub fn get_channel_mut(&mut self, id: ChannelId) -> &mut Box<dyn NetworkChannel> {
        self.channels.get_mut(&id).unwrap()
    }

    pub fn iter_channels_mut(
        &mut self,
    ) -> impl Iterator<Item = (&ChannelId, &mut Box<dyn NetworkChannel>)> {
        self.channels.iter_mut()
    }
}

pub fn sys_network_comms(
    mut channel_manager: NonSendMut<ResChannelManager>,
    mut network_queue: ResMut<ResNetworkQueue>,
) {
    for (channel_id, channel) in channel_manager.iter_channels_mut() {
        let messages = channel.drain();
        network_queue.on_messages(*channel_id, messages);
    }
}

//#region For tests
struct FakeNetworkChannel {
    inbound: Arc<Mutex<Vec<NetworkMessage>>>,
    outbound: Arc<Mutex<Vec<NetworkMessage>>>,
}

#[cfg(test)]
pub fn make_fake_network_channel_pair() -> (Box<dyn NetworkChannel>, Box<dyn NetworkChannel>) {
    let inbound = Arc::new(Mutex::new(Vec::new()));
    let outbound = Arc::new(Mutex::new(Vec::new()));

    let channel1 = Box::new(FakeNetworkChannel {
        inbound: outbound.clone(),
        outbound: inbound.clone(),
    });
    let channel2 = Box::new(FakeNetworkChannel { inbound, outbound });

    (channel1, channel2)
}

impl NetworkChannel for FakeNetworkChannel {
    fn send(&mut self, mut messages: Vec<NetworkMessage>) {
        self.outbound.lock().unwrap().append(&mut messages);
    }

    fn drain(&mut self) -> Vec<NetworkMessage> {
        self.inbound.lock().unwrap().drain(..).collect()
    }
}

#[test]
fn test_fake_network_pair() {
    // Ensure we can send and receive messages over our testing channel
    let (mut channel1, mut channel2) = make_fake_network_channel_pair();

    channel1.send(vec![NetworkMessage::Ping(1)]);
    channel1.send(vec![NetworkMessage::Ping(2), NetworkMessage::Pong(3)]);
    channel2.send(vec![NetworkMessage::Ping(4)]);

    assert_eq!(
        channel2.drain().as_slice(),
        [
            NetworkMessage::Ping(1),
            NetworkMessage::Ping(2),
            NetworkMessage::Pong(3)
        ]
    );

    channel1.send(vec![NetworkMessage::Ping(5)]);

    assert_eq!(channel2.drain().as_slice(), [NetworkMessage::Ping(5)]);
    assert_eq!(channel1.drain().as_slice(), [NetworkMessage::Ping(4)]);
}
//#endregion For tests
