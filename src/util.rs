//! Utility functions.

use std::io::{Error as IoError, Write};

use arrayvec::{Array, ArrayVec};
use byteorder::{ByteOrder, LE};
use error_chain::ChainedError;

use errors::Error;

/// Logs an error, returning whether an error occurred.
pub fn log_err<E: Into<Error>>(r: Result<(), E>) -> bool {
    match r {
        Ok(_) => false,
        Err(err) => {
            error!("{}", err.into().display_chain());
            true
        }
    }
}

/// Converts a slice to an `ArrayVec<[u8; n]>` if possible.
pub fn slice_to_arrayvec<A, T>(s: &[T]) -> Option<ArrayVec<A>>
where
    A: Array<Item = T>,
    T: Clone,
{
    let mut arr = ArrayVec::new();
    if s.len() > arr.capacity() {
        None
    } else {
        arr.extend(s.iter().cloned());
        Some(arr)
    }
}

/// Converts a `&str` to an `ArrayVec<[u8; n]>` if possible.
pub fn str_to_arrayvec<A: Array<Item = u8>>(s: &str) -> Option<ArrayVec<A>> {
    let mut arr = ArrayVec::new();
    if s.len() > arr.capacity() {
        None
    } else {
        arr.extend(s.bytes());
        Some(arr)
    }
}

/// Converts a `Vec` to an `ArrayVec` if possible.
pub fn vec_to_arrayvec<A: Array>(vec: Vec<A::Item>) -> Option<ArrayVec<A>> {
    let mut arr = ArrayVec::new();
    if vec.len() > arr.capacity() {
        None
    } else {
        arr.extend(vec);
        Some(arr)
    }
}

/// Writes a little-endian `u64` to the given `Write`.
pub fn write_u64_to<W: Write>(n: u64, w: &mut W) -> Result<(), IoError> {
    let mut buf = [0; 8];
    LE::write_u64(&mut buf, n);
    w.write_all(&buf)
}
