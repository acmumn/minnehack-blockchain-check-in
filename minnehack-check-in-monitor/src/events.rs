use std::io::stdin;

use crossbeam::sync::MsQueue;
use termion::event::{Event as TermEvent, Key};
use termion::input::TermRead;

pub enum Event {
    Quit,
    Tick,
}

pub fn thread(queue: &MsQueue<Event>) -> ! {
    loop {
        for event in stdin().events() {
            match event.unwrap() {
                TermEvent::Key(Key::Ctrl('c'))
                | TermEvent::Key(Key::Char('q')) => queue.push(Event::Quit),
                e => println!("{:?}", e),
            }
        }
    }
}
