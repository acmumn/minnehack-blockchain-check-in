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

use error_chain::ChainedError;
use minnehack_check_in::Client;

fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init().unwrap();

    clap_app!((crate_name!()) =>
        (about: crate_description!())
        (author: crate_authors!())
        (version: crate_version!())
    ).get_matches();

    info!("Starting up...");
    match Client::new().and_then(|client| client.run()) {
        Ok(()) => info!("Exiting peacefully..."),
        Err(err) => {
            error!("{}", err.display_chain());
            info!("Exiting with error...");
            exit(1);
        }
    }
}
