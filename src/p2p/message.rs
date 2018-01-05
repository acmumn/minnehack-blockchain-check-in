use std::net::SocketAddr;

use sodiumoxide::crypto::box_::PublicKey;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Message {
    Discovery(PublicKey),

    Ping,
    Pong,

    PeerListRequest,
    PeerListResponse(Vec<(SocketAddr, PublicKey)>),
}
