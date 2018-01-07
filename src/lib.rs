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
use std::io::{stdin, BufRead, BufReader};
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::Duration;
use std::thread::sleep;

use arrayvec::ArrayVec;
use crossbeam::scope;
use crossbeam::sync::MsQueue;

use blockchain::Chain;
use cards::{parse_card, CardParse};
pub use errors::{Error, ErrorKind, Result, ResultExt};
use p2p::{Message, P2P, Peer};
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
        })
    }

    /// Adds a peer with the given address.
    pub fn add_peer(&self, addr: SocketAddr) {
        self.peers
            .lock()
            .unwrap()
            .entry(addr)
            .or_insert_with(|| Peer::new(addr));
    }

    /// Runs the `Client`.
    pub fn run(&self) -> Result<()> {
        let send_queue = MsQueue::new();
        scope(|scope| {
            let mut guards = ArrayVec::<[_; 5]>::new();

            guards.push(scope.spawn(|| -> Result<()> {
                let mut stdin = BufReader::new(stdin());
                let mut line = String::new();
                loop {
                    line.clear();
                    if log_err(stdin.read_line(&mut line).map(|_| ())) {
                        continue;
                    }

                    match parse_card(&line) {
                        CardParse::Card(fields) => info!("TODO {:#?}", fields),
                        err => error!("Error reading card: {:?}", err),
                    }
                }
            }));
            guards.push(scope.spawn(|| loop {
                let (addr, msg) = send_queue.pop();
                log_err(self.p2p.send(addr, msg));
            }));
            guards.push(scope.spawn(|| loop {
                match self.p2p.recv() {
                    Ok((addr, msg)) => {
                        debug!("{} sent {:?}", addr, msg);
                        self.add_peer(addr);
                        match msg {
                            Message::Ping => {
                                send_queue.push((addr, Message::Pong));
                            }
                            Message::Pong => {}
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
                                send_queue.push((addr, msg));
                            }
                            Message::StatusResponse(gh, ti, th) => {
                                warn!(
                                    "TODO StatusResponse({}, {}, {})",
                                    gh, ti, th
                                );
                            }
                            Message::BlockRequest(idx) => {
                                warn!("TODO BlockRequest({})", idx);
                            }
                            Message::BlockResponse(block) => {
                                warn!("TODO BlockResponse({:?})", block);
                            }
                            Message::BlockAnnounce(block) => {
                                warn!("TODO BlockAnnounce({:?})", block);
                            }
                        }
                    }
                    Err(err) => {
                        log_err(Err(err));
                    }
                }
            }));
            guards.push(scope.spawn(|| loop {
                debug!("Sending discovery ping...");
                log_err(self.p2p.send_discovery());
                sleep(self.discovery_ping_interval);
            }));
            guards.push(scope.spawn(|| loop {
                debug!("Asking peers for status updates...");
                for addr in self.peers.lock().unwrap().keys() {
                    send_queue.push((*addr, Message::StatusRequest));
                }
                sleep(self.discovery_ping_interval);
            }));

            for guard in guards {
                guard.join()?;
            }
            Ok(())
        })
    }
}
