extern crate ctrlc;
extern crate minnehack_check_in;
extern crate tui;

use std::process::exit;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use minnehack_check_in::{Client, Result};
use tui::Terminal;
use tui::backend::{Backend, RawBackend};
use tui::layout::{Direction, Group, Size};
use tui::widgets::{Block, Borders, SelectableList, Widget};

fn main() {
    ctrlc::set_handler(|| exit(130)).unwrap();
    run().unwrap();
}

fn run() -> Result<()> {
    let mut backend = RawBackend::new()?;
    backend.clear()?;
    let mut terminal = Terminal::new(backend)?;

    let client = Arc::new(Client::new()?);
    client.clone().run_with_one(move |_| loop {
        render(&client, &mut terminal).unwrap();
        terminal.draw().unwrap();
        sleep(Duration::from_secs(1));
    });

    Ok(())
}

fn render(client: &Client, terminal: &mut Terminal<RawBackend>) -> Result<()> {
    let hashes = client.with_chain(|chain| {
        chain
            .into_iter()
            .map(|block| block.hash.to_string())
            .collect::<Vec<String>>()
    });
    let peers = client.with_peers(|peers| {
        peers
            .values()
            .map(|peer| peer.addr.to_string())
            .collect::<Vec<String>>()
    });

    let size = terminal.size()?;
    Group::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .sizes(&[Size::Min(10), Size::Fixed(70)])
        .render(terminal, &size, |terminal, chunks| {
            // TODO Peer List
            SelectableList::default()
                .block(Block::default().title("Peers").borders(Borders::ALL))
                .items(&peers)
                .render(terminal, &chunks[0]);
            SelectableList::default()
                .block(Block::default().title("Blocks").borders(Borders::ALL))
                .items(&hashes)
                .render(terminal, &chunks[1]);
        });
    Ok(())
}
