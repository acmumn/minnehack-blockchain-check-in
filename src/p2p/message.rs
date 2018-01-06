use std::net::SocketAddr;
#[cfg(test)]
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6};

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};
use sodiumoxide::crypto::box_::PublicKey;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Message {
    Ping,
    Pong,

    PeerListRequest,
    PeerListResponse(Vec<(PublicKey, SocketAddr)>),
}

#[cfg(test)]
impl Arbitrary for Message {
    fn arbitrary<G: Gen>(gen: &mut G) -> Message {
        match gen.gen::<u8>() % 4 {
            0 => Message::Ping,
            1 => Message::Pong,
            2 => Message::PeerListRequest,
            3 => {
                let num_peers = gen.gen::<u8>() % 8;
                let mut peers = Vec::new();
                for _ in 0..num_peers {
                    let key = arbitrary_key(gen);
                    let addr = arbitrary_addr(gen);
                    peers.push((key, addr));
                }
                Message::PeerListResponse(peers)
            },
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

#[cfg(test)]
fn arbitrary_key<G: Gen>(gen: &mut G) -> PublicKey {
    let mut key = [0; 32];
    gen.fill_bytes(&mut key);
    PublicKey(key)
}
