use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use crossterm::event::{self, Event, KeyCode};
use std::io;

struct App {
    phonology: String,
    morphology: String,
    syntax: String,
    selected_index: usize,
}

impl App {
    fn new() -> Self {
        Self {
            phonology: String::new(),
            morphology: String::new(),
            syntax: String::new(),
            selected_index: 0,
        }
    }
}

pub fn run_tui() -> io::Result<()> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Length(3), Constraint::Percentage(50)])
                .split(f.size());

            let phono_text = format!("Phonology: {}", app.phonology);
            let phono_block = Block::default().title("Phonology").borders(Borders::ALL);
            let phono_para = Paragraph::new(phono_text).block(phono_block);
            f.render_widget(phono_para, chunks[0]);

            let morph_text = format!("Morphology: {}", app.morphology);
            let morph_block = Block::default().title("Morphology").borders(Borders::ALL);
            let morph_para = Paragraph::new(morph_text).block(morph_block);
            f.render_widget(morph_para, chunks[1]);

            let syntax_text = format!("Syntax: {}", app.syntax);
            let syntax_block = Block::default().title("Syntax").borders(Borders::ALL);
            let syntax_para = Paragraph::new(syntax_text).block(syntax_block);
            f.render_widget(syntax_para, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char(c) => {
                    match app.selected_index {
                        0 => app.phonology.push(c),
                        1 => app.morphology.push(c),
                        2 => app.syntax.push(c),
                        _ => {}
                    }
                }
                KeyCode::Backspace => {
                    match app.selected_index {
                        0 => { app.phonology.pop(); }
                        1 => { app.morphology.pop(); }
                        2 => { app.syntax.pop(); }
                        _ => {}
                    }
                }
                KeyCode::Tab => {
                    app.selected_index = (app.selected_index + 1) % 3;
                }
                KeyCode::Enter => {
                    // Trigger Generation
                    info!("Generating with: Phono: {}, Morph: {}, Syntax: {}", app.phonology, app.morphology, app.syntax);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

use tracing::info;
