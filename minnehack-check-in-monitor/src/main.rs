extern crate crossbeam;
extern crate minnehack_check_in;
extern crate termion;
extern crate tui;

mod events;
mod render;

use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use crossbeam::sync::MsQueue;
use minnehack_check_in::{Client, Config, Result, ResultExt};
use tui::Terminal;
use tui::backend::RawBackend;

fn main() {
    run().unwrap();
}

fn run() -> Result<()> {
    let mut terminal =
        Terminal::new(RawBackend::new()
            .chain_err(|| "Couldn't open the terminal backend")?)
            .chain_err(
            || "Couldn't open the terminal",
        )?;
    terminal.clear().chain_err(|| "Is stdin closed?")?;

    let client = Arc::new(Client::new_from_config(
        Config::load_from("minnehack-check-in.toml").unwrap_or_default(),
    )?);

    let event_queue = Arc::new(MsQueue::new());
    client.clone().run_with(move |scope, _| {
        let input_event_queue = event_queue.clone();
        scope.spawn(move || events::thread(&input_event_queue));

        let timer_event_queue = event_queue.clone();
        scope.spawn(move || loop {
            timer_event_queue.push(events::Event::Tick);
            sleep(Duration::from_secs(1));
        });

        scope.spawn(move || {
            render::thread(&client, &event_queue, &mut terminal)
        });
    });

    Ok(())
}
