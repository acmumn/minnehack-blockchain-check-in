//! Memey blockchain-based written-in-Rust check-in for MinneHack.

#![warn(missing_docs)]

extern crate arrayvec;
extern crate byteorder;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;
extern crate serde_cbor;
#[macro_use]
extern crate serde_derive;
extern crate sodiumoxide;

pub mod blockchain;
pub mod cards;
mod errors;
pub mod p2p;
pub mod util;

pub use errors::{Error, ErrorKind, Result, ResultExt};

/// Initializes the library. This performs one-time startup tasks.
pub fn init() -> Result<()> {
    if !sodiumoxide::init() {
        return Err(ErrorKind::CouldNotInitLibSodium.into());
    }
    Ok(())
}
