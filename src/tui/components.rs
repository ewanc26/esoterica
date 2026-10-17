//! Reusable UI components: config selector with four list panels, and a help overlay.
//! Follows a minimal Component trait for key handling and rendering.

use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState, Paragraph, Clear},
    style::{Style, Color, Modifier},
    Frame,
};
use crossterm::event::KeyCode;
use std::collections::BTreeSet;
use crate::pipeline::Registries;

/// Phonology list entry that selects the phonology built in the designer.
pub const CUSTOM_PHONOLOGY: &str = "(custom designer)";

/// Common interface for TUI widget components.
pub trait Component {
    fn handle_event(&mut self, key_code: KeyCode) -> bool;
    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect);
}

/// The configuration chosen in the four panels.
#[derive(Debug, Clone, PartialEq)]
pub struct Selection {
    pub phonology: String,
    pub morphology: String,
    pub syntax: String,
    /// Sound-change set keys, in alphabetical order.
    pub sound_changes: Vec<String>,
}

// ── Config Selection Panel ──────────────────────────────────────────────

/// Four-column config browser: phonology, morphology, syntax, sound changes.
/// Tab moves between columns, Up/Down navigates, and Space toggles sound-change sets.
pub struct ConfigComponent {
    phonology_options: Vec<String>,
    morphology_options: Vec<String>,
    syntax_options: Vec<String>,
    sound_change_options: Vec<String>,
    selected_field: usize,
    phono_list_state: ListState,
    morph_list_state: ListState,
    syntax_list_state: ListState,
    sound_change_list_state: ListState,
    sound_changes_on: BTreeSet<String>,
}

impl ConfigComponent {
    pub fn new(registries: &Registries) -> Self {
        let mut phonology_options = vec![CUSTOM_PHONOLOGY.to_string()];
        let mut built_in: Vec<String> = registries.phonologies.keys().cloned().collect();
        built_in.sort();
        phonology_options.extend(built_in);

        let mut morph_options: Vec<String> = registries.morphologies.keys().cloned().collect();
        morph_options.sort();
        let mut syntax_options: Vec<String> = registries.syntaxes.keys().cloned().collect();
        syntax_options.sort();
        let mut sc_options: Vec<String> = registries.sound_changes.keys().cloned().collect();
        sc_options.sort();

        let first = |len: usize| {
            let mut state = ListState::default();
            if len > 0 { state.select(Some(0)); }
            state
        };
        let default_phono = phonology_options.iter().position(|p| p == "uralic_finnic").unwrap_or(0);
        let default_morph = morph_options.iter().position(|p| p == "agglutinative").unwrap_or(0);
        let default_syntax = syntax_options.iter().position(|p| p == "svo").unwrap_or(0);

        let mut phono_state = first(phonology_options.len());
        phono_state.select(Some(default_phono));
        let mut morph_state = first(morph_options.len());
        morph_state.select(Some(default_morph));
        let mut syntax_state = first(syntax_options.len());
        syntax_state.select(Some(default_syntax));
        let sc_state = first(sc_options.len());

        Self {
            phonology_options,
            morphology_options: morph_options,
            syntax_options,
            sound_change_options: sc_options,
            selected_field: 0,
            phono_list_state: phono_state,
            morph_list_state: morph_state,
            syntax_list_state: syntax_state,
            sound_change_list_state: sc_state,
            sound_changes_on: BTreeSet::new(),
        }
    }

    /// The current selection across all four panels.
    pub fn selection(&self) -> Selection {
        let pick = |state: &ListState, options: &[String]| {
            state.selected().and_then(|i| options.get(i)).cloned().unwrap_or_default()
        };
        Selection {
            phonology: pick(&self.phono_list_state, &self.phonology_options),
            morphology: pick(&self.morph_list_state, &self.morphology_options),
            syntax: pick(&self.syntax_list_state, &self.syntax_options),
            sound_changes: self.sound_changes_on.iter().cloned().collect(),
        }
    }

    /// Select the designer's phonology in the phonology panel.
    pub fn select_custom(&mut self) {
        self.phono_list_state.select(Some(0));
    }

