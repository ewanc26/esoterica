//! Ratatui-based interactive TUI entry point and module declarations.

pub mod app;
pub mod ui;
pub mod event;
pub mod components;
pub mod phonology_designer;

use crate::args::Args;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

/// Restores the terminal when dropped, including on error and panic unwinds.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

/// Launch the interactive TUI. The terminal is restored when this returns, whether
/// the user quit or an error occurred.
pub fn run_tui(args: Args) -> io::Result<()> {
    enable_raw_mode()?;
    let _guard = TerminalGuard;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal: Terminal<CrosstermBackend<Stdout>> = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut app = app::App::new(args);
    let events = event::EventHandler::new();
    loop {
        terminal.draw(|f| ui::render(f, &app))?;
        match events.next()? {
            event::TuiEvent::Key(key) => {
                if !app.handle_key(key) { break; }
            }
            event::TuiEvent::Resize => {}
        }
    }
    Ok(())
}
