//! A useful configuration type for a peer.

use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::path::Path;

use toml::de::from_str as toml_from_str;

use errors::{ErrorKind, Result, ResultExt};

/// A peer's configuration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Config {
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

impl Config {
    /// Attempts to load the config from a file.
    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Config> {
        let path = path.as_ref();

        let mut s = String::new();
        let mut file = File::open(path)
            .chain_err(|| ErrorKind::CouldNotReadConfig(path.to_owned()))?;
        file.read_to_string(&mut s)
            .chain_err(|| ErrorKind::CouldNotReadConfig(path.to_owned()))?;
        drop(file);

        toml_from_str(&s)
            .chain_err(|| ErrorKind::CouldNotParseConfig(path.to_owned()))
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            discovery_ping_interval: 60,
            max_karma: 10,
            status_check_interval: 30,
            peers: Vec::new(),
            port: 10101,
        }
    }
}
