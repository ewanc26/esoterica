use crate::archetypes::{self};
use crate::phonology::PhonologyEngine;
use crate::morphology::MorphologyEngine;
use crate::args::Args;
use ratatui::widgets::ListState;

pub struct App {
    pub phonology: String,
    pub morphology: String,
    pub syntax: String,
    pub output: String,
    pub fields: Vec<String>,
    pub list_state: ListState,
}

impl App {
    pub fn new(args: Args) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            phonology: args.phonology.first().cloned().unwrap_or_default(),
            morphology: args.morphology.first().cloned().unwrap_or_default(),
            syntax: args.syntax.unwrap_or_default(),
            output: String::new(),
            fields: vec!["Phonology".to_string(), "Morphology".to_string(), "Syntax".to_string()],
            list_state: state,
        }
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Char('q') => return false,
            KeyCode::Enter => self.generate(),
            KeyCode::Tab => {
                let i = match self.list_state.selected() {
                    Some(i) => (i + 1) % self.fields.len(),
                    None => 0,
                };
                self.list_state.select(Some(i));
            }
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
            _ => {}
        }
        true
    }

    pub fn handle_mouse(&mut self, _mouse: crossterm::event::MouseEvent) {
        // Implement mouse handling here if needed
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