    /// Move the selection cursor within the active field's list.
    fn move_selection(&mut self, down: bool) {
        let (len, state) = match self.selected_field {
            0 => (self.phonology_options.len(), &mut self.phono_list_state),
            1 => (self.morphology_options.len(), &mut self.morph_list_state),
            2 => (self.syntax_options.len(), &mut self.syntax_list_state),
            3 => (self.sound_change_options.len(), &mut self.sound_change_list_state),
            _ => return,
        };
        if len == 0 { return; }
        let i = match state.selected() {
            Some(i) => if down { (i + 1) % len } else { (i + len - 1) % len },
            None => 0,
        };
        state.select(Some(i));
    }

    /// Toggle the sound-change set under the cursor on or off.
    fn toggle_sound_change(&mut self) {
        if let Some(key) = self.sound_change_list_state.selected().and_then(|i| self.sound_change_options.get(i)) {
            if !self.sound_changes_on.remove(key) {
                self.sound_changes_on.insert(key.clone());
            }
        }
    }
}

impl Component for ConfigComponent {
    fn handle_event(&mut self, key_code: KeyCode) -> bool {
        match key_code {
            KeyCode::Tab => { self.selected_field = (self.selected_field + 1) % 4; }
            KeyCode::BackTab => { self.selected_field = (self.selected_field + 3) % 4; }
            KeyCode::Up => { self.move_selection(false); }
            KeyCode::Down => { self.move_selection(true); }
            KeyCode::Char(' ') if self.selected_field == 3 => { self.toggle_sound_change(); }
            _ => return false,
        }
        true
    }

    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default().direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1,4), Constraint::Ratio(1,4), Constraint::Ratio(1,4), Constraint::Ratio(1,4)])
            .split(area);
        self.render_list(f, chunks[0], "Phonology", &self.phonology_options, &self.phono_list_state, self.selected_field == 0);
        self.render_list(f, chunks[1], "Morphology", &self.morphology_options, &self.morph_list_state, self.selected_field == 1);
        self.render_list(f, chunks[2], "Syntax", &self.syntax_options, &self.syntax_list_state, self.selected_field == 2);
        let sc_labels: Vec<String> = self.sound_change_options.iter()
            .map(|k| format!("[{}] {}", if self.sound_changes_on.contains(k) { "x" } else { " " }, k))
            .collect();
        self.render_list(f, chunks[3], "Sound Changes (Space)", &sc_labels, &self.sound_change_list_state, self.selected_field == 3);
    }
}

impl ConfigComponent {
    /// Render a single scrollable list panel with title and scrollbar.
    fn render_list(&self, f: &mut Frame, area: ratatui::layout::Rect, title: &str, items: &[String], state: &ListState, is_selected: bool) {
        let list_items: Vec<ListItem> = items.iter().map(|i| ListItem::new(i.as_str())).collect();
        let list = List::new(list_items)
            .block(Block::default().title(title).borders(Borders::ALL)
                .border_style(if is_selected { Style::default().fg(Color::Yellow) } else { Style::default() }))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        let mut state = state.clone();
        f.render_stateful_widget(list, area, &mut state);
        let scrollbar = Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight);
        let mut scrollbar_state = ScrollbarState::new(items.len()).position(state.selected().unwrap_or(0));
        f.render_stateful_widget(scrollbar, area.inner(&ratatui::layout::Margin { vertical: 1, horizontal: 0 }), &mut scrollbar_state);
    }
}

// ── Help Overlay ────────────────────────────────────────────────────────

pub const HELP_TEXT: &str = "\
Configure
  Tab / Shift-Tab   switch panel
  Up / Down         move in panel
  Space             toggle sound-change set
Actions
  Enter             generate lexicon
  l                 browse lexicon (Up/Down, PgUp/PgDn, Esc closes)
  s                 save lexicon to esoterica_out/
  p                 phonology designer (Tab switches tabs, y confirms)
General
  h                 toggle this help
  q / Ctrl-C        quit";

pub struct HelpComponent;

impl HelpComponent {
    pub fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        f.render_widget(Clear, area);
        f.render_widget(
            Paragraph::new(HELP_TEXT).block(Block::default().title("Help").borders(Borders::ALL)),
            area,
        );
    }
}
