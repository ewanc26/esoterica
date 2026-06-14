use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    style::{Style, Color},
    Terminal,
};
use crossterm::event::{self, Event, KeyCode};
use std::io;
use crate::archetypes::{self};
use crate::phonology::PhonologyEngine;
use crate::morphology::MorphologyEngine;

struct App {
    phonology: String,
    morphology: String,
    syntax: String,
    output: String,
    selected_index: usize,
}

impl App {
    fn new() -> Self {
        Self {
            phonology: String::new(),
            morphology: String::new(),
            syntax: String::new(),
            output: String::new(),
            selected_index: 0,
        }
    }

    fn generate(&mut self) {
        let phono_reg = archetypes::get_phonology_registry();
        let morph_reg = archetypes::get_morphology_registry();

        if let (Some(ph), Some(mo)) = (phono_reg.get(&self.phonology), morph_reg.get(&self.morphology)) {
            let ph_engine = PhonologyEngine::new(ph.clone());
            let mo_engine = MorphologyEngine::new(mo.clone());
            
            let root = ph_engine.generate_word(2);
            let word = mo_engine.apply_rules(&root);
            self.output = format!("Generated: {}", word);
        } else {
            self.output = "Error: Invalid input".to_string();
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

            let highlight = Style::default().fg(Color::Yellow);

            let phono_block = Block::default().title("Phonology").borders(Borders::ALL).style(if app.selected_index == 0 { highlight } else { Style::default() });
            f.render_widget(Paragraph::new(app.phonology.as_str()).block(phono_block), chunks[0]);
            
            let morph_block = Block::default().title("Morphology").borders(Borders::ALL).style(if app.selected_index == 1 { highlight } else { Style::default() });
            f.render_widget(Paragraph::new(app.morphology.as_str()).block(morph_block), chunks[1]);
            
            let syntax_block = Block::default().title("Syntax").borders(Borders::ALL).style(if app.selected_index == 2 { highlight } else { Style::default() });
            f.render_widget(Paragraph::new(app.syntax.as_str()).block(syntax_block), chunks[2]);
            
            f.render_widget(Paragraph::new(app.output.as_str()).block(Block::default().title("Output").borders(Borders::ALL)), chunks[3]);
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
                    app.generate();
                }
                _ => {}
            }
        }
    }
    Ok(())
}
