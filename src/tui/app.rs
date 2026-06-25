use crate::archetypes::{self};
use crate::lexicon::LexiconGenerator;
use crate::args::Args;
use crate::tui::components::{ConfigComponent, Component, HelpComponent};
use crate::tui::phonology_designer::PhonologyDesigner;
use crate::phonology::PhonologyEngine;
use crate::morphology::MorphologyEngine;
use crate::syntax::SyntaxEngine;

pub struct App {
    pub config: ConfigComponent,
    pub output: String,
    pub generator: Option<LexiconGenerator>,
    pub help: HelpComponent,
    pub show_help: bool,
    pub designer: PhonologyDesigner,
}

impl App {
    pub fn new(_args: Args) -> Self {
        Self {
            config: ConfigComponent::new(),
            output: String::new(),
            generator: None,
            help: HelpComponent,
            show_help: false,
            designer: PhonologyDesigner::new(),
        }
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;

        // If designer is active, route keys there
        if self.designer.active {
            // Handle toggle before calling designer
            if key.code == KeyCode::Char(' ') || key.code == KeyCode::Enter {
                self.designer.toggle_phoneme_mut();
                return true;
            }
            return self.designer.handle_key(key.code);
        }

        match key.code {
            KeyCode::Char('q') => return false,
            KeyCode::Char('h') => self.show_help = !self.show_help,
            KeyCode::Char('p') => {
                self.designer.toggle();
            }
            KeyCode::Enter => {
                if !self.show_help { self.generate(); }
            }
            KeyCode::Char('s') => {
                if !self.show_help { self.save_lexicon(); }
            }
            code => {
                if !self.show_help {
                    self.config.handle_event(code);
                }
            }
        }
        true
    }

    pub fn handle_mouse(&mut self, _mouse: crossterm::event::MouseEvent) {}

    fn generate(&mut self) {
        let (ph, mo, sy, sc_keys) = self.config.get_selected_values();
        let phono_reg = archetypes::get_phonology_registry();
        let morph_reg = archetypes::get_morphology_registry();
        let syntax_reg = archetypes::get_syntax_registry();
        let sc_reg = archetypes::get_sound_change_registry();

        // Use custom phonology if the designer has one confirmed
        let ph_cfg = if let Some(ref custom) = self.designer.confirmed_phonology {
            custom.clone()
        } else {
            match phono_reg.get(&ph) {
                Some(cfg) => cfg.clone(),
                None => { self.output = format!("Error: Unknown phonology '{}'", ph); return; }
            }
        };

        let mo_cfg = match morph_reg.get(&mo) {
            Some(cfg) => cfg.clone(),
            None => { self.output = format!("Error: Unknown morphology '{}'", mo); return; }
        };

        let mut sound_changes = Vec::new();
        for key in &sc_keys {
            if let Some(rules) = sc_reg.get(key) {
                sound_changes.extend(rules.clone());
            }
        }

        let ph_engine = PhonologyEngine::new(ph_cfg.clone());
        let mo_engine = MorphologyEngine::new(mo_cfg.clone());

        let root1 = ph_engine.generate_word(2);
        let root3 = ph_engine.generate_word(2);
        let (word1, _) = mo_engine.apply_rules(&root1);
        let word2 = ph_engine.generate_word(1);
        let (word3, _) = mo_engine.apply_rules(&root3);

        let mut sentence_info = String::new();
        if let Some(syntax_cfg) = syntax_reg.get(&sy) {
            let syntax_engine = SyntaxEngine::new(syntax_cfg.clone());
            let sentence = syntax_engine.generate_sentence(&[
                word1.clone(), word2.clone(), word3.clone()
            ]);
            sentence_info = format!("\nSentence ({}): {}", sy.to_uppercase(), sentence);
        }

        let custom_label = if self.designer.confirmed_phonology.is_some() { " (custom)" } else { "" };
        self.output = format!(
            "Phonology: {}{} | Morphology: {} | Syntax: {} | Sound Changes: {:?}\nWords: {}, {}, {}{}",
            ph, custom_label, mo, sy, sc_keys, word1, word2, word3, sentence_info
        );

        let mut gen = LexiconGenerator::new(ph_cfg, mo_cfg, sound_changes);
        gen.generate_core_lexicon(100);
        self.generator = Some(gen);
    }

    fn save_lexicon(&mut self) {
        if let Some(gen) = &self.generator {
            match gen.save_to_file("lexicon_output.json") {
                Ok(_) => self.output = format!("{}\n\nLexicon saved to lexicon_output.json", self.output),
                Err(e) => self.output = format!("Error saving: {}", e),
            }
        } else {
            self.output = "Error: Generate a lexicon first!".to_string();
        }
    }
}
