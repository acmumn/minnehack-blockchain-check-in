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
use std::time::Duration;
use std::thread::sleep;

use arrayvec::ArrayVec;
use crossbeam::scope;
use crossbeam::sync::MsQueue;

use blockchain::Chain;
use cards::{parse_card, CardParse};
pub use errors::{Error, ErrorKind, Result, ResultExt};
use p2p::{P2P, Peer};
use util::log_err;

/// A blockchain client, using the `p2p` module for sending blocks.
#[derive(Debug)]
pub struct Client {
    /// The time to wait before sending discovery pings.
    pub discovery_ping_interval: Duration,

    /// The maximum karma value a peer can reach before it is ignored.
    pub max_karma: usize,

    chain: Chain,
    p2p: P2P,
    peers: HashMap<SocketAddr, Peer>,
}

impl Client {
    /// Creates a new `Client` with the default options.
    pub fn new() -> Result<Client> {
        Client::new_with_opts(10101, Chain::new(), Duration::from_secs(60), 10)
    }

    /// Creates a new `Client`.
    pub fn new_with_opts(
        port: u16,
        chain: Chain,
        discovery_ping_interval: Duration,
        max_karma: usize,
    ) -> Result<Client> {
        let p2p = P2P::with_port(port)?;
        Ok(Client {
            chain,
            discovery_ping_interval,
            max_karma,
            p2p,
            peers: HashMap::new(),
        })
    }

    /// Runs the `Client`.
    pub fn run(&self) -> Result<()> {
        let send_queue = MsQueue::new();
        scope(|scope| {
            let mut guards = ArrayVec::<[_; 4]>::new();

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
                        info!("Got {:?} from {}", msg, addr);
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

            for guard in guards {
                guard.join()?;
            }
            Ok(())
        })
    }
}
