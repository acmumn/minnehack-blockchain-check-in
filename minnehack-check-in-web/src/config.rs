use std::net::IpAddr;

use minnehack_check_in::Config as PeerConfig;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Config {
    /// The address to serve on.
    pub addr: IpAddr,

    /// The port to serve on.
    pub port: u16,

    /// The settings for the peer.
    pub peer: PeerConfig,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            addr: [0, 0, 0, 0].into(),
            port: 8080,
            peer: PeerConfig::default(),
        }
    }
}
