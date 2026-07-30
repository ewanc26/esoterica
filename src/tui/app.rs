//! Application state and key handling for the Ratatui-based TUI.
//! Routes key events between the config selector, phonology designer, and help overlay.

use crate::archetypes::{self};
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
    /// CLI arguments the TUI honours: seed, lexicon size, syllable count, output path.
    args: Args,
}

impl App {
    pub fn new(args: Args) -> Self {
        Self {
            config: ConfigComponent::new(),
            output: String::new(),
            generator: None,
            help: HelpComponent,
            show_help: false,
            designer: PhonologyDesigner::new(),
            args,
        }
    }

    /// Where `s` writes the lexicon, honouring `--output`.
    fn output_path(&self) -> String {
        self.args
            .output
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "lexicon_output.json".to_string())
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

        // A seed makes the previewed words and the saved lexicon reproducible.
        let (ph_engine, mo_engine) = match self.args.seed {
            Some(seed) => (
                PhonologyEngine::seeded(ph_cfg.clone(), seed),
                MorphologyEngine::seeded(mo_cfg.clone(), seed.wrapping_add(0x9E37_79B9)),
            ),
            None => (
                PhonologyEngine::new(ph_cfg.clone()),
                MorphologyEngine::new(mo_cfg.clone()),
            ),
        };

        let root1 = ph_engine.generate_word(2);
        let root3 = ph_engine.generate_word(2);
        let (word1, _) = mo_engine.apply_rules(&root1);
        let word2 = ph_engine.generate_word(1);
        let (word3, _) = mo_engine.apply_rules(&root3);

        // ── Sentence Generation ──────────────────────────────────────────────

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

        let mut generator = match LexiconGenerator::try_new(
            ph_cfg,
            mo_cfg,
            sound_changes,
            self.args.seed,
        ) {
            Ok(generator) => generator.with_syllables(self.args.syllables.unwrap_or(2)),
            Err(e) => {
                self.output = format!("Error: {}", e);
                return;
            }
        };
        let size = self.args.lexicon_size.unwrap_or(100);
        let produced = generator.generate_core_lexicon(size).len();
        if produced < size {
            self.output.push_str(&format!(
                "\nOnly {} of {} entries: this phonology has too few distinct forms.",
                produced, size
            ));
        }
        self.generator = Some(generator);
    }

    // ── Save ────────────────────────────────────────────────────────────────

    /// Save the currently generated lexicon to disk.
    fn save_lexicon(&mut self) {
        if let Some(generator) = &self.generator {
            let path = self.output_path();
            match generator.save_to_file(&path) {
                Ok(_) => self.output = format!("{}\n\nLexicon saved to {}", self.output, path),
                Err(e) => self.output = format!("Error saving: {}", e),
            }
        } else {
            self.output = "Error: Generate a lexicon first!".to_string();
        }
    }
}
