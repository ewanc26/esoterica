use crate::archetypes::{self};
use crate::phonology::PhonologyEngine;
use crate::morphology::MorphologyEngine;
use crate::args::Args;
use crate::tui::components::{ConfigComponent, Component};

pub struct App {
    pub config: ConfigComponent,
    pub output: String,
}

impl App {
    pub fn new(_args: Args) -> Self {
        Self {
            config: ConfigComponent::new(),
            output: String::new(),
        }
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Char('q') => return false,
            KeyCode::Enter => self.generate(),
            _ => {
                self.config.handle_event(key.code);
            }
        }
        true
    }

    pub fn handle_mouse(&mut self, _mouse: crossterm::event::MouseEvent) {}

    fn generate(&mut self) {
        let (ph, mo, _sy) = self.config.get_selected_values();
        let phono_reg = archetypes::get_phonology_registry();
        let morph_reg = archetypes::get_morphology_registry();

        if let (Some(ph_cfg), Some(mo_cfg)) = (phono_reg.get(&ph), morph_reg.get(&mo)) {
            let ph_engine = PhonologyEngine::new(ph_cfg.clone());
            let mo_engine = MorphologyEngine::new(mo_cfg.clone());
            
            let root = ph_engine.generate_word(2);
            let word = mo_engine.apply_rules(&root);
            self.output = format!("{} {}: Generated {}", ph, mo, word);
        } else {
            self.output = "Error: Invalid selection".to_string();
        }
    }
}
