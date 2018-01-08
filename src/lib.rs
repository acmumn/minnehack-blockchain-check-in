//! Memey blockchain-based written-in-Rust check-in for MinneHack.

#![warn(missing_docs)]

extern crate arrayvec;
extern crate byteorder;
extern crate crossbeam;
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
#[macro_use]
extern crate serde_derive;

pub mod blockchain;
pub mod cards;
mod errors;
pub mod p2p;
pub mod util;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread::sleep;

use arrayvec::ArrayVec;
use crossbeam::{scope, Scope};
use crossbeam::sync::MsQueue;

use blockchain::{Block, BlockStatus, Chain, Hash};
pub use errors::{Error, ErrorKind, Result, ResultExt};
use p2p::{Message, P2P, Peer, PeerState};
use util::log_err;

/// A blockchain client, using the `p2p` module for sending blocks.
#[derive(Debug)]
pub struct Client {
    /// The time to wait between sending discovery pings.
    pub discovery_ping_interval: Duration,

    /// The maximum karma value a peer can reach before it is ignored.
    pub max_karma: usize,

    /// The time to wait between asking peers for status updates.
    pub status_check_interval: Duration,

    chain: Mutex<Chain>,
    p2p: P2P,
    peers: Mutex<HashMap<SocketAddr, Peer>>,
    send_queue: Arc<MsQueue<(Option<SocketAddr>, Message)>>,
}

impl Client {
    /// Creates a new `Client` with the default options.
    pub fn new() -> Result<Client> {
        Client::new_with_opts(
            10101,
            Chain::new(),
            Duration::from_secs(60),
            Duration::from_secs(30),
            10,
        )
    }

    /// Creates a new `Client`.
    pub fn new_with_opts(
        port: u16,
        chain: Chain,
        discovery_ping_interval: Duration,
        status_check_interval: Duration,
        max_karma: usize,
    ) -> Result<Client> {
        let p2p = P2P::with_port(port)?;
        Ok(Client {
            discovery_ping_interval,
            max_karma,
            status_check_interval,

            chain: Mutex::new(chain),
            p2p,
            peers: Mutex::new(HashMap::new()),
            send_queue: Arc::new(MsQueue::new()),
        })
    }

    /// Adds a peer with the given address.
    pub fn add_peer(&self, addr: SocketAddr) {
        let mut peers = self.peers.lock().unwrap();
        let peer = peers.entry(addr).or_insert_with(|| {
            self.send_queue.push((Some(addr), Message::Ping));
            Peer::new(addr)
        });
        peer.karma = peer.karma.saturating_sub(1);
    }

    fn handle_block(&self, block: Block, broadcast: bool) {
        let mut chain = self.chain.lock().unwrap();
        match chain.status(&block) {
            // Add it and broadcast it.
            BlockStatus::ValidTip => {
                debug!("Adding and rebroadcasting block {}", block.hash);
                assert!(chain.push(block.clone()));
            }

            // Don't try adding it, but broadcast it.
            BlockStatus::PotentiallyValid => {
                debug!("Not adding (but rebroadcasting) potentially valid block {}", block.hash);
            }

            // Ignore it and don't rebroadcast.
            status => {
                debug!("Ignoring {:?} block {}", status, block.hash);
                return;
            }
        }

        if broadcast {
            let peers = self.peers.lock().unwrap();
            let idx = block.index;
            let peers = peers.values().filter(|p| {
                if let PeerState::Confirmed(i, _) = p.state {
                    i <= idx
                } else {
                    false
                }
            });
            let msg = Message::BlockAnnounce(block);
            for peer in peers {
                self.send_queue.push((Some(peer.addr), msg.clone()));
            }
        }
    }

    fn handle_peer_status(
        &self,
        addr: SocketAddr,
        genesis_hash: Hash,
        tip_index: u64,
        tip_hash: Hash,
    ) {
        let sync = {
            let chain = self.chain.lock().unwrap();
            let mut peers = self.peers.lock().unwrap();

            if chain.genesis().hash == genesis_hash {
                peers.entry(addr).or_insert_with(|| Peer::new(addr)).state =
                    PeerState::Confirmed(tip_index, tip_hash);
                true
            } else {
                peers.entry(addr).or_insert_with(|| Peer::new(addr)).state =
                    PeerState::Ignore;
                false
            }
        };
        if sync {
            self.sync_with_peer(addr)
        }
    }

