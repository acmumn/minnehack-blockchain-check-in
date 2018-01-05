#[macro_use]
extern crate clap;
extern crate dotenv;
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate minnehack_check_in;
extern crate pretty_env_logger;

use std::process::exit;

use clap::ArgMatches;
use error_chain::ChainedError;
use minnehack_check_in::Result;

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

fn run(matches: ArgMatches) -> Result<()> {
    minnehack_check_in::init()?;
    Ok(())
}
