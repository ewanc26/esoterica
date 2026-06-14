use crate::archetypes::{self};
use crate::lexicon::LexiconGenerator;
use crate::args::Args;
use crate::tui::components::{ConfigComponent, Component};
use crate::phonology::PhonologyEngine;
use crate::morphology::MorphologyEngine;

pub struct App {
    pub config: ConfigComponent,
    pub output: String,
    pub generator: Option<LexiconGenerator>,
}

impl App {
    pub fn new(_args: Args) -> Self {
        Self {
            config: ConfigComponent::new(),
            output: String::new(),
            generator: None,
        }
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Char('q') => return false,
            KeyCode::Enter => self.generate(),
            KeyCode::Char('s') => self.save_lexicon(),
            code => {
                self.config.handle_event(code);
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
            self.output = format!("Generated: {}", word);
            
            let mut gen = LexiconGenerator::new(ph_cfg.clone(), mo_cfg.clone(), Vec::new());
            gen.generate_core_lexicon(100);
            self.generator = Some(gen);
        } else {
            self.output = "Error: Invalid selection".to_string();
        }
    }

    fn save_lexicon(&mut self) {
        if let Some(gen) = &self.generator {
            match gen.save_to_file("lexicon_output.json") {
                Ok(_) => self.output = "Lexicon saved to lexicon_output.json".to_string(),
                Err(e) => self.output = format!("Error saving: {}", e),
            }
        } else {
            self.output = "Error: Generate a lexicon first!".to_string();
        }
    }
}
