extern crate arrayvec;
#[macro_use]
extern crate clap;
extern crate dotenv;
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate minnehack_check_in;
extern crate pretty_env_logger;
extern crate serde_cbor;
extern crate toml;

use std::io::{stdin, BufRead, BufReader, Write};
use std::process::exit;
use std::sync::Arc;

use arrayvec::ArrayVec;

use error_chain::ChainedError;
use minnehack_check_in::Client;
use minnehack_check_in::cards::{parse_card, CardParse};

fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init().unwrap();

    clap_app!((crate_name!()) =>
        (about: crate_description!())
        (author: crate_authors!())
        (version: crate_version!())
    ).get_matches();

    info!("Starting up...");
    let client = match Client::new() {
        Ok(val) => Arc::new(val),
        Err(err) => {
            error!("{}", err.display_chain());
            info!("Exiting with error...");
            exit(1);
        }
    };
    client.add_peer("160.94.179.148:10101".parse().unwrap());

    client.clone().run_with_one(move |_queue| {
        let mut stdin = BufReader::new(stdin());
        let mut line = String::new();
        loop {
            line.clear();
            stdin.read_line(&mut line).unwrap();

            match parse_card(&line) {
                CardParse::Card(fields) => {
                    let mut buf = ArrayVec::<[u8; 256]>::new();
                    let l = fields.len();
                    assert!(l < 256);
                    buf.push(l as u8);
                    for field in fields.iter() {
                        let l = field.len();
                        assert!(l < 256);
                        buf.push(l as u8);
                        buf.write_all(field.as_bytes()).unwrap();
                    }
                    client.mine(buf);
                }
                err => error!("Error reading card: {:?}", err),
            }
        }
    })
}
