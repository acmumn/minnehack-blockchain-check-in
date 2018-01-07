use std::net::SocketAddr;
#[cfg(test)]
use std::net::{SocketAddrV4, SocketAddrV6};

use arrayvec::ArrayVec;
#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use blockchain::{Block, Hash};

/// A message sent over the P2P layer.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Message {
    /// A ping, which requests a `Pong` in response. Used for discovery.
    Ping,

    /// A response to a `Ping`.
    Pong,

    /// A request for a list of connected peers.
    ListRequest,

    /// A response containing up to 8 connected peers.
    ListResponse(ArrayVec<[SocketAddr; 8]>),

    /// A request for the peer's status.
    StatusRequest,

    /// The peer's status. The fields here are the hash of the genesis block,
    /// the tip index, and the tip hash.
    StatusResponse(Hash, u64, Hash),

    /// A request for a block.
    BlockRequest(u64),

    /// A transmitted block.
    BlockResponse(Block),

    /// An announced block.
    BlockAnnounce(Block),
}

#[cfg(test)]
impl Arbitrary for Message {
    fn arbitrary<G: Gen>(gen: &mut G) -> Message {
        match gen.gen::<u8>() % 8 {
            0 => Message::Ping,
            1 => Message::Pong,
            2 => Message::ListRequest,
            3 => {
                let mut peers = ArrayVec::new();
                let num_peers = gen.gen::<usize>() % peers.capacity();
                for _ in 0..num_peers {
                    peers.push(arbitrary_addr(gen));
                }
                Message::ListResponse(peers)
            }
            4 => Message::StatusRequest,
            5 => {
                let g_hash = Hash::arbitrary(gen);
                let t_idx = u64::arbitrary(gen);
                let t_hash = Hash::arbitrary(gen);
                Message::StatusResponse(g_hash, t_idx, t_hash)
            }
            6 => Message::BlockRequest(u64::arbitrary(gen)),
            7 => Message::BlockResponse(Block::arbitrary(gen)),
            8 => Message::BlockAnnounce(Block::arbitrary(gen)),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
fn arbitrary_addr<G: Gen>(gen: &mut G) -> SocketAddr {
    let port = gen.gen::<u16>();
    if gen.gen::<bool>() {
        let mut buf = [0; 4];
        gen.fill_bytes(&mut buf);
        SocketAddr::V4(SocketAddrV4::new(buf.into(), port))
    } else {
        let mut buf = [0; 16];
        gen.fill_bytes(&mut buf);
        SocketAddr::V6(SocketAddrV6::new(buf.into(), port, 0, 0))
    }
}
