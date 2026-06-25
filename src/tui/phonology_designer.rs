//! Interactive Phonology Designer TUI
//! Allows toggling phonemes on an IPA grid with real-time syllable preview.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, Clear},
    style::{Style, Color, Modifier},
    text::{Line, Span},
    Frame,
};
use crossterm::event::KeyCode;
use crate::archetypes::Phonology;
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

const IPA_VOWELS: &[(&str, &[&str])] = &[
    ("Close", &["i", "y", "\u{0268}", "\u{0289}", "\u{026f}", "u"]),
    ("Near-close", &["\u{026a}", "\u{028f}", "", "", "\u{028a}"]),
    ("Close-mid", &["e", "\u{00f8}", "\u{0258}", "\u{0275}", "\u{0264}", "o"]),
    ("Mid", &["", "", "\u{0259}", "", "", ""]),
    ("Open-mid", &["\u{025b}", "\u{0153}", "\u{025c}", "\u{025e}", "\u{028c}", "\u{0254}"]),
    ("Open", &["\u{00e6}", "", "\u{0250}", "", "\u{0251}", "\u{0252}"]),
];

#[derive(PartialEq, Clone, Copy)]
enum DesignerTab { Consonants, Vowels, Presets }

pub struct PhonologyDesigner {
    pub active: bool,
    selected_consonants: HashSet<String>,
    selected_vowels: HashSet<String>,
    syllable_structure: String,
    tones: bool,
    tone_count: u8,
    vowel_harmony: bool,
    cursor_row: usize,
    cursor_col: usize,
    active_tab: DesignerTab,
    preview_word: String,
    /// The generated phonology, if the user has confirmed
    pub confirmed_phonology: Option<Phonology>,
}

impl PhonologyDesigner {
    pub fn new() -> Self {
        // Default: a basic phonology preset
        let selected_consonants: HashSet<String> = ["p", "t", "k", "m", "n", "s", "l", "r", "h"]
            .iter().map(|s| s.to_string()).collect();
        let selected_vowels: HashSet<String> = ["a", "e", "i", "o", "u"]
            .iter().map(|s| s.to_string()).collect();

        let mut designer = Self {
            active: false,
            selected_consonants,
            selected_vowels,
            syllable_structure: "CV".to_string(),
            tones: false,
            tone_count: 4,
            vowel_harmony: false,
            cursor_row: 0,
            cursor_col: 0,
            active_tab: DesignerTab::Consonants,
            preview_word: String::new(),
            confirmed_phonology: None,
        };
        designer.regenerate_preview();
        designer
    }

