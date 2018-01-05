extern crate byteorder;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;
extern crate serde_cbor;
#[macro_use]
extern crate serde_derive;
extern crate sodiumoxide;

pub mod blockchain;
mod errors;
pub mod p2p;

pub use errors::{Error, ErrorKind, Result, ResultExt};

/// Initializes the library. This performs one-time startup tasks.
pub fn init() -> Result<()> {
    if !sodiumoxide::init() {
        return Err(ErrorKind::CouldNotInitLibsodium.into());
    }
    Ok(())
}
