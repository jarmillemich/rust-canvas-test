use std::sync::{Arc, Mutex};

use super::{networking::NetworkChannel, tick_queue::TickQueue, types::NetworkMessage};
use crate::resources::tick_coordination::networking::RtcNetworkChannel;
use bevy::prelude::Component;
use wasm_bindgen::prelude::*;
use web_sys::RtcDataChannel;

pub enum ConnectionState {
    InitialConnection,
    NeedsWorldLoad,
    CatchingUp,
    Connected,
    Disconnected,
}

/// On the host side, a connection to a client
#[wasm_bindgen]
#[derive(Component)]
pub struct ConnectionToClient {
    //channel: Arc<Mutex<Box<dyn NetworkChannel + Send>>>,
    channel_id: usize,
    last_sync_tick: usize,
    state: ConnectionState,
}

#[wasm_bindgen]
impl ConnectionToClient {
    #[wasm_bindgen(constructor)]
    pub fn new(channel: RtcDataChannel) -> Self {
        let channel: Box<dyn NetworkChannel + Send> = Box::new(RtcNetworkChannel::new(channel));
        let channel = Arc::new(Mutex::new(channel));

        Self {
            channel,
            last_sync_tick: 0,
            state: ConnectionState::InitialConnection,
        }
    }
}

impl ConnectionToClient {
    pub fn get_state(&self) -> &ConnectionState {
        &self.state
    }

    pub fn set_state(&mut self, state: ConnectionState) {
        self.state = state;
    }

    pub fn set_sync(&mut self, tick: usize) {
        self.last_sync_tick = tick;
    }

    pub fn take_current_messages(&mut self) -> Vec<NetworkMessage> {
        self.channel.lock().unwrap().drain()
    }

    pub fn synchronize_to_queue(&mut self, queue: &TickQueue) {
        // Don't start sending ticks until the client is ready for them
        if matches!(self.state, ConnectionState::InitialConnection) {
            return;
        }

        // Send all ticks between this clients last synced tick and the latest available finalized tick
        let (last_sync_tick, messages) =
            queue.make_tick_finalization_messages(self.last_sync_tick + 1);
        // Just kind of move things forward, we'll want to get ACKs from the client later
        self.last_sync_tick = last_sync_tick;

        self.channel.lock().unwrap().send(messages.as_slice());
    }

    pub fn send_message(&mut self, message: NetworkMessage) {
        self.channel.lock().unwrap().send(&[message]);
    }
}
