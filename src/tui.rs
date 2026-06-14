use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    style::{Style, Color, Modifier},
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
    fields: Vec<String>,
    selected_index: usize,
}

impl App {
    fn new() -> Self {
        Self {
            phonology: String::new(),
            morphology: String::new(),
            syntax: String::new(),
            output: String::new(),
            fields: vec!["Phonology".to_string(), "Morphology".to_string(), "Syntax".to_string()],
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
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(f.size());

            let items: Vec<ListItem> = app.fields.iter().enumerate().map(|(i, field)| {
                let content = match i {
                    0 => format!("{}: {}", field, app.phonology),
                    1 => format!("{}: {}", field, app.morphology),
                    2 => format!("{}: {}", field, app.syntax),
                    _ => field.clone(),
                };
                let style = if i == app.selected_index {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(content).style(style)
            }).collect();

            let list = List::new(items).block(Block::default().title("Configuration (Tab to select, Enter to generate)").borders(Borders::ALL));
            f.render_widget(list, chunks[0]);
            
            f.render_widget(Paragraph::new(app.output.as_str()).block(Block::default().title("Output").borders(Borders::ALL)), chunks[1]);
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
                    app.selected_index = (app.selected_index + 1) % app.fields.len();
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
