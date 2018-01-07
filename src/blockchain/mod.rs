//! The implementation of the actual blockchain, which manages a distributed
//! set.

#[cfg(test)]
mod tests;

use std::cmp::max;
use std::time::{SystemTime, UNIX_EPOCH};

use byteorder::{ByteOrder, LE};
use crypto::digest::Digest;
use crypto::sha2::Sha256;

/// A SHA-256 hash.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Hash(pub [u8; 32]);

/// The zero hash.
pub const ZERO_HASH: Hash = Hash([0; 32]);

/// A block on the blockchain.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Block {
    index: u64,
    prev_hash: Hash,

    /// The time at which the block's hash was computed. This may be updated
    /// when a fork occurs.
    pub timestamp: u64,

    /// The data in the block.
    pub data: Vec<u8>,

    hash: Hash,
}

impl Block {
    /// Calculates the hash that this block should have.
    pub fn calc_hash(&self) -> Hash {
        hash_block(self.index, &self.prev_hash, self.timestamp, &self.data)
    }

    /// Creates a new block appended onto the current one with the given data.
    pub fn create(&self, data: Vec<u8>) -> Block {
        self.create_at(data, timestamp())
    }

    /// Creates a new block appended onto the current one with the given data
    /// and timestamp.
    pub fn create_at(&self, data: Vec<u8>, timestamp: u64) -> Block {
        let index = self.index + 1;
        let prev_hash = self.hash;
        let hash = hash_block(index, &prev_hash, timestamp, &data);

        Block {
            index,
            prev_hash,
            timestamp,
            data,
            hash,
        }
    }

    /// Checks if this block's hash is internally consistent.
    pub fn is_valid(&self) -> bool {
        self.calc_hash() == self.hash
    }

    /// Updates the hash value to be correct.
    pub fn update_hash(&mut self) {
        self.hash = self.calc_hash();
    }

    /// Checks if another block is a valid "next block" relative to this block.
    pub fn valid_next(&self, next: &Block) -> bool {
        if self.index + 1 != next.index {
            false
        } else if self.hash != next.prev_hash {
            false
        } else {
            next.is_valid()
        }
    }
}

/// A blockchain.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Chain {
    genesis: Block,
    blocks: Vec<Block>,
}

impl Chain {
    /// Combines the two blockchains deterministically. The point at which they
    /// fork is found, and the greater block (by `Block`'s `Ord` impl) is moved
    /// to the end. The blockchains must share a genesis block and both be
    /// valid.
    pub fn combine(mut self, mut other: Chain) -> Chain {
        if let Some(i) = self.find_fork(&other) {
            let l = self.blocks.drain(i..).collect::<Vec<_>>();
            let r = other.blocks.drain(i..).collect::<Vec<_>>();
            drop(other);

            // TODO: Rewrite this once NLLs are stable.
            let l_is_less = if let (Some(l), Some(r)) = (l.get(0), r.get(0)) {
                l <= r
            } else {
                true
            };

            if l_is_less {
                self.blocks.extend(l);
                r.into_iter().map(|b| b.data).for_each(|d| self.mine(d));
            } else {
                self.blocks.extend(r);
                l.into_iter().map(|b| b.data).for_each(|d| self.mine(d));
            }
            self
        } else {
            self
        }
    }

    /// Finds the position at which two chains diverge. The blockchains must
    /// share a genesis block and both be valid.
    pub fn find_fork(&self, other: &Chain) -> Option<usize> {
        assert_eq!(self.genesis, other.genesis);
        assert!(self.is_valid());
        assert!(other.is_valid());

        for i in 0..max(self.blocks.len(), other.blocks.len()) {
            match (self.blocks.get(i), other.blocks.get(i)) {
                (Some(l), Some(r)) => if l == r {
                    continue;
                } else {
                    return Some(i);
                },
                (Some(_), None) => return Some(i),
                (None, Some(_)) => return Some(i),
                (None, None) => unreachable!(),
            }
        }
        None
    }

    /// Returns whether the chain is valid.
    pub fn is_valid(&self) -> bool {
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

    /// Returns a reference to the last block in the chain.
    pub fn last(&self) -> &Block {
        self.blocks.last().unwrap_or(&self.genesis)
    }

    /// Mines a new block with the given data.
    pub fn mine(&mut self, data: Vec<u8>) {
        let block = self.last().create(data);
        self.blocks.push(block);
    }

    /// Mines a new block with the given data and timestamp.
    pub fn mine_at(&mut self, data: Vec<u8>, timestamp: u64) {
        let block = self.last().create_at(data, timestamp);
        self.blocks.push(block);
    }

    /// Creates a new Chain with the default genesis block.
    pub fn new() -> Chain {
        let mut genesis = Block {
            index: 0,
            prev_hash: ZERO_HASH,
            timestamp: 1515140055,
            data: "Hello, world!".into(),
            hash: ZERO_HASH,
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

impl<'a> IntoIterator for &'a Chain {
    type Item = &'a Block;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        Iter {
            chain: self,
            pos: 0,
        }
    }
}

/// An iterator over the blocks in the blockchain.
pub struct Iter<'a> {
    chain: &'a Chain,
    pos: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Block;
    fn next(&mut self) -> Option<&'a Block> {
        if self.pos == 0 {
            let block = &self.chain.genesis;
            self.pos = 1;
            Some(block)
        } else if self.pos > self.chain.blocks.len() {
            None
        } else {
            let block = &self.chain.blocks[self.pos - 1];
            self.pos += 1;
            Some(block)
        }
    }
}

/// Hashes the components of a block.
fn hash_block(
    index: u64,
    prev_hash: &Hash,
    timestamp: u64,
    data: &[u8],
) -> Hash {
    let mut buf = [0; 8];
    let mut hasher = Sha256::new();

    LE::write_u64(&mut buf, index);
    hasher.input(&buf);

    hasher.input(&prev_hash.0);

    LE::write_u64(&mut buf, timestamp);
    hasher.input(&buf);

    hasher.input(data);

    let mut hash = ZERO_HASH;
    hasher.result(&mut hash.0);
    hash
}

/// Returns the current Unix timestamp.
fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
