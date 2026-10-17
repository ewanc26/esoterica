//! Application state and key handling for the Ratatui-based TUI.
//! Routes key events between the config selector, phonology designer, and help overlay.

use crate::pipeline::Registries;
use crate::lexicon::LexiconGenerator;
use crate::args::Args;
use crate::tui::components::{ConfigComponent, Component, HelpComponent};
use crate::tui::phonology_designer::PhonologyDesigner;
use crate::phonology::PhonologyEngine;
use crate::morphology::MorphologyEngine;
use crate::syntax::SyntaxEngine;

/// Top-level TUI application state.
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

    /// Route a key event to the active subsystem.
    /// Returns false when the user requests quit.
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;

        // ── Designer takes priority when active ──────────────────────────────

        if self.designer.active {
            // Space/Enter toggle the phoneme at the cursor
            if key.code == KeyCode::Char(' ') || key.code == KeyCode::Enter {
                self.designer.toggle_phoneme_mut();
                return true;
            }
            return self.designer.handle_key(key.code);
        }

        // ── Main view keybindings ────────────────────────────────────────────

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

    // ── Generation ──────────────────────────────────────────────────────────

    /// Run the generation pipeline and populate the output display.
    fn generate(&mut self) {
        let (ph, mo, sy, sc_keys) = self.config.get_selected_values();
        let mut spec = match Registries::builtin().spec(&ph, &mo, &sy, &sc_keys) {
            Ok(spec) => spec,
            Err(e) => { self.output = format!("Error: {}", e); return; }
        };

        // Use custom phonology if the designer has one confirmed
        let custom_label = if let Some(ref custom) = self.designer.confirmed_phonology {
            spec.phonology = custom.clone();
            " (custom)"
        } else {
            ""
        };

        let ph_engine = PhonologyEngine::new(spec.phonology.clone());
        let mo_engine = MorphologyEngine::new(spec.morphology.clone());

        let root1 = ph_engine.generate_word(2);
        let root3 = ph_engine.generate_word(2);
        let (word1, _) = mo_engine.apply_rules(&root1);
        let word2 = ph_engine.generate_word(1);
        let (word3, _) = mo_engine.apply_rules(&root3);

        // ── Sentence Generation ──────────────────────────────────────────────

        let sentence = SyntaxEngine::new(spec.syntax.clone())
            .generate_sentence(&[word1.clone(), word2.clone(), word3.clone()]);
        let sentence_info = format!("\nSentence ({}): {}", sy.to_uppercase(), sentence);

        self.output = format!(
            "Phonology: {}{} | Morphology: {} | Syntax: {} | Sound Changes: {:?}\nWords: {}, {}, {}{}",
            ph, custom_label, mo, sy, sc_keys, word1, word2, word3, sentence_info
        );

        let mut gen = LexiconGenerator::new(spec.phonology, spec.morphology, &spec.sound_changes);
        gen.generate_core_lexicon(100);
        self.generator = Some(gen);
    }

    // ── Save ────────────────────────────────────────────────────────────────

    /// Save the currently generated lexicon to disk.
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
