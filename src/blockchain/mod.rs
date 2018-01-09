//! The implementation of the actual blockchain.

pub(crate) mod parse;
mod serialize;
#[cfg(test)]
mod tests;

use std::cmp::{max, Ordering};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::Index;
use std::time::{SystemTime, UNIX_EPOCH};

use arrayvec::ArrayVec;
use byteorder::{ByteOrder, LE};
use crypto::digest::Digest;
use crypto::sha2::Sha256;
#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

use util::str_to_arrayvec;

/// A SHA-256 hash.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd,
         Serialize)]
pub struct Hash(pub [u8; 32]);

/// The zero hash.
pub const ZERO_HASH: Hash = Hash([0; 32]);

#[cfg(test)]
impl Arbitrary for Hash {
    fn arbitrary<G: Gen>(gen: &mut G) -> Hash {
        let mut buf = [0; 32];
        gen.fill_bytes(&mut buf);
        Hash(buf)
    }
}

impl Display for Hash {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        for b in self.0.iter() {
            write!(fmt, "{:02x}", b)?;
        }
        Ok(())
    }
}

/// A block on the blockchain.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Block {
    /// The index of the block in the chain.
    pub index: u64,

    /// The hash of the previous block in the chain.
    pub prev_hash: Hash,

    /// The time at which the block's hash was computed. This may be updated
    /// when a fork occurs.
    pub timestamp: u64,

    /// The hash of this block.
    pub hash: Hash,

    /// The data in the block.
    pub data: ArrayVec<[u8; 256]>,
}

impl Block {
    /// Creates a new block appended onto the current one with the given data.
    pub fn create(&self, data: ArrayVec<[u8; 256]>) -> Block {
        self.create_at(now(), data)
    }

    /// Creates a new block appended onto the current one with the given data
    /// and timestamp.
    pub fn create_at(
        &self,
        timestamp: u64,
        data: ArrayVec<[u8; 256]>,
    ) -> Block {
        Block::new(self.index + 1, self.hash, timestamp, data)
    }

    /// Checks if this block's hash is internally consistent.
    pub fn is_valid(&self) -> bool {
        hash_block(self.index, &self.prev_hash, self.timestamp, &self.data)
            == self.hash
    }

