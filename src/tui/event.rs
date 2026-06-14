use crossterm::event::{self, Event, KeyEvent, MouseEvent};
use std::time::Duration;

pub enum TuiEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn next(&self) -> std::io::Result<TuiEvent> {
        loop {
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => return Ok(TuiEvent::Key(key)),
                    Event::Mouse(mouse) => return Ok(TuiEvent::Mouse(mouse)),
                    _ => continue,
                }
            }
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
