//! A useful configuration type for a peer.

use std::net::SocketAddr;

/// A peer's configuration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Config {
    /// The time to wait between asking peers for their peers.
    pub discovery_peer_interval: u64,

    /// The time to wait, in seconds, between sending discovery pings.
    pub discovery_ping_interval: u64,

    /// The maximum karma value a peer can reach before it is ignored.
    pub max_karma: usize,

    /// The time to wait, in seconds, between asking peers for status updates.
    pub status_check_interval: u64,

    /// A list of peers to connect to.
    pub peers: Vec<SocketAddr>,

    /// The port to run on.
    pub port: u16,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            discovery_peer_interval: 60,
            discovery_ping_interval: 60,
            max_karma: 10,
            status_check_interval: 30,
            peers: Vec::new(),
            port: 10101,
        }
    }
}
