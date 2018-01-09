use std::process::exit;

use crossbeam::sync::MsQueue;
use minnehack_check_in::{Client, Result};
use minnehack_check_in::util::log_err;
use tui::Terminal;
use tui::backend::RawBackend;
use tui::layout::{Direction, Group, Rect, Size};
use tui::widgets::{Block, Borders, Item, List, Widget};

use events::Event;

pub fn thread(
    client: &Client,
    event_queue: &MsQueue<Event>,
    terminal: &mut Terminal<RawBackend>,
) -> ! {
    let mut size = terminal.size().unwrap();
    loop {
        if log_err(render(&client, terminal, &mut size)) {
            continue;
        }
        terminal.draw().unwrap();
        match event_queue.pop() {
            Event::Quit => {
                terminal.show_cursor().unwrap();
                exit(0);
            }
            Event::Tick => continue,
        }
    }
}

fn render(
    client: &Client,
    terminal: &mut Terminal<RawBackend>,
    old_size: &mut Rect,
) -> Result<()> {
    let size = terminal.size()?;
    if size != *old_size {
        terminal.resize(size)?;
        *old_size = size;
    }

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
                    .map(|peer| format!("{} -- {}", peer.addr, peer.karma))
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
