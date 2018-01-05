use std::io::Error as IoError;

use p2p::Message;

error_chain!{
    errors {
        CouldNotInitLibsodium {
            description("Could not initialize libsodium")
            display("Could not initialize libsodium")
        }
        CouldNotSerializeMessage(msg: Message) {
            description("Could not serialize a message")
            display("Could not serialize the message {:?}", msg)
        }
    }
    foreign_links {
        Io(IoError);
    }
}