    fn mark_peer_exists(&self, addr: SocketAddr) {
        let mut peers = self.peers.lock().unwrap();
        let peer = peers.entry(addr).or_insert_with(|| {
            warn!(
                "Peer should be Speculative before it gets marked as Existent"
            );
            let mut peer = Peer::new(addr);
            peer.state = PeerState::Existent;
            peer
        });
        if peer.state == PeerState::Speculative {
            peer.state = PeerState::Existent;
            self.send_queue.push((Some(addr), Message::StatusRequest));
        }
    }

    fn sync_with_peer(&self, addr: SocketAddr) {
        // let mut chain = self.chain.lock().unwrap();
        let peers = self.peers.lock().unwrap();

        let peer = peers[&addr];
        warn!("TODO Sync with {:?}", peer)
    }

    /// Mines a new block with the given data.
    pub fn mine(&self, data: ArrayVec<[u8; 256]>) {
        let mut chain = self.chain.lock().unwrap();
        let block = chain.mine(data);
        info!("Mined block {}", block.hash);

        self.send_queue
            .push((None, Message::BlockAnnounce(block.clone())));
    }

    /// Runs the `Client` alongside the threads spawned by `spawn_others`.
    pub fn run_with<F>(&self, spawn_others: F)
    where
        F: FnOnce(&Scope, Arc<MsQueue<(Option<SocketAddr>, Message)>>),
    {
        scope(|scope| {
            scope.spawn(|| loop {
                // Sender thread
                let (addr, msg) = self.send_queue.pop();
                if let Some(addr) = addr {
                    log_err(self.p2p.send(addr, &msg));
                } else {
                    let peers = self.peers.lock().unwrap();
                    let peers = peers.values().filter(|p| p.same_blockchain());
                    for peer in peers {
                        log_err(self.p2p.send(peer.addr, &msg));
                    }
                }
            });
            scope.spawn(|| loop {
                // Receiver thread
                match self.p2p.recv() {
                    Ok((addr, msg)) => {
                        debug!("{} sent {:?}", addr, msg);
                        self.add_peer(addr);
                        match msg {
                            Message::Ping => {
                                self.send_queue
                                    .push((Some(addr), Message::Pong));
                            }
                            Message::Pong => {
                                self.mark_peer_exists(addr);
                            }
                            Message::PeerRequest => warn!("TODO PeerRequest"),
                            Message::PeerResponse(peers) => {
                                warn!("TODO PeerResponse({:?})", peers);
                            }
                            Message::StatusRequest => {
                                let chain = self.chain.lock().unwrap();
                                let gh = chain.genesis().hash;
                                let tip = chain.tip();
                                let ti = tip.index;
                                let th = tip.hash;
                                let msg = Message::StatusResponse(gh, ti, th);
                                self.send_queue.push((Some(addr), msg));
                            }
                            Message::StatusResponse(gh, ti, th) => {
                                self.handle_peer_status(addr, gh, ti, th);
                            }
                            Message::BlockRequest(idx) => {
                                let chain = self.chain.lock().unwrap();
                                if idx < chain.len() {
                                    self.send_queue.push((
                                        Some(addr),
                                        Message::BlockResponse(
                                            chain[idx].clone(),
                                        ),
                                    ));
                                }
                            }
                            Message::BlockResponse(block) => {
                                self.handle_block(block, false);
                            }
                            Message::BlockAnnounce(block) => {
                                self.handle_block(block, true);
                            }
                        }
                    }
                    Err(err) => {
                        log_err(Err(err));
                    }
                }
            });
            scope.spawn(|| loop {
                // Discovery thread
                debug!("Sending discovery ping...");
                log_err(self.p2p.send_discovery());
                sleep(self.discovery_ping_interval);
            });
            scope.spawn(|| loop {
                // Status check thread
                debug!("Asking peers for status updates...");
                self.send_queue.push((None, Message::StatusRequest));
                sleep(self.status_check_interval);
            });
            spawn_others(scope, self.send_queue.clone());
        })
    }

    /// Runs the `Client` alongside the thread given by the function.
    pub fn run_with_one<F>(&self, thread: F)
    where
        F: 'static + FnOnce(&MsQueue<(Option<SocketAddr>, Message)>) + Send,
    {
        self.run_with(|scope, send_queue| {
            scope.spawn(move || thread(&send_queue));
        })
    }

    /// Runs the given closure with the blockchain as an argument.
    pub fn with_chain<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Chain) -> T,
    {
        let chain = self.chain.lock().unwrap();
        f(&chain)
    }

    /// Runs the given closure with the peer list as an argument.
    pub fn with_peers<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&HashMap<SocketAddr, Peer>) -> T,
    {
        let peers = self.peers.lock().unwrap();
        f(&peers)
    }
}
