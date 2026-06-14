pub mod app;
pub mod ui;
pub mod event;

use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use crate::args::Args;
use crossterm::{execute, terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType}};

pub fn run_tui(args: Args) -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), Clear(ClearType::All))?;
    
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = app::App::new(args);
    let event_handler = event::EventHandler::new();

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        match event_handler.next()? {
            event::TuiEvent::Key(key) => {
                if !app.handle_key(key) {
                    break;
                }
            }
            event::TuiEvent::Mouse(mouse) => {
                app.handle_mouse(mouse);
            }
        }
    }
    disable_raw_mode()?;
    Ok(())
}
