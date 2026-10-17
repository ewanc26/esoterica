//! Application state and key handling for the Ratatui-based TUI.
//! Routes key events between the config selector, phonology designer, lexicon
//! browser and help overlay. Generation goes through the shared pipeline.

use crate::args::Args;
use crate::pipeline::{Generated, Registries};
use crate::syntax::SyntaxEngine;
use crate::tui::components::{ConfigComponent, Component, HelpComponent, CUSTOM_PHONOLOGY};
use crate::tui::phonology_designer::PhonologyDesigner;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Directory that saved lexicons are written to, relative to the working directory.
pub const DEFAULT_OUTPUT_DIR: &str = "esoterica_out";

/// Number of headwords in the lexicon browser's page-up and page-down step.
const PAGE: usize = 10;

/// Top-level TUI application state.
pub struct App {
    pub config: ConfigComponent,
    /// Summary of the last generation (configuration, size, example sentence).
    pub summary: String,
    /// Result of the last action (errors, save paths, hints).
    pub status: String,
    pub generated: Option<Generated>,
    pub help: HelpComponent,
    pub show_help: bool,
    pub show_lexicon: bool,
    pub lexicon_scroll: usize,
    pub designer: PhonologyDesigner,
    registries: Registries,
    output_dir: PathBuf,
}

impl App {
    /// Build the app from command-line arguments. A `--data-dir` that fails to load is
    /// reported in the status line, and the built-in catalogues are used instead.
    pub fn new(args: Args) -> Self {
        let (registries, startup) = match &args.data_dir {
            Some(dir) => match Registries::builtin().with_user_dir(dir) {
                Ok(regs) => (regs, format!("Loaded user data from {}", dir.display())),
                Err(e) => (Registries::builtin(), format!("Could not load {}; using built-in data.\n{}", dir.display(), e)),
            },
            None => (Registries::builtin(), "Press Enter to generate a lexicon. h for help.".to_string()),
        };
        Self::with_registries(registries, PathBuf::from(DEFAULT_OUTPUT_DIR), startup)
    }

    pub fn with_registries(registries: Registries, output_dir: PathBuf, startup: String) -> Self {
        Self {
            config: ConfigComponent::new(&registries),
            summary: String::new(),
            status: startup,
            generated: None,
            help: HelpComponent,
            show_help: false,
            show_lexicon: false,
            lexicon_scroll: 0,
            designer: PhonologyDesigner::new(),
            registries,
            output_dir,
        }
    }

