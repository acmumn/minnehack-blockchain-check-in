use std::io::{Result, Write};
use std::net::SocketAddr;

use byteorder::{ByteOrder, LE};

use p2p::Message;
use util::write_u64_to;

impl Message {
    /// Serializes the `Message` to a `Write`.
    pub fn write_to<W: Write>(&self, mut w: W) -> Result<()> {
        match *self {
            Message::Ping => w.write_all(&[0x00]),
            Message::Pong => w.write_all(&[0x01]),
            Message::PeerRequest => w.write_all(&[0x02]),
            Message::PeerResponse(ref peers) => {
                let l = peers.len();
                assert!(l < 256);
                w.write_all(&[0x03, l as u8])?;
                for i in 0..l {
                    write_addr_to(peers[i], &mut w)?;
                }
                Ok(())
            }
            Message::StatusRequest => w.write_all(&[0x04]),
            Message::StatusResponse(ref g_hash, t_idx, ref t_hash) => {
                w.write_all(&[0x05])?;
                w.write_all(&g_hash.0)?;
                write_u64_to(t_idx, &mut w)?;
                w.write_all(&t_hash.0)
            }
            Message::BlockRequest(idx) => {
                w.write_all(&[0x06])?;
                write_u64_to(idx, &mut w)
            }
            Message::BlockResponse(ref block) => {
                w.write_all(&[0x07])?;
                block.write_to(w)
            }
            Message::BlockAnnounce(ref block) => {
                w.write_all(&[0x08])?;
                block.write_to(w)
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

    let mut buf = [0; 2];
    LE::write_u16(&mut buf, port);
    w.write_all(&buf)
}
