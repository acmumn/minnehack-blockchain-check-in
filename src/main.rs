#[macro_use]
extern crate clap;
extern crate dotenv;
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate minnehack_check_in;
extern crate pretty_env_logger;
extern crate serde_cbor;

use std::process::exit;

use clap::ArgMatches;
use error_chain::ChainedError;
use minnehack_check_in::{Client, Result};

fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init().unwrap();

    let matches = clap_app!((crate_name!()) =>
        (about: crate_description!())
        (author: crate_authors!())
        (version: crate_version!())
    ).get_matches();

    info!("Starting up...");
    match run(matches) {
        Ok(()) => info!("Exiting peacefully..."),
        Err(err) => {
            error!("{}", err.display_chain());
            error!("Exiting with error...");
            exit(1);
        }
    }
}

fn run(_matches: ArgMatches) -> Result<()> {
    Client::new()?.run()
}

/*
fn response_thread(p2p: Arc<P2P>) {
    loop {
        match p2p.listen() {
            Ok((addr, msg)) => match msg {
                Message::Ping => {
                    info!("Got ping from {}", addr);
                    p2p.peers
                        .write()
                        .unwrap()
                        .entry(addr)
                        .or_insert(PeerState::Confirmed);
                    log_err(p2p.send_to(addr, &Message::Pong));
                }
                Message::Pong => {
                    info!("Got pong from {}", addr);
                    let mut peers = p2p.peers.write().unwrap();
                    match peers.entry(addr) {
                        Entry::Occupied(mut entry) => {
                            if entry.get() == &PeerState::Speculative {
                                entry.insert(PeerState::Confirmed);
                            }
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(PeerState::Confirmed);
                        }
                    }
                }
                Message::PeerListRequest => {
                    info!("Got request for list of peers from {}", addr);

                    let lock = p2p.peers.read().unwrap();
                    let known_peers = (&*lock)
                        .into_iter()
                        .filter(|&(_, &state)| state != PeerState::Speculative)
                        .map(|(&addr, _)| addr);

                    let mut peers = ArrayVec::new();
                    peers.extend(known_peers);
                    log_err(p2p.send_to(
                        addr,
                        &Message::PeerListResponse(peers),
                    ));
                }
                Message::PeerListResponse(peers) => {
                    info!("Got {} peers from {}", peers.len(), addr);
                    unimplemented!("{:#?}", peers)
                }
            },
            Err(err) => error!("{}", err.display_chain()),
        }
    }
}
*/
