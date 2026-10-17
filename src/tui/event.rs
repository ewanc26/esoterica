//! Terminal event polling. Wraps crossterm events into a TuiEvent enum so the
//! application layer does not depend on crossterm's event model directly.

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use std::time::Duration;

pub enum TuiEvent {
    /// A key press (key releases and repeats are ignored).
    Key(KeyEvent),
    /// The terminal was resized and should be redrawn.
    Resize,
}

/// Event polling loop. Blocks until a key press or resize arrives.
pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self { Self }

    pub fn next(&self) -> std::io::Result<TuiEvent> {
        loop {
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => return Ok(TuiEvent::Key(key)),
                    Event::Resize(_, _) => return Ok(TuiEvent::Resize),
                    _ => continue,
                }
            }
        }
    }
}

impl Default for EventHandler { fn default() -> Self { Self::new() } }
