use nom::{IResult, le_u64, le_u8};

use blockchain::{Block, Hash};
use util::slice_to_arrayvec;

impl Block {
    /// Attempts to parse a `Block` from a buffer.
    pub fn parse_from(msg: &[u8]) -> Option<Block> {
        match block(msg) {
            IResult::Done(rest, msg) => if rest.is_empty() {
                Some(msg)
            } else {
                None
            },
            _ => None,
        }
    }
}

named!(pub block(&[u8]) -> Block, do_parse!(
    index: le_u64 >>
    prev_hash: hash >>
    timestamp: le_u64 >>
    data_len: le_u8 >>
    data: map_opt!(take!(data_len), slice_to_arrayvec) >>
    hash: hash >>
    ( Block { index, prev_hash, timestamp, hash, data })));
named!(pub hash(&[u8]) -> Hash, map!(count_fixed!(u8, le_u8, 32), Hash));
