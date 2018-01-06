//! The p2p messaging layer under the blockchain.

mod message;
mod parse;
mod serialize;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};

use sodiumoxide::crypto::box_::{precompute, PrecomputedKey, PublicKey, SecretKey};

use errors::{ErrorKind, Result, ResultExt};

pub use self::message::Message;
pub use self::parse::parse_packet;

/// A P2P client.
#[derive(Debug)]
pub struct P2P {
    pub connected: HashMap<PublicKey, Peer>,
    pub public_key: PublicKey,
    secret_key: SecretKey,
    pub socket: UdpSocket,
}

impl P2P {
    /// Broadcasts a message to all peers.
    pub fn broadcast(&self, msg: &Message) -> Result<()> {
        for key in self.connected.keys() {
            self.send_to(key, msg)?;
        }
        Ok(())
    }

    /// Listens for a message. Blocks until a message is received.
    pub fn listen(&mut self) -> Result<(PublicKey, Message)> {
        let mut buf = [0; 0x10000];
        let (len, addr) = self.socket.recv_from(&mut buf)?;
        let buf = &buf[..len];

        let (pub_key, msg) = parse_packet(buf)
            .ok_or_else(|| ErrorKind::InvalidPacket(buf.to_vec()))?;
        let secret_key = &self.secret_key;
        self.connected.entry(pub_key).or_insert_with(|| Peer {
            addr,
            key: precompute(&pub_key, secret_key),
            state: PeerState::Speculative,
        });

        unimplemented!()
    }

    /// Sends a message to the peer with the given public key.
    pub fn send_to(&self, pub_key: &PublicKey, msg: &Message) -> Result<()> {
        unimplemented!()
    }
}

/// Information about a connected peer.
#[derive(Debug)]
pub struct Peer {
    pub addr: SocketAddr,
    key: PrecomputedKey,
    pub state: PeerState,
}

/// The state of the peer.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PeerState {
    /// This peer might exist; we don't know.
    Speculative,

    // TODO
}
