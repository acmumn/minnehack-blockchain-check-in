extern crate ctrlc;
extern crate minnehack_check_in;
extern crate tui;

use std::process::exit;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use minnehack_check_in::{Client, Result};
use tui::Terminal;
use tui::backend::RawBackend;
use tui::layout::{Direction, Group, Size};
use tui::widgets::{Block, Borders, Item, List, Widget};

fn main() {
    ctrlc::set_handler(|| exit(130)).unwrap();
    run().unwrap();
}

fn run() -> Result<()> {
    let mut terminal = Terminal::new(RawBackend::new()?)?;
    terminal.clear()?;

    let client = Arc::new(Client::new()?);
    client.clone().run_with_one(move |_| loop {
        render(&client, &mut terminal).unwrap();
        terminal.draw().unwrap();
        sleep(Duration::from_secs(1));
    });

    Ok(())
}

fn render(client: &Client, terminal: &mut Terminal<RawBackend>) -> Result<()> {
    let size = terminal.size()?;
    Group::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .sizes(&[Size::Min(10), Size::Fixed(70)])
        .render(terminal, &size, |terminal, chunks| {
            let hashes = client.with_chain(|chain| {
                chain
                    .into_iter()
                    .map(|block| block.hash.to_string())
                    .map(Item::Data)
                    .collect::<Vec<_>>()
            });
            let peers = client.with_peers(|peers| {
                peers
                    .values()
                    .map(|peer| peer.addr.to_string())
                    .map(Item::Data)
                    .collect::<Vec<_>>()
            });

            List::new(peers.into_iter())
                .block(Block::default().title("Peers").borders(Borders::ALL))
                .render(terminal, &chunks[0]);
            List::new(hashes.into_iter())
                .block(Block::default().title("Blocks").borders(Borders::ALL))
                .render(terminal, &chunks[1]);
        });
    Ok(())
}
