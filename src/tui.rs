use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, List, ListItem, ListState},
    style::{Style, Color, Modifier},
    Terminal, Frame,
};
use crossterm::event::{self, Event, KeyCode};
use std::io;
use crate::archetypes::{self};
use crate::phonology::PhonologyEngine;
use crate::morphology::MorphologyEngine;

/// The Component trait defines the interface for UI sections.
trait Component {
    fn handle_event(&mut self, key_code: KeyCode) -> bool;
    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect);
}

struct ConfigComponent {
    phonology: String,
    morphology: String,
    syntax: String,
    list_state: ListState,
    fields: Vec<String>,
}

impl ConfigComponent {
    fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            phonology: String::new(),
            morphology: String::new(),
            syntax: String::new(),
            list_state: state,
            fields: vec!["Phonology".to_string(), "Morphology".to_string(), "Syntax".to_string()],
        }
    }
}

impl Component for ConfigComponent {
    fn handle_event(&mut self, key_code: KeyCode) -> bool {
        match key_code {
            KeyCode::Char(c) => {
                if let Some(i) = self.list_state.selected() {
                    match i {
                        0 => self.phonology.push(c),
                        1 => self.morphology.push(c),
                        2 => self.syntax.push(c),
                        _ => {}
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(i) = self.list_state.selected() {
                    match i {
                        0 => { self.phonology.pop(); }
                        1 => { self.morphology.pop(); }
                        2 => { self.syntax.pop(); }
                        _ => {}
                    }
                }
            }
            KeyCode::Tab => {
                let i = match self.list_state.selected() {
                    Some(i) => (i + 1) % self.fields.len(),
                    None => 0,
                };
                self.list_state.select(Some(i));
            }
            _ => return false,
        }
        true
    }

    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let items: Vec<ListItem> = self.fields.iter().enumerate().map(|(i, field)| {
            let content = match i {
                0 => format!("{}: {}", field, self.phonology),
                1 => format!("{}: {}", field, self.morphology),
                2 => format!("{}: {}", field, self.syntax),
                _ => field.clone(),
            };
            let style = if Some(i) == self.list_state.selected() {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(content).style(style)
        }).collect();

        let list = List::new(items)
            .block(Block::default().title("Configuration (Tab to select, Enter to generate)").borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        
        f.render_stateful_widget(list, area, &mut self.list_state.clone());
    }
}

struct OutputComponent {
    output: String,
}

impl Component for OutputComponent {
    fn handle_event(&mut self, _key_code: KeyCode) -> bool { false }

    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        f.render_widget(Paragraph::new(self.output.as_str()).block(Block::default().title("Output").borders(Borders::ALL)), area);
    }
}

pub fn run_tui() -> io::Result<()> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let mut config_comp = ConfigComponent::new();
    let mut output_comp = OutputComponent { output: String::new() };

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(f.size());

            config_comp.render(f, chunks[0]);
            output_comp.render(f, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Enter => {
                    let phono_reg = archetypes::get_phonology_registry();
                    let morph_reg = archetypes::get_morphology_registry();

                    if let (Some(ph), Some(mo)) = (phono_reg.get(&config_comp.phonology), morph_reg.get(&config_comp.morphology)) {
                        let ph_engine = PhonologyEngine::new(ph.clone());
                        let mo_engine = MorphologyEngine::new(mo.clone());
                        
                        let root = ph_engine.generate_word(2);
                        let word = mo_engine.apply_rules(&root);
                        output_comp.output = format!("Generated: {}", word);
                    } else {
                        output_comp.output = "Error: Invalid input".to_string();
                    }
                }
                code => {
                    config_comp.handle_event(code);
                }
            }
        }
    }
    Ok(())
}