    /// Creates a new block with the given values.
    pub fn new(
        index: u64,
        prev_hash: Hash,
        timestamp: u64,
        data: ArrayVec<[u8; 256]>,
    ) -> Block {
        let hash = hash_block(index, &prev_hash, timestamp, &data);
        Block {
            index,
            prev_hash,
            timestamp,
            data,
            hash,
        }
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

#[cfg(test)]
impl Arbitrary for Block {
    fn arbitrary<G: Gen>(gen: &mut G) -> Block {
        let len = gen.gen::<u8>();
        let mut data = ArrayVec::new();
        for _ in 0..len {
            data.push(gen.gen());
        }

        Block::new(
            u64::arbitrary(gen),
            Hash::arbitrary(gen),
            u64::arbitrary(gen),
            data,
        )
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
            let i_usize = i as usize;
            let l = self.blocks.drain(i_usize..).collect::<Vec<_>>();
            let r = other.blocks.drain(i_usize..).collect::<Vec<_>>();
            drop(other);

            // TODO: Rewrite this once NLLs are stable.
            let l_is_less = if let (Some(l), Some(r)) = (l.get(0), r.get(0)) {
                l <= r
            } else {
                true
            };

            if l_is_less {
                self.blocks.extend(l);
                for block in r {
                    self.mine(block.data);
                }
            } else {
                self.blocks.extend(r);
                for block in l {
                    self.mine(block.data);
                }
            }
            self
        } else {
            self
        }
    }

    /// Finds the position at which two chains diverge. The blockchains must
    /// share a genesis block and both be valid.
    pub fn find_fork(&self, other: &Chain) -> Option<u64> {
        assert_eq!(self.genesis, other.genesis);
        assert!(self.is_valid());
        assert!(other.is_valid());

        for i in 0..max(self.blocks.len(), other.blocks.len()) {
            match (self.blocks.get(i), other.blocks.get(i)) {
                (Some(l), Some(r)) => if l == r {
                    continue;
                } else {
                    return Some(i as u64);
                },
                (Some(_), None) => return Some(i as u64),
                (None, Some(_)) => return Some(i as u64),
                (None, None) => unreachable!(),
            }
        }
        None
    }

    /// Returns a reference to the first block in the chain.
    pub fn genesis(&self) -> &Block {
        &self.genesis
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

    /// Returns the number of blocks in the chain.
    pub fn len(&self) -> u64 {
        (self.blocks.len() as u64) + 1
    }

    /// Mines a new block with the given data.
    pub fn mine(&mut self, data: ArrayVec<[u8; 256]>) -> &Block {
        self.mine_at(now(), data)
    }

    /// Mines a new block with the given data and timestamp.
    pub fn mine_at(
        &mut self,
        timestamp: u64,
        data: ArrayVec<[u8; 256]>,
    ) -> &Block {
        let block = self.tip().create_at(timestamp, data);
        self.blocks.push(block);
        let block = self.blocks.last().unwrap();
        debug!("Blockchain now has {} blocks", self.len());
        block
    }

    /// Creates a new Chain with the default genesis block.
    pub fn new() -> Chain {
        Chain::with_genesis(Block::new(
            0,
            ZERO_HASH,
            1515140055,
            str_to_arrayvec("Hello, world!").unwrap(),
        ))
    }

    /// Pushes a new block onto the chain. Returns whether the block was pushed
    /// or not.
    pub fn push(&mut self, block: Block) -> bool {
        if self.valid_tip(&block) {
            self.blocks.push(block);
            true
        } else {
            false
        }
    }

    /// Gets the status of a block with respect to the chain. This chain must
    /// be valid.
    pub fn status(&self, block: &Block) -> BlockStatus {
        assert!(self.is_valid());

        match self.len().cmp(&block.index) {
            Ordering::Greater => {
                if block == &self[block.index] {
                    BlockStatus::Contained
                } else {
                    BlockStatus::Invalid
                }
            }
            Ordering::Equal => {
                if self.valid_tip(block) {
                    BlockStatus::ValidTip
                } else {
                    BlockStatus::Invalid
                }
            }
            Ordering::Less => BlockStatus::PotentiallyValid,
        }
    }

    /// Returns a reference to the last block in the chain.
    pub fn tip(&self) -> &Block {
        self.blocks.last().unwrap_or(&self.genesis)
    }

    /// Returns whether the given block is valid as the next block in the
    /// chain.
    pub fn valid_tip(&self, block: &Block) -> bool {
        self.tip().valid_next(block)
    }

    /// Creates a new Chain with the given genesis block.
    pub fn with_genesis(genesis: Block) -> Chain {
        Chain {
            genesis,
            blocks: Vec::new(),
        }
    }
}

impl Index<u64> for Chain {
    type Output = Block;
    fn index(&self, index: u64) -> &Block {
        if index == 0 {
            self.genesis()
        } else {
            &self.blocks[(index as usize) - 1]
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

/// The status of a block with regards to the chain.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BlockStatus {
    /// The block is contained within the chain.
    Contained,

    /// The block is valid to be appended to the tip of the chain.
    ValidTip,

    /// The block is potentially valid, but our chain isn't long enough to
    /// verify that.
    PotentiallyValid,

    /// The block is definitely not valid.
    Invalid,
}

/// An iterator over the blocks in the blockchain.
pub struct Iter<'a> {
    chain: &'a Chain,
    pos: u64,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Block;
    fn next(&mut self) -> Option<&'a Block> {
        if self.pos == 0 {
            let block = &self.chain.genesis;
            self.pos = 1;
            Some(block)
        } else if self.pos > (self.chain.blocks.len() as u64) {
            None
        } else {
            let block = &self.chain.blocks[(self.pos - 1) as usize];
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
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
