//! Ratatui-based interactive TUI entry point and module declarations.

pub mod app;
pub mod ui;
pub mod event;
pub mod components;
pub mod phonology_designer;

use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use crate::args::Args;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
};

/// Launch the interactive TUI.
/// Switches to alternate screen, runs the event loop, restores terminal on exit.
pub fn run_tui(args: Args) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Clear(ClearType::All))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = app::App::new(args);
    let event_handler = event::EventHandler::new();

    // ── Event Loop ──────────────────────────────────────────────────────────

    loop {
        terminal.draw(|f| ui::render(f, &app))?;
        match event_handler.next()? {
            event::TuiEvent::Key(key) => { if !app.handle_key(key) { break; } }
            event::TuiEvent::Mouse(mouse) => { app.handle_mouse(mouse); }
        }
    }

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
