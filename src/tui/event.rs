//! Crossterm event polling loop. Wraps raw terminal events into a TuiEvent enum
//! so the application layer doesn't depend directly on crossterm's event model.

use crossterm::event::{self, Event, KeyEvent, MouseEvent};
use std::time::Duration;

pub enum TuiEvent { Key(KeyEvent), Mouse(MouseEvent) }

/// Event polling loop. Blocks until a key or mouse event arrives (100ms poll interval).
pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self { Self }
    /// Block until a relevant terminal event arrives. Ignores resize events.
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

impl Default for EventHandler { fn default() -> Self { Self::new() } }
