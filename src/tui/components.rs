use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, ListState},
    style::{Style, Color, Modifier},
    Frame,
};
use crossterm::event::KeyCode;
use crate::archetypes;

/// The Component trait defines the interface for UI sections.
pub trait Component {
    fn handle_event(&mut self, key_code: KeyCode) -> bool;
    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect);
}

pub struct ConfigComponent {
    phonology_options: Vec<String>,
    morphology_options: Vec<String>,
    syntax_options: Vec<String>,
    
    selected_field: usize, // 0: Phono, 1: Morph, 2: Syntax
    
    phono_list_state: ListState,
    morph_list_state: ListState,
    syntax_list_state: ListState,
}

impl Default for ConfigComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigComponent {
    pub fn new() -> Self {
        let phono_reg = archetypes::get_phonology_registry();
        let morph_reg = archetypes::get_morphology_registry();
        let syntax_reg = archetypes::get_syntax_registry();

        let mut phono_options: Vec<String> = phono_reg.keys().cloned().collect();
        phono_options.sort();
        let mut morph_options: Vec<String> = morph_reg.keys().cloned().collect();
        morph_options.sort();
        let mut syntax_options: Vec<String> = syntax_reg.keys().cloned().collect();
        syntax_options.sort();

        let mut phono_state = ListState::default();
        phono_state.select(Some(0));
        let mut morph_state = ListState::default();
        morph_state.select(Some(0));
        let mut syntax_state = ListState::default();
        syntax_state.select(Some(0));

        Self {
            phonology_options: phono_options,
            morphology_options: morph_options,
            syntax_options,
            selected_field: 0,
            phono_list_state: phono_state,
            morph_list_state: morph_state,
            syntax_list_state: syntax_state,
        }
    }

    pub fn get_selected_values(&self) -> (String, String, String) {
        let ph = self.phono_list_state.selected().and_then(|i| self.phonology_options.get(i)).cloned().unwrap_or_default();
        let mo = self.morph_list_state.selected().and_then(|i| self.morphology_options.get(i)).cloned().unwrap_or_default();
        let sy = self.syntax_list_state.selected().and_then(|i| self.syntax_options.get(i)).cloned().unwrap_or_default();
        (ph, mo, sy)
    }

    fn move_selection(&mut self, down: bool) {
        let (len, state) = match self.selected_field {
            0 => (self.phonology_options.len(), &mut self.phono_list_state),
            1 => (self.morphology_options.len(), &mut self.morph_list_state),
            2 => (self.syntax_options.len(), &mut self.syntax_list_state),
            _ => return,
        };
        let i = match state.selected() {
            Some(i) => if down { (i + 1) % len } else { (i + len - 1) % len },
            None => 0,
        };
        state.select(Some(i));
    }
}

impl Component for ConfigComponent {
    fn handle_event(&mut self, key_code: KeyCode) -> bool {
        match key_code {
            KeyCode::Tab => {
                self.selected_field = (self.selected_field + 1) % 3;
            }
            KeyCode::Up => {
                self.move_selection(false);
            }
            KeyCode::Down => {
                self.move_selection(true);
            }
            _ => return false,
        }
        true
    }

    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1,3), Constraint::Ratio(1,3), Constraint::Ratio(1,3)])
            .split(area);

        self.render_list(f, chunks[0], "Phonology", &self.phonology_options, &self.phono_list_state, self.selected_field == 0);
        self.render_list(f, chunks[1], "Morphology", &self.morphology_options, &self.morph_list_state, self.selected_field == 1);
        self.render_list(f, chunks[2], "Syntax", &self.syntax_options, &self.syntax_list_state, self.selected_field == 2);
    }
}

impl ConfigComponent {
    fn render_list(&self, f: &mut Frame, area: ratatui::layout::Rect, title: &str, items: &[String], state: &ListState, is_selected: bool) {
        let list_items: Vec<ListItem> = items.iter().map(|i| ListItem::new(i.as_str())).collect();
        let list = List::new(list_items)
            .block(Block::default().title(title).borders(Borders::ALL).border_style(if is_selected { Style::default().fg(Color::Yellow) } else { Style::default() }))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        
        let mut state = state.clone();
        f.render_stateful_widget(list, area, &mut state);
    }
}
