//! Memey blockchain-based written-in-Rust check-in for MinneHack.

#![warn(missing_docs)]

extern crate arrayvec;
extern crate byteorder;
extern crate crypto;
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

pub mod blockchain;
pub mod cards;
mod errors;
pub mod p2p;
pub mod util;

pub use errors::{Error, ErrorKind, Result, ResultExt};
