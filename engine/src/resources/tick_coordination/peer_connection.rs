#![allow(dead_code)]
use futures::channel::oneshot;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Error};
use serde_json::json;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{
    RtcConfiguration, RtcDataChannel, RtcIceCandidate, RtcIceGatheringState, RtcPeerConnection,
};

// TODO forget it, we're just going to establish this in JS for the moment

/// A wrapper around an RtcPeerConnection with helpers for establishment
pub struct PeerConnection {
    state: PeerConnectionState,
    connection: RtcPeerConnection,
    channel: Option<RtcDataChannel>,
}

pub enum PeerConnectionState {
    Idle,
    Connecting,
    Connected,
    Ended,
}

impl Default for PeerConnection {
    fn default() -> Self {
        // Our default ICE server config
        let mut config = RtcConfiguration::new();
        config.ice_servers(
            &serde_wasm_bindgen::to_value(&json!({
                "iceServers": [ {
                    "urls": "stun:stun3.l.google.com:19302"
                } ]
            }))
            .unwrap(),
        );

        let connection = RtcPeerConnection::new_with_configuration(&config)
            .expect("Should be able to construct an RtcPeerConnection");

        PeerConnection {
            state: PeerConnectionState::Idle,
            connection,
            channel: None,
        }
    }
}

/// Gathers ICE candidates for the current WebRTC connection.
///
/// Returns a vector of ICE candidate strings that can be used to establish a peer-to-peer
/// connection between two devices. This function blocks until all ICE candidates have been
/// gathered.
async fn gather_ice_candidates(connection: &RtcPeerConnection) -> Result<Vec<String>, Error> {
    assert_eq!(
        connection.ice_gathering_state(),
        RtcIceGatheringState::New,
        "Should not start gathering ICE candidates twice"
    );

    let candidates = Arc::new(Mutex::new(Some(Vec::new())));

    let (sender, receiver) = oneshot::channel::<Vec<String>>();
    let mut sender = Some(sender);

    let on_ice_candidate =
        Closure::<dyn FnMut(_)>::new(move |candidate: Option<RtcIceCandidate>| {
            let mut end_candidates = || {
                // End of candidates
                let mutex = candidates.lock().unwrap().take().unwrap();
                sender.take().unwrap().send(mutex).unwrap();
            };

            let add_candidate = |candidate: String| {
                // An actual candidate
                candidates.lock().unwrap().as_mut().unwrap().push(candidate);
            };

            match candidate.map(|c| c.candidate()) {
                None => end_candidates(),
                Some(s) if s.is_empty() => end_candidates(),
                Some(candidate) => add_candidate(candidate),
            }
        });

    connection.set_onicecandidate(Some(on_ice_candidate.as_ref().unchecked_ref()));

    let ret = receiver.await.map_err(|_| anyhow!("ICE gathering failed"));

    connection.set_onicecandidate(None);

    ret
}
