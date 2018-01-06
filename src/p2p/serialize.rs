use std::cmp::min;
use std::io::{Result, Write};
use std::net::SocketAddr;

use nom::{IResult, le_u8, le_u16};

use p2p::Message;

impl Message {
    /// Serializes the `Message` to a `Write`.
    pub fn write_to<W: Write>(&self, mut w: W) -> Result<()> {
        match *self {
            Message::Ping => w.write_all(&[0x00]),
            Message::Pong => w.write_all(&[0x01]),
            Message::PeerListRequest => w.write_all(&[0x02]),
            Message::PeerListResponse(ref peers) => {
                let l = min(8, peers.len());
                w.write_all(&[0x03, l as u8])?;
                for i in 0..l {
                    w.write_all(&(peers[i].0).0)?;
                    write_addr_to(peers[i].1, &mut w)?;
                }
                Ok(())
            }
        }
    }
}

fn write_addr_to<W: Write>(addr: SocketAddr, w: &mut W) -> Result<()> {
    let port = match addr {
        SocketAddr::V4(addr) => {
            w.write_all(&[0x04])?;
            let ip = addr.ip().octets();
            w.write_all(&ip)?;
            addr.port()
        }
        SocketAddr::V6(addr) => {
            w.write_all(&[0x06])?;
            let ip = addr.ip().octets();
            w.write_all(&ip)?;
            addr.port()
        }
    };
    w.write_all(&[
        (port & 0xff) as u8,
        (port >> 8) as u8,
    ])
}
