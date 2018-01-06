use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use nom::{IResult, le_u8, le_u16};
use sodiumoxide::crypto::box_::PublicKey;

use p2p::Message;

/// Parses a packet from a buffer.
pub fn parse_packet(buf: &[u8]) -> Option<(PublicKey, Message)> {
    match packet(buf) {
        IResult::Done(rest, packet) => if rest.is_empty() {
            Some(packet)
        } else {
            None
        },
        _ => None,
    }
}

impl Message {
    /// Attempts to parse a `Message` from a buffer.
    pub fn parse_from(msg: &[u8]) -> Option<Message> {
        match message(msg) {
            IResult::Done(rest, msg) => if rest.is_empty() {
                Some(msg)
            } else {
                None
            },
            _ => None,
        }
    }
}

named!(packet(&[u8]) -> (PublicKey, Message), pair!(pub_key, message));
named!(message(&[u8]) -> Message, alt_complete!(
    ping | pong | peer_list_request | peer_list_response
));

named!(ping(&[u8]) -> Message, map!(tag!([0x00]), |_| Message::Ping));
named!(pong(&[u8]) -> Message, map!(tag!([0x01]), |_| Message::Pong));
named!(peer_list_request(&[u8]) -> Message,
    map!(tag!([0x02]), |_| Message::PeerListRequest));
named!(peer_list_response(&[u8]) -> Message, do_parse!(
    tag!([0x03]) >>
    peers: length_count!(le_u8, peer) >>
    ( Message::PeerListResponse(peers) )));

named!(peer(&[u8]) -> (PublicKey, SocketAddr), pair!(pub_key, sock_addr));
named!(pub_key(&[u8]) -> PublicKey,
    map!(count_fixed!(u8, le_u8, 32), PublicKey));

named!(sock_addr(&[u8]) -> SocketAddr, alt_complete!(
    map!(sock_addr_4, SocketAddr::V4) |
    map!(sock_addr_6, SocketAddr::V6)
));
named!(sock_addr_4(&[u8]) -> SocketAddrV4, do_parse!(
    tag!([0x04]) >>
    addr: ipv4 >>
    port: le_u16 >>
    ( SocketAddrV4::new(addr, port) )));
named!(sock_addr_6(&[u8]) -> SocketAddrV6, do_parse!(
    tag!([0x06]) >>
    addr: ipv6 >>
    port: le_u16 >>
    ( SocketAddrV6::new(addr, port, 0, 0) )));
named!(ipv4(&[u8]) -> Ipv4Addr,
    map!(count_fixed!(u8, le_u8, 4), Ipv4Addr::from));
named!(ipv6(&[u8]) -> Ipv6Addr,
    map!(count_fixed!(u8, le_u8, 16), Ipv6Addr::from));