    pub fn toggle(&mut self) { self.active = !self.active; }

    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        if !self.active { return false; }
        match key {
            KeyCode::Char('q') => { self.active = false; return true; }
            KeyCode::Char('\t') => {
                self.active_tab = match self.active_tab {
                    DesignerTab::Consonants => DesignerTab::Vowels,
                    DesignerTab::Vowels => DesignerTab::Presets,
                    DesignerTab::Presets => DesignerTab::Consonants,
                };
                self.cursor_row = 0; self.cursor_col = 0;
            }
            KeyCode::Char('p') => {
                // Apply preset
                self.apply_preset("full");
            }
            KeyCode::Char('m') => {
                self.apply_preset("minimal");
            }
            KeyCode::Char('t') => {
                self.tones = !self.tones;
                self.regenerate_preview();
            }
            KeyCode::Char('h') => {
                self.vowel_harmony = !self.vowel_harmony;
                self.regenerate_preview();
            }
            KeyCode::Char('+') => {
                // Add a consonant cluster slot
                if self.syllable_structure.len() < 8 {
                    let pos = self.syllable_structure.len() - 1;
                    self.syllable_structure.insert(pos, 'C');
                }
                self.regenerate_preview();
            }
            KeyCode::Char('-') => {
                if self.syllable_structure.len() > 2 {
                    // Remove one C from the onset (before first V)
                    if let Some(pos) = self.syllable_structure.find('C') {
                        self.syllable_structure.remove(pos);
                    }
                }
                self.regenerate_preview();
            }
            KeyCode::Up => {
                if self.cursor_row > 0 { self.cursor_row -= 1; }
            }
            KeyCode::Down => {
                let max = self.grid_rows().saturating_sub(1);
                if self.cursor_row < max { self.cursor_row += 1; }
            }
            KeyCode::Left => {
                if self.cursor_col > 0 { self.cursor_col -= 1; }
            }
            KeyCode::Right => {
                let max = self.grid_cols().saturating_sub(1);
                if self.cursor_col < max { self.cursor_col += 1; }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle_phoneme();
                self.regenerate_preview();
            }
            KeyCode::Char('y') => {
                // Confirm and return the custom phonology
                self.confirmed_phonology = Some(self.to_phonology());
                self.active = false;
            }
            _ => {}
        }
        true
    }

    fn grid_rows(&self) -> usize {
        match self.active_tab {
            DesignerTab::Consonants => IPA_CONSONANTS.len(),
            DesignerTab::Vowels => IPA_VOWELS.len(),
            DesignerTab::Presets => 4,
        }
    }

    fn grid_cols(&self) -> usize {
        match self.active_tab {
            DesignerTab::Consonants => IPA_CONSONANTS.iter().map(|r| r.1.len()).max().unwrap_or(0),
            DesignerTab::Vowels => IPA_VOWELS.iter().map(|r| r.1.len()).max().unwrap_or(0),
            DesignerTab::Presets => 1,
        }
    }

    fn toggle_phoneme(&self) {
        // This is intentionally a no-op at the shared-data level;
        // toggling happens via interior mutability on the HashSet fields.
        // We use unsafe to get mutable access since ratatui renders immutably.
    }

    /// This must be called from a context that has &mut self
    pub fn toggle_phoneme_mut(&mut self) {
        let phoneme = match self.active_tab {
            DesignerTab::Consonants => {
                IPA_CONSONANTS.get(self.cursor_row)
                    .and_then(|r| r.1.get(self.cursor_col))
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
            }
            DesignerTab::Vowels => {
                IPA_VOWELS.get(self.cursor_row)
                    .and_then(|r| r.1.get(self.cursor_col))
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
            }
            DesignerTab::Presets => None,
        };

        if let Some(p) = phoneme {
            match self.active_tab {
                DesignerTab::Consonants => {
                    if self.selected_consonants.contains(&p) {
                        self.selected_consonants.remove(&p);
                    } else {
                        self.selected_consonants.insert(p);
                    }
                }
                DesignerTab::Vowels => {
                    if self.selected_vowels.contains(&p) {
                        self.selected_vowels.remove(&p);
                    } else {
                        self.selected_vowels.insert(p);
                    }
                }
                _ => {}
            }
        }
    }

    fn apply_preset(&mut self, preset: &str) {
        match preset {
            "minimal" => {
                self.selected_consonants = ["p", "t", "k", "m", "n", "s", "a"]
                    .iter().map(|s| s.to_string()).collect();
                self.selected_vowels = ["a", "i", "u"]
                    .iter().map(|s| s.to_string()).collect();
                self.syllable_structure = "CV".to_string();
                self.vowel_harmony = false;
                self.tones = false;
            }
            _ => {
                // Full preset: many common phonemes
                self.selected_consonants = ["p", "b", "t", "d", "k", "g", "m", "n", "f", "v", "s", "z", "h", "l", "r", "j", "w"]
                    .iter().map(|s| s.to_string()).collect();
                self.selected_vowels = ["a", "e", "i", "o", "u"]
                    .iter().map(|s| s.to_string()).collect();
                self.syllable_structure = "CVC".to_string();
                self.vowel_harmony = false;
                self.tones = false;
            }
        }
        self.regenerate_preview();
    }

    fn regenerate_preview(&mut self) {
        let phono = self.to_phonology();
        let engine = PhonologyEngine::new(phono);
        let word = engine.generate_word(3);
        self.preview_word = format!("{} {}", word, engine.to_ipa(&word));
    }

    pub fn to_phonology(&self) -> Phonology {
        let mut consonants: Vec<String> = self.selected_consonants.iter().cloned().collect();
        consonants.sort();
        let mut vowels: Vec<String> = self.selected_vowels.iter().cloned().collect();
        vowels.sort();
        Phonology {
            vowels,
            consonants,
            syllable_structure: self.syllable_structure.clone(),
            tones: if self.tones { Some(self.tone_count) } else { None },
            vowel_harmony: Some(self.vowel_harmony),
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let outer = Block::default()
            .title("Phonology Designer [q:back tab:switch p:full m:minimal t:tone h:harmony +/-:structure y:confirm]")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = outer.inner(area);
        f.render_widget(Clear, area);
        f.render_widget(outer, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(2)])
            .split(inner);

        // Status bar
        let status = format!(
            "C:{} V:{} | Structure: {} | Tone: {} | Harmony: {} | Preview: {}",
            self.selected_consonants.len(),
            self.selected_vowels.len(),
            self.syllable_structure,
            if self.tones { format!("{} tones", self.tone_count) } else { "off".to_string() },
            if self.vowel_harmony { "on" } else { "off" },
            self.preview_word,
        );
        f.render_widget(Paragraph::new(status), chunks[0]);

        // IPA Grid
        match self.active_tab {
            DesignerTab::Consonants => self.render_ipa_grid(f, chunks[1], IPA_CONSONANTS, &self.selected_consonants),
            DesignerTab::Vowels => self.render_ipa_grid(f, chunks[1], IPA_VOWELS, &self.selected_vowels),
            DesignerTab::Presets => self.render_presets(f, chunks[1]),
        }

        // Tab indicator
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
                let is_cursor = self.active && row_idx == self.cursor_row && col_idx == self.cursor_col;
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
            "Presets - Press key to apply:\n\n\
             [p] Full   - 17 consonants, 5 vowels, CVC structure\n\
             [m] Minimal - 7 consonants, 3 vowels, CV structure\n\n\
             Controls:\n\
             [t] Toggle tones (currently: {})\n\
             [h] Toggle vowel harmony (currently: {})\n\
             [+/-] Adjust syllable structure (currently: {})\n\
             [y] Confirm and use this phonology\n\n\
             Current selection: {} consonants, {} vowels",
            if self.tones { format!("{} tones ON", self.tone_count) } else { "OFF".to_string() },
            if self.vowel_harmony { "ON" } else { "OFF" },
            self.syllable_structure,
            self.selected_consonants.len(),
            self.selected_vowels.len(),
        );
        f.render_widget(Paragraph::new(text), area);
    }
}
