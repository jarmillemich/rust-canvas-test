use web_sys::{RtcDataChannel, RtcPeerConnection};

use super::peer_connection::PeerConnection;

/// On the host side, a connection to a client
pub struct ConnectionToClient {
    connection: RtcPeerConnection,
    channel: RtcDataChannel,
}

impl ConnectionToClient {
    pub fn new(connection: RtcPeerConnection, channel: RtcDataChannel) -> Self {
        Self {
            connection,
            channel,
        }
    }
}
