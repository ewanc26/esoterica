//! Interactive Phonology Designer TUI.
//! Browse the IPA chart, toggle consonants and vowels, pick a syllable template,
//! toggle tone and vowel harmony, then confirm. The preview updates on every change.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, Clear},
    style::{Style, Color, Modifier},
    text::{Line, Span},
    Frame,
};
use crossterm::event::KeyCode;
use crate::archetypes::{validate_template, Phonology};
use crate::phonology::PhonologyEngine;
use std::collections::HashSet;

/// Full IPA consonant chart organized by place/manner of articulation
const IPA_CONSONANTS: &[(&str, &[&str])] = &[
    ("Plosive", &["p", "b", "t", "d", "\u{0288}", "\u{0256}", "c", "\u{025f}", "k", "g", "q", "\u{0262}", "\u{0294}"]),
    ("Nasal", &["m", "\u{0271}", "n", "\u{0273}", "\u{0272}", "\u{014b}", "\u{0274}"]),
    ("Fricative", &["\u{0278}", "\u{03b2}", "f", "v", "\u{03b8}", "\u{00f0}", "s", "z", "\u{0283}", "\u{0292}", "x", "\u{0263}", "h", "\u{0266}"]),
    ("Affricate", &["ts", "dz", "t\u{0283}", "d\u{0292}"]),
    ("Approximant", &["w", "\u{028b}", "l", "j", "\u{0279}", "\u{027b}"]),
    ("Trill/Tap", &["\u{027e}", "r", "\u{0280}"]),
];

/// IPA vowel chart organized by vowel height (rows) and backness (columns).
const IPA_VOWELS: &[(&str, &[&str])] = &[
    ("Close", &["i", "y", "\u{0268}", "\u{0289}", "\u{026f}", "u"]),
    ("Near-close", &["\u{026a}", "\u{028f}", "", "", "\u{028a}"]),
    ("Close-mid", &["e", "\u{00f8}", "\u{0258}", "\u{0275}", "\u{0264}", "o"]),
    ("Mid", &["", "", "\u{0259}", "", "", ""]),
    ("Open-mid", &["\u{025b}", "\u{0153}", "\u{025c}", "\u{025e}", "\u{028c}", "\u{0254}"]),
    ("Open", &["\u{00e6}", "", "\u{0250}", "", "\u{0251}", "\u{0252}"]),
];

/// Syllable templates the designer cycles through with + and -. Every entry is valid.
const STRUCTURES: &[&str] = &["V", "CV", "CVC", "CCVC", "CCVCC"];
const DEFAULT_STRUCTURE: usize = 1;
const MIN_TONES: u8 = 2;
const MAX_TONES: u8 = 9;

const MINIMAL_CONSONANTS: &[&str] = &["p", "t", "k", "m", "n", "s", "l"];
const MINIMAL_VOWELS: &[&str] = &["a", "i", "u"];
const FULL_CONSONANTS: &[&str] = &["p", "b", "t", "d", "k", "g", "m", "n", "f", "v", "s", "z", "h", "l", "r", "j", "w"];
const FULL_VOWELS: &[&str] = &["a", "e", "i", "o", "u"];

#[derive(PartialEq, Clone, Copy, Debug)]
enum DesignerTab { Consonants, Vowels, Presets }

// ── Phonology Designer ──────────────────────────────────────────────────

pub struct PhonologyDesigner {
    /// True while the designer screen is open.
    pub active: bool,
    selected_consonants: HashSet<String>,
    selected_vowels: HashSet<String>,
    structure: usize,
    tones: bool,
    tone_count: u8,
    vowel_harmony: bool,
    cursor_row: usize,
    cursor_col: usize,
    active_tab: DesignerTab,
    preview_word: String,
    confirmed: bool,
}

impl Default for PhonologyDesigner {
    fn default() -> Self { Self::new() }
}

impl PhonologyDesigner {
    pub fn new() -> Self {
        let mut designer = Self {
            active: false,
            selected_consonants: HashSet::new(),
            selected_vowels: HashSet::new(),
            structure: DEFAULT_STRUCTURE,
            tones: false,
            tone_count: 4,
            vowel_harmony: false,
            cursor_row: 0,
            cursor_col: 0,
            active_tab: DesignerTab::Consonants,
            preview_word: String::new(),
            confirmed: false,
        };
        designer.apply_minimal();
        designer
    }

