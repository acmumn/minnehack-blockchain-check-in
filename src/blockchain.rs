//! The implementation of the actual blockchain, which manages a distributed
//! set.

use std::time::{SystemTime, UNIX_EPOCH};

use byteorder::{ByteOrder, LE};
use sodiumoxide::crypto::hash::sha512::{Digest, State};

const ZERO_DIGEST: Digest = Digest([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);

/// A block on the blockchain.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    index: u64,
    prev_hash: Digest,
    timestamp: u64,
    data: Vec<u8>,

    hash: Digest,
}

impl Block {
    /// Calculates the hash that this block should have.
    pub fn calc_hash(&self) -> Digest {
        hash_block(self.index, &self.prev_hash, self.timestamp, &self.data)
    }

    /// Creates a new block appended onto the current one.
    pub fn create(&self, data: Vec<u8>) -> Block {
        let index = self.index + 1;
        let prev_hash = self.hash;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let hash = hash_block(index, &prev_hash, timestamp, &data);

        Block {
            index,
            prev_hash,
            timestamp,
            data,
            hash,
        }
    }

    /// Updates the hash value to be correct.
    pub fn update_hash(&mut self) {
        self.hash = self.calc_hash();
    }

    /// Checks if this block's hash is internally consistent.
    pub fn valid(&self) -> bool {
        self.calc_hash() == self.hash
    }

    /// Checks if another block is a valid "next block" relative to this block.
    pub fn valid_next(&self, next: &Block) -> bool {
        if self.index + 1 != next.index {
            false
        } else if self.hash != next.prev_hash {
            false
        } else {
            next.valid()
        }
    }
}

/// A blockchain.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Chain {
    genesis: Block,
    blocks: Vec<Block>,
}

impl Chain {
    /// Returns a reference to the last block in the chain.
    pub fn last(&self) -> &Block {
        self.blocks.last().unwrap_or(&self.genesis)
    }

    /// Creates a new Chain with the default genesis block.
    pub fn new() -> Chain {
        let mut genesis = Block {
            index: 0,
            prev_hash: ZERO_DIGEST,
            timestamp: 1515140055,
            data: "Hello, world!".into(),
            hash: ZERO_DIGEST,
        };
        genesis.update_hash();
        Chain::with_genesis(genesis)
    }

    /// Pushes a new block onto the chain. Returns whether the block was pushed
    /// or not.
    pub fn push(&mut self, block: Block) -> bool {
        if self.valid_tail(&block) {
            self.blocks.push(block);
            true
        } else {
            false
        }
    }

    /// If `other` is a valid chain of greater length than `self`, returns
    /// `other`. Otherwise, returns `self`.
    pub fn subsume(self, other: Chain) -> Chain {
        if other.validate() && other.blocks.len() > self.blocks.len() {
            other
        } else {
            self
        }
    }

    /// Returns whether the chain is valid.
    pub fn validate(&self) -> bool {
        let mut prev = &self.genesis;
        for block in &self.blocks {
            if prev.valid_next(block) {
                prev = block;
            } else {
                return false;
            }
        }
        self.genesis.index == 0
    }

    /// Returns whether the given block is valid as the next block in the
    /// chain.
    pub fn valid_tail(&self, block: &Block) -> bool {
        self.last().valid_next(block)
    }

    /// Creates a new Chain with the given genesis block.
    pub fn with_genesis(genesis: Block) -> Chain {
        Chain {
            genesis,
            blocks: Vec::new(),
        }
    }
}

/// Hashes the components of a block.
fn hash_block(
    index: u64,
    prev_hash: &Digest,
    timestamp: u64,
    data: &[u8],
) -> Digest {
    let mut buf: [u8; 8] = Default::default();
    let mut state = State::new();

    LE::write_u64(&mut buf, index);
    state.update(&buf);

    state.update(&prev_hash.0);

    LE::write_u64(&mut buf, timestamp);
    state.update(&buf);

    state.update(data);

    state.finalize()
}