    /// Route a key event to the active view. Returns false when the user quits.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return false;
        }

        if self.designer.active {
            self.designer.handle_key(key.code);
            if self.designer.take_confirmed() {
                self.config.select_custom();
                self.status = "Custom phonology selected. Press Enter to generate.".to_string();
            }
            return true;
        }

        if self.show_help {
            match key.code {
                KeyCode::Char('q') => return false,
                KeyCode::Char('h') | KeyCode::Esc => self.show_help = false,
                _ => {}
            }
            return true;
        }

        if self.show_lexicon {
            self.handle_lexicon_key(key.code);
            return !matches!(key.code, KeyCode::Char('q'));
        }

        match key.code {
            KeyCode::Char('q') => return false,
            KeyCode::Char('h') => self.show_help = true,
            KeyCode::Char('p') => self.designer.toggle(),
            KeyCode::Char('l') => self.open_lexicon(),
            KeyCode::Char('s') => self.save_lexicon(),
            KeyCode::Enter => self.generate(),
            other => { self.config.handle_event(other); }
        }
        true
    }

    fn handle_lexicon_key(&mut self, code: KeyCode) {
        let last = self.lexicon_len().saturating_sub(1);
        match code {
            KeyCode::Char('l') | KeyCode::Esc => self.show_lexicon = false,
            KeyCode::Up => self.lexicon_scroll = self.lexicon_scroll.saturating_sub(1),
            KeyCode::Down => self.lexicon_scroll = (self.lexicon_scroll + 1).min(last),
            KeyCode::PageUp => self.lexicon_scroll = self.lexicon_scroll.saturating_sub(PAGE),
            KeyCode::PageDown => self.lexicon_scroll = (self.lexicon_scroll + PAGE).min(last),
            KeyCode::Home => self.lexicon_scroll = 0,
            KeyCode::End => self.lexicon_scroll = last,
            _ => {}
        }
    }

    // ── Lexicon ─────────────────────────────────────────────────────────────

    fn lexicon_len(&self) -> usize {
        self.generated.as_ref().map_or(0, |g| g.lexicon.0.len())
    }

    /// Lexicon entries in headword order, one display line each.
    pub fn lexicon_lines(&self) -> Vec<String> {
        let Some(g) = &self.generated else { return Vec::new() };
        let mut headwords: Vec<&String> = g.lexicon.0.keys().collect();
        headwords.sort();
        headwords.into_iter().map(|hw| {
            let entry = &g.lexicon.0[hw];
            let definition = entry.senses.first().map(|s| s.definition.as_str()).unwrap_or("");
            format!("{:<18} {:<22} {:<10} {}", entry.headword, entry.ipa, entry.part_of_speech, definition)
        }).collect()
    }

    fn open_lexicon(&mut self) {
        if self.generated.is_none() {
            self.status = "Generate a lexicon first (Enter).".to_string();
            return;
        }
        self.show_lexicon = true;
        self.lexicon_scroll = 0;
    }

    // ── Generation ──────────────────────────────────────────────────────────

    /// Generate a lexicon from the current selections. Invalid configurations are
    /// reported in the status line rather than generated.
    pub fn generate(&mut self) {
        let sel = self.config.selection();
        let spec = if sel.phonology == CUSTOM_PHONOLOGY {
            self.registries.spec_with_phonology(self.designer.to_phonology(), &sel.morphology, &sel.syntax, &sel.sound_changes)
        } else {
            self.registries.spec(&sel.phonology, &sel.morphology, &sel.syntax, &sel.sound_changes)
        };
        let spec = match spec {
            Ok(spec) => spec,
            Err(e) => { self.status = format!("Error: {}", e); return; }
        };
        let generated = match spec.generate() {
            Ok(g) => g,
            Err(e) => {
                self.generated = None;
                self.summary.clear();
                self.status = format!("Cannot generate with this configuration: {}", e);
                return;
            }
        };

        let mut headwords: Vec<&String> = generated.lexicon.0.keys().collect();
        headwords.sort();
        let three: Vec<String> = headwords.iter().take(3).map(|s| s.to_string()).collect();
        let sentence = if three.len() == 3 {
            SyntaxEngine::new(spec.syntax.clone()).generate_sentence(&three)
        } else {
            String::from("(not enough words for a sentence)")
        };

        let phonology_label = if sel.phonology == CUSTOM_PHONOLOGY { "custom designer".to_string() } else { sel.phonology.clone() };
        let sound_changes = if sel.sound_changes.is_empty() { "none".to_string() } else { sel.sound_changes.join(", ") };
        self.summary = format!(
            "Phonology: {} | Morphology: {} | Syntax: {} | Sound changes: {}\nLexicon: {} entries. Example: {}",
            phonology_label, sel.morphology, sel.syntax, sound_changes, generated.lexicon.0.len(), sentence,
        );
        self.generated = Some(generated);
        self.lexicon_scroll = 0;
        self.status = "Generated. l to browse, s to save.".to_string();
    }

    // ── Save ────────────────────────────────────────────────────────────────

    /// Write the lexicon to a new file in the output directory. Existing files are never overwritten.
    pub fn save_lexicon(&mut self) {
        let Some(g) = &self.generated else {
            self.status = "Generate a lexicon first (Enter).".to_string();
            return;
        };
        match write_lexicon(&self.output_dir, &g.lexicon) {
            Ok(path) => self.status = format!("Saved {} entries to {}", g.lexicon.0.len(), path.display()),
            Err(e) => self.status = format!("Error saving: {}", e),
        }
    }
}

