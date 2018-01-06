//! The p2p messaging layer under the blockchain.

mod message;
mod parse;
mod serialize;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::sync::RwLock;

use errors::{ErrorKind, Result, ResultExt};

pub use self::message::Message;

/// A P2P client.
pub struct P2P {
    /// All known peers.
    pub peers: RwLock<HashMap<SocketAddr, PeerState>>,

    socket: UdpSocket,
}

impl P2P {
    /// Broadcasts a message to all peers.
    pub fn broadcast(&self, msg: &Message) -> Result<()> {
        for addr in self.peers.read().unwrap().keys() {
            self.send_to(*addr, msg)?;
        }
        Ok(())
    }

    /// Listens for a message. Blocks until a message is received.
    pub fn listen(&self) -> Result<(SocketAddr, Message)> {
        let mut buf = [0; 0x10000];
        let (len, addr) = self.socket.recv_from(&mut buf)?;
        let buf = &buf[..len];

        let msg = Message::parse_from(buf)
            .ok_or_else(|| ErrorKind::InvalidPacket(buf.to_vec()))?;
        debug!("Got message {:?} from {}", msg, addr);
        Ok((addr, msg))
    }

    /// Creates a new `P2P` instance with the default port (`10101`).
    pub fn new() -> Result<P2P> {
        P2P::with_port(10101)
    }

    /// Broadcasts a discovery message. This only helps to discover peers on
    /// the same LAN, and only for IPv4.
    pub fn send_discovery_broadcast(&self) -> Result<()> {
        self.send_to(([0xff; 4], 10101).into(), &Message::Ping)
    }

    /// Sends a message to the peer with the given public key.
    pub fn send_to(&self, addr: SocketAddr, msg: &Message) -> Result<()> {
        let mut buf = Vec::new();
        msg.write_to(&mut buf).unwrap();
        self.socket
            .send_to(&buf, &addr)
            .chain_err(|| ErrorKind::CouldNotSendMessage(msg.clone(), addr))
            .map(|_| ())
    }

    /// Creates a new `P2P` instance with the given port.
    pub fn with_port(port: u16) -> Result<P2P> {
        let ipv4_unspecified: Ipv4Addr = [0; 4].into();
        let ipv6_unspecified: Ipv6Addr = [0; 16].into();
        let addrs: &[SocketAddr] = &[
            (ipv4_unspecified, port).into(),
            (ipv6_unspecified, port).into(),
        ];
        let socket = UdpSocket::bind(addrs)?;
        socket.set_broadcast(true)?;

        Ok(P2P {
            peers: RwLock::new(HashMap::new()),
            socket,
        })
    }
}

/// The state of the peer.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PeerState {
    /// This peer might exist; we don't know.
    Speculative,

    /// This peer exists.
    Confirmed,
}
