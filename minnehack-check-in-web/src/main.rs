#[macro_use]
extern crate iron;
extern crate logger;
extern crate minnehack_check_in;
extern crate mount;
extern crate persistent;
extern crate pretty_env_logger;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod config;
mod handlers;
mod util;

use std::sync::Arc;

use iron::{Chain, Iron};
use logger::Logger;
use minnehack_check_in::{Client, Result, ResultExt};
use minnehack_check_in::util::load_toml_or_default;
use mount::Mount;
use persistent::Read;

use config::Config;

fn main() {
    run().unwrap();
}

fn run() -> Result<()> {
    pretty_env_logger::init().unwrap();

    let config: Config = load_toml_or_default("minnehack-check-in-web.toml");
    let client = Arc::new(Client::new_from_config(config.peer.clone())?);

    let mut mount = Mount::new();
    mount.mount("/api/mine", handlers::mine);
    mount.mount("/api/status", handlers::status);

    let mut chain = Chain::new(mount);
    chain.link(Logger::new(None));
    chain.link_before(Read::<util::Client>::one(client.clone()));

    Ok(client.run_with(move |_, _| {
        Iron::new(chain)
            .http((config.addr, config.port))
            .chain_err(|| "Couldn't start HTTP server")
            .unwrap();
    }))
}
