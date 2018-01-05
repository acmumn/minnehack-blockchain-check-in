//! The p2p messaging layer under the blockchain.

mod message;

use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, UdpSocket};

use serde_cbor;
use sodiumoxide::crypto::box_::{PrecomputedKey, PublicKey};

use errors::{ErrorKind, Result, ResultExt};

pub use self::message::Message;

/// Information about connected peers.
#[derive(Debug)]
struct Peers {
    connected: HashMap<PublicKey, Peer>,
    host: UdpSocket,
    seen: HashSet<PublicKey>,
}

impl Peers {
    /// Broadcasts a message to all peers.
    fn broadcast(&mut self, msg: &Message) -> Result<()> {
        for peer in self.connected.values_mut() {
            peer.send_to(msg)?;
        }
        Ok(())
    }

    /// Listens for a message.
    fn listen(&self) -> Result<(Message, PublicKey)> {
        unimplemented!()
    }
}

/// Information about a connected peer.
#[derive(Debug)]
struct Peer {
    addr: SocketAddr,
    key: PrecomputedKey,
    sock: UdpSocket,
}

impl Peer {
    /// Sends a message to the peer.
    fn send_to(&mut self, msg: &Message) -> Result<()> {
        let bytes = serde_cbor::to_vec(msg)
            .chain_err(|| ErrorKind::CouldNotSerializeMessage(msg.clone()))?;
        self.sock.send(&bytes)?;
        Ok(())
    }
}

/// The state of the peer.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum PeerState {
    /// This peer might exist; we don't know.
    Speculative,
    // TODO
}
