#![allow(missing_docs)]

use std::net::SocketAddr;
use std::path::PathBuf;

use p2p::Message;

error_chain!{
    errors {
        CouldNotParseConfig(path: PathBuf) {
            description("Could not parse the config")
            display("Could not parse the config in {}", path.display())
        }
        CouldNotReadConfig(path: PathBuf) {
            description("Could not read the config")
            display("Could not read the config in {}", path.display())
        }
        CouldNotRecvMessage {
            description("Could not receive a message")
            display("Could not receive a message")
        }
        CouldNotSendMessage(msg: Message, addr: SocketAddr) {
            description("Could not send a message")
            display("Could not send the message {:?} to {}", msg, addr)
        }
        CouldNotSerializeMessage(msg: Message) {
            description("Could not serialize a message")
            display("Could not serialize the message {:?}", msg)
        }
        CouldNotStartListener {
            description("Could not start listener")
            display("Could not start listener")
        }
        InvalidPacket(buf: Vec<u8>) {
            description("Received invalid packet")
            display("Received invalid packet: {:?}", buf)
        }
    }
}
