pub mod app;
pub mod ui;
pub mod event;

use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use crate::args::Args;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
};

pub fn run_tui(args: Args) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Clear(ClearType::All))?;
    
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
    
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
