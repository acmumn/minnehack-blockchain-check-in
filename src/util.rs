//! Utility functions.

use arrayvec::{Array, ArrayVec};

/// Converts a `Vec` to an `ArrayVec` if possible.
pub fn to_arrayvec<A: Array>(vec: Vec<A::Item>) -> Option<ArrayVec<A>> {
    let mut arr = ArrayVec::new();
    if vec.len() > arr.capacity() {
        None
    } else {
        arr.extend(vec);
        Some(arr)
    }
}
