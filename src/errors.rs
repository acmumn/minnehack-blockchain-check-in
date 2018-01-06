#![allow(missing_docs)]

use std::io::Error as IoError;
use std::net::SocketAddr;

use p2p::Message;

error_chain!{
    errors {
        CouldNotInitLibSodium {
            description("Could not initialize libsodium")
            display("Could not initialize libsodium")
        }
        CouldNotSendMessage(msg: Message, addr: SocketAddr) {
            description("Could not send a message")
            display("Could not send the message {:?} to {}", msg, addr)
        }
        CouldNotSerializeMessage(msg: Message) {
            description("Could not serialize a message")
            display("Could not serialize the message {:?}", msg)
        }
        InvalidPacket(buf: Vec<u8>) {
            description("Received invalid packet")
            display("Received invalid packet: {:?}", buf)
        }
    }
    foreign_links {
        Io(IoError);
    }
}