    pub fn toggle(&mut self) { self.active = !self.active; }

    /// True once, after the user has confirmed the design with `y`.
    pub fn take_confirmed(&mut self) -> bool {
        std::mem::take(&mut self.confirmed)
    }

    // ── Key Handling ────────────────────────────────────────────────────────

    /// Handle a key while the designer is open. Returns true if the key was used.
    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        if !self.active { return false; }
        match key {
            KeyCode::Char('q') | KeyCode::Esc => { self.active = false; }
            KeyCode::Tab => {
                self.active_tab = match self.active_tab {
                    DesignerTab::Consonants => DesignerTab::Vowels,
                    DesignerTab::Vowels => DesignerTab::Presets,
                    DesignerTab::Presets => DesignerTab::Consonants,
                };
                self.cursor_row = 0;
                self.cursor_col = 0;
            }
            KeyCode::Char('p') => self.apply_full(),
            KeyCode::Char('m') => self.apply_minimal(),
            KeyCode::Char('t') => { self.tones = !self.tones; self.regenerate_preview(); }
            KeyCode::Char('[') => { self.tone_count = self.tone_count.saturating_sub(1).max(MIN_TONES); self.regenerate_preview(); }
            KeyCode::Char(']') => { self.tone_count = (self.tone_count + 1).min(MAX_TONES); self.regenerate_preview(); }
            KeyCode::Char('h') => { self.vowel_harmony = !self.vowel_harmony; self.regenerate_preview(); }
            KeyCode::Char('+') => {
                self.structure = (self.structure + 1).min(STRUCTURES.len() - 1);
                self.regenerate_preview();
            }
            KeyCode::Char('-') => {
                self.structure = self.structure.saturating_sub(1);
                self.regenerate_preview();
            }
            KeyCode::Up => { self.cursor_row = self.cursor_row.saturating_sub(1); }
            KeyCode::Down => {
                let max = self.grid_rows().saturating_sub(1);
                if self.cursor_row < max { self.cursor_row += 1; }
            }
            KeyCode::Left => { self.cursor_col = self.cursor_col.saturating_sub(1); }
            KeyCode::Right => {
                let max = self.grid_cols().saturating_sub(1);
                if self.cursor_col < max { self.cursor_col += 1; }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle_phoneme_at_cursor();
                self.regenerate_preview();
            }
            KeyCode::Char('y') => {
                self.confirmed = true;
                self.active = false;
            }
            _ => return false,
        }
        true
    }

    fn grid_rows(&self) -> usize {
        match self.active_tab {
            DesignerTab::Consonants => IPA_CONSONANTS.len(),
            DesignerTab::Vowels => IPA_VOWELS.len(),
            DesignerTab::Presets => 1,
        }
    }

    fn grid_cols(&self) -> usize {
        match self.active_tab {
            DesignerTab::Consonants => IPA_CONSONANTS.iter().map(|r| r.1.len()).max().unwrap_or(0),
            DesignerTab::Vowels => IPA_VOWELS.iter().map(|r| r.1.len()).max().unwrap_or(0),
            DesignerTab::Presets => 1,
        }
    }

    /// The IPA symbol under the cursor on the active grid, if it is a real phoneme.
    fn phoneme_at_cursor(&self) -> Option<&'static str> {
        let grid = match self.active_tab {
            DesignerTab::Consonants => IPA_CONSONANTS,
            DesignerTab::Vowels => IPA_VOWELS,
            DesignerTab::Presets => return None,
        };
        grid.get(self.cursor_row)
            .and_then(|r| r.1.get(self.cursor_col))
            .copied()
            .filter(|s| !s.is_empty())
    }

    fn toggle_phoneme_at_cursor(&mut self) {
        let Some(p) = self.phoneme_at_cursor() else { return };
        let set = match self.active_tab {
            DesignerTab::Consonants => &mut self.selected_consonants,
            DesignerTab::Vowels => &mut self.selected_vowels,
            DesignerTab::Presets => return,
        };
        if !set.remove(p) {
            set.insert(p.to_string());
        }
    }

    fn apply_minimal(&mut self) {
        self.selected_consonants = MINIMAL_CONSONANTS.iter().map(|s| s.to_string()).collect();
        self.selected_vowels = MINIMAL_VOWELS.iter().map(|s| s.to_string()).collect();
        self.structure = 1; // CV
        self.vowel_harmony = false;
        self.tones = false;
        self.regenerate_preview();
    }

    fn apply_full(&mut self) {
        self.selected_consonants = FULL_CONSONANTS.iter().map(|s| s.to_string()).collect();
        self.selected_vowels = FULL_VOWELS.iter().map(|s| s.to_string()).collect();
        self.structure = 2; // CVC
        self.vowel_harmony = false;
        self.tones = false;
        self.regenerate_preview();
    }

    fn regenerate_preview(&mut self) {
        let engine = PhonologyEngine::new(self.to_phonology());
        let word = engine.generate_word(3);
        self.preview_word = format!("{} {}", word, engine.to_ipa(&word));
    }

    /// The phonology described by the current selections.
    pub fn to_phonology(&self) -> Phonology {
        let mut consonants: Vec<String> = self.selected_consonants.iter().cloned().collect();
        consonants.sort();
        let mut vowels: Vec<String> = self.selected_vowels.iter().cloned().collect();
        vowels.sort();
        let syllable_structure = STRUCTURES[self.structure].to_string();
        debug_assert!(validate_template(&syllable_structure).is_ok());
        Phonology {
            vowels,
            consonants,
            syllable_structure,
            tones: if self.tones { Some(self.tone_count) } else { None },
            vowel_harmony: Some(self.vowel_harmony),
        }
    }

    // ── Rendering ──────────────────────────────────────────────────────────

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let outer = Block::default()
            .title("Phonology Designer [q/Esc:back Tab:tab p:full m:minimal t:tone []:count h:harmony +/-:structure y:use]")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = outer.inner(area);
        f.render_widget(Clear, area);
        f.render_widget(outer, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(2)])
            .split(inner);

        let status = format!(
            "C:{} V:{} | Structure: {} | Tone: {} | Harmony: {}\nPreview: {}",
            self.selected_consonants.len(),
            self.selected_vowels.len(),
            STRUCTURES[self.structure],
            if self.tones { format!("{} tones", self.tone_count) } else { "off".to_string() },
            if self.vowel_harmony { "on" } else { "off" },
            self.preview_word,
        );
        f.render_widget(Paragraph::new(status), chunks[0]);

        match self.active_tab {
            DesignerTab::Consonants => self.render_ipa_grid(f, chunks[1], IPA_CONSONANTS, &self.selected_consonants),
            DesignerTab::Vowels => self.render_ipa_grid(f, chunks[1], IPA_VOWELS, &self.selected_vowels),
            DesignerTab::Presets => self.render_presets(f, chunks[1]),
        }

        let indicator = match self.active_tab {
            DesignerTab::Consonants => " [Consonants] | Vowels | Presets ",
            DesignerTab::Vowels => " Consonants | [Vowels] | Presets ",
            DesignerTab::Presets => " Consonants | Vowels | [Presets] ",
        };
        f.render_widget(
            Paragraph::new(indicator).style(Style::default().fg(Color::Yellow)),
            chunks[2],
        );
    }

    fn render_ipa_grid(&self, f: &mut Frame, area: Rect, grid: &[(&str, &[&str])], selected: &HashSet<String>) {
        let mut lines: Vec<Line> = Vec::new();
        for (row_idx, (label, phonemes)) in grid.iter().enumerate() {
            let mut spans: Vec<Span> = vec![
                Span::styled(format!("{:15}", label), Style::default().fg(Color::Gray))
            ];
            for (col_idx, phoneme) in phonemes.iter().enumerate() {
                if phoneme.is_empty() {
                    spans.push(Span::raw("   "));
                    continue;
                }
                let is_selected = selected.contains(*phoneme);
                let is_cursor = row_idx == self.cursor_row && col_idx == self.cursor_col;
                let mut style = if is_selected {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                if is_cursor {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                spans.push(Span::styled(format!(" {} ", phoneme), style));
            }
            lines.push(Line::from(spans));
        }
        f.render_widget(Paragraph::new(lines), area);
    }

    fn render_presets(&self, f: &mut Frame, area: Rect) {
        let text = format!(
            "Presets - press a key to apply:\n\n\
             [p] Full    - {} consonants, {} vowels, CVC structure\n\
             [m] Minimal - {} consonants, {} vowels, CV structure\n\n\
             Controls:\n\
             [t] Tones: {}     [ / ] tone count: {}\n\
             [h] Vowel harmony: {}\n\
             [+ / -] Syllable structure: {}\n\
             [y] Use this phonology in generation\n\n\
             Current selection: {} consonants, {} vowels",
            FULL_CONSONANTS.len(), FULL_VOWELS.len(),
            MINIMAL_CONSONANTS.len(), MINIMAL_VOWELS.len(),
            if self.tones { "ON" } else { "OFF" }, self.tone_count,
            if self.vowel_harmony { "ON" } else { "OFF" },
            STRUCTURES[self.structure],
            self.selected_consonants.len(),
            self.selected_vowels.len(),
        );
        f.render_widget(Paragraph::new(text), area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_selection_is_a_valid_phonology() {
        assert!(PhonologyDesigner::new().to_phonology().validate().is_ok());
    }

    #[test]
    fn empty_vowel_inventory_is_rejected_by_validation() {
        let mut designer = PhonologyDesigner::new();
        designer.selected_vowels.clear();
        assert!(designer.to_phonology().validate().is_err());
    }

    #[test]
    fn minimal_preset_has_no_vowel_as_consonant() {
        let mut designer = PhonologyDesigner::new();
        designer.handle_key(KeyCode::Char('m'));
        let p = designer.to_phonology();
        assert!(!p.consonants.iter().any(|c| c == "a"), "{:?}", p.consonants);
        assert_eq!(p.consonants.len(), 7);
    }

    #[test]
    fn structure_cycling_only_yields_valid_templates() {
        let mut designer = PhonologyDesigner::new();
        designer.active = true;
        for _ in 0..10 { designer.handle_key(KeyCode::Char('+')); assert!(validate_template(&designer.to_phonology().syllable_structure).is_ok()); }
        for _ in 0..10 { designer.handle_key(KeyCode::Char('-')); assert!(validate_template(&designer.to_phonology().syllable_structure).is_ok()); }
    }

    #[test]
    fn tab_cycles_grids_and_enter_toggles_with_a_fresh_preview() {
        let mut designer = PhonologyDesigner::new();
        designer.active = true;
        designer.handle_key(KeyCode::Tab);
        assert_eq!(designer.active_tab, DesignerTab::Vowels);
        designer.handle_key(KeyCode::Tab);
        assert_eq!(designer.active_tab, DesignerTab::Presets);
        designer.handle_key(KeyCode::Tab);
        assert_eq!(designer.active_tab, DesignerTab::Consonants);

        // Cursor starts on "p" (row 0, col 0), which is selected by default: Enter removes it.
        let before = designer.selected_consonants.len();
        designer.handle_key(KeyCode::Enter);
        assert_eq!(designer.selected_consonants.len(), before - 1);
        assert!(!designer.to_phonology().consonants.contains(&"p".to_string()));
    }

    #[test]
    fn tone_count_stays_in_range() {
        let mut designer = PhonologyDesigner::new();
        designer.active = true;
        for _ in 0..20 { designer.handle_key(KeyCode::Char(']')); }
        assert_eq!(designer.tone_count, MAX_TONES);
        for _ in 0..20 { designer.handle_key(KeyCode::Char('[')); }
        assert_eq!(designer.tone_count, MIN_TONES);
    }

    #[test]
    fn confirm_closes_the_designer_and_signals_once() {
        let mut designer = PhonologyDesigner::new();
        designer.active = true;
        designer.handle_key(KeyCode::Char('y'));
        assert!(!designer.active);
        assert!(designer.take_confirmed());
        assert!(!designer.take_confirmed());
    }
}