/// Write a lexicon as pretty JSON to `dir/lexicon_<unix-time>.json`, adding a numeric
/// suffix if that name is taken, so earlier saves are kept.
fn write_lexicon(dir: &Path, lexicon: &crate::lexicon_structs::Lexicon) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let json = serde_json::to_string_pretty(lexicon)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let mut path = dir.join(format!("lexicon_{}.json", stamp));
    let mut n = 1;
    while path.exists() {
        path = dir.join(format!("lexicon_{}_{}.json", stamp, n));
        n += 1;
    }
    std::fs::write(&path, json)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn app_in(dir: &Path) -> App {
        App::with_registries(Registries::builtin(), dir.to_path_buf(), String::new())
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("esoterica-tui-{}-{}", name, std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        dir
    }

    #[test]
    fn enter_generates_a_lexicon_with_summary() {
        let dir = temp_dir("generate");
        let mut app = app_in(&dir);
        assert!(app.handle_key(key(KeyCode::Enter)));
        assert!(app.generated.is_some());
        assert!(app.summary.contains("Lexicon:"), "{}", app.summary);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn q_and_ctrl_c_quit() {
        let dir = temp_dir("quit");
        let mut app = app_in(&dir);
        assert!(!app.handle_key(key(KeyCode::Char('q'))));
        assert!(!app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)));
    }

    #[test]
    fn lexicon_view_scroll_is_clamped() {
        let dir = temp_dir("scroll");
        let mut app = app_in(&dir);
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Char('l')));
        assert!(app.show_lexicon);
        for _ in 0..1000 { app.handle_key(key(KeyCode::Down)); }
        assert_eq!(app.lexicon_scroll, app.lexicon_len() - 1);
        app.handle_key(key(KeyCode::Esc));
        assert!(!app.show_lexicon);
    }

    #[test]
    fn lexicon_view_needs_a_generated_lexicon() {
        let dir = temp_dir("needs-gen");
        let mut app = app_in(&dir);
        app.handle_key(key(KeyCode::Char('l')));
        assert!(!app.show_lexicon);
        assert!(app.status.contains("Generate a lexicon first"));
    }

    #[test]
    fn save_writes_a_file_and_never_overwrites() {
        let dir = temp_dir("save");
        let mut app = app_in(&dir);
        app.handle_key(key(KeyCode::Char('s')));
        assert!(app.status.contains("Generate a lexicon first"));
        app.handle_key(key(KeyCode::Enter));
        app.handle_key(key(KeyCode::Char('s')));
        app.handle_key(key(KeyCode::Char('s')));
        let files: Vec<_> = std::fs::read_dir(&dir).unwrap().collect();
        assert_eq!(files.len(), 2, "each save should create a new file");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn space_toggles_sound_change_sets_in_the_config() {
        let dir = temp_dir("sc");
        let mut app = app_in(&dir);
        for _ in 0..3 { app.handle_key(key(KeyCode::Tab)); }
        app.handle_key(key(KeyCode::Char(' ')));
        assert_eq!(app.config.selection().sound_changes.len(), 1);
        app.handle_key(key(KeyCode::Char(' ')));
        assert!(app.config.selection().sound_changes.is_empty());
    }

    #[test]
    fn confirming_the_designer_selects_the_custom_phonology() {
        let dir = temp_dir("custom");
        let mut app = app_in(&dir);
        app.handle_key(key(KeyCode::Char('p')));
        app.handle_key(key(KeyCode::Char('y')));
        assert!(!app.designer.active);
        assert_eq!(app.config.selection().phonology, CUSTOM_PHONOLOGY);
        app.handle_key(key(KeyCode::Enter));
        assert!(app.generated.is_some());
        assert!(app.summary.contains("custom designer"));
    }

    #[test]
    fn invalid_data_dir_is_reported_in_status() {
        let dir = temp_dir("bad-data");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("phonologies.toml"), "[bad]\nvowels = []\nconsonants = [\"p\"]\nsyllable_structure = \"CV\"\n").unwrap();
        let args = Args::try_parse_from(["esoterica", "--interactive", "--data-dir", dir.to_str().unwrap()]).unwrap();
        let app = App::new(args);
        assert!(app.status.contains("Could not load"), "{}", app.status);
        std::fs::remove_dir_all(&dir).ok();
    }
}
