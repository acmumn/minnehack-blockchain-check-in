//! The p2p messaging layer under the blockchain.

mod message;
pub(crate) mod parse;
mod serialize;
#[cfg(test)]
mod tests;

use std::net::{SocketAddr, UdpSocket};

use blockchain::Hash;
use errors::{ErrorKind, Result, ResultExt};

pub use self::message::Message;

/// A client for the P2P protocol.
#[derive(Debug)]
pub struct P2P {
    port: u16,
    socket: UdpSocket,
}

impl P2P {
    /// Creates a new `P2P` instance with the default port (`10101`).
    pub fn new() -> Result<P2P> {
        P2P::with_port(10101)
    }

    /// Waits for a message, blocking until one is received.
    pub fn recv(&self) -> Result<(SocketAddr, Message)> {
        let mut buf = [0; 0x10000];
        let (len, addr) = self.socket
            .recv_from(&mut buf)
            .chain_err(|| ErrorKind::CouldNotRecvMessage)?;
        let buf = &buf[..len];

        let msg = Message::parse_from(&buf)
            .chain_err(|| ErrorKind::InvalidPacket(buf.to_vec()))?;
        Ok((addr, msg))
    }

    /// Broadcasts a discovery message. This only helps to discover peers on
    /// the same LAN, and only for IPv4.
    pub fn send_discovery(&self) -> Result<()> {
        let addr = ([0xff; 4], self.port).into();
        self.send(addr, &Message::Ping)
    }

    /// Sends a message to the peer.
    pub fn send(&self, addr: SocketAddr, msg: &Message) -> Result<()> {
        let mut buf = Vec::new();
        msg.write_to(&mut buf).unwrap();
        self.socket
            .send_to(&buf, &addr)
            .chain_err(|| ErrorKind::CouldNotSendMessage(msg.clone(), addr))
            .map(|_| ())
    }

    /// Creates a new `P2P` instance with the given port.
    pub fn with_port(port: u16) -> Result<P2P> {
        let addr = SocketAddr::from(([0; 4], port));
        let socket = UdpSocket::bind(&addr)
            .chain_err(|| ErrorKind::CouldNotStartListener)?;
        socket
            .set_broadcast(true)
            .chain_err(|| ErrorKind::CouldNotStartListener)?;

        Ok(P2P { port, socket })
    }
}

/// Information about a peer.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Peer {
    /// The peer's address.
    pub addr: SocketAddr,

    /// The peer's karma.
    ///
    /// Karma is expended every time we send a packet, and increased every time
    /// we receive one. If it reaches a configured maximum value, the peer's
    /// status is set back to `Speculative`.
    pub karma: usize,

    /// The peer's state.
    pub state: PeerState,
}

impl Peer {
    /// Creates a new Peer.
    pub fn new(addr: SocketAddr) -> Peer {
        Peer {
            addr,
            karma: 0,
            state: PeerState::Speculative,
        }
    }

    /// Returns whether the peer has been confirmed to be on the same
    /// blockchain as us.
    pub fn same_blockchain(&self) -> bool {
        if let PeerState::Confirmed(_, _) = self.state {
            true
        } else {
            false
        }
    }
}

/// The state of the peer.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PeerState {
    /// This peer might exist; we don't know.
    Speculative,

    /// This peer exists, but might be on a different blockchain from us.
    Existent,

    /// This peer exists and is on the same blockchain as us. Their tip index
    /// and hash are the parameters.
    Confirmed(u64, Hash),

    /// The peer is on another blockchain or is blocked for other reasons.
    Ignore,
}
