use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    style::{Style, Color, Modifier},
    Frame,
};
use crate::tui::app::App;

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(f.size());

    let items: Vec<ListItem> = app.fields.iter().enumerate().map(|(i, field)| {
        let content = match i {
            0 => format!("{}: {}", field, app.phonology),
            1 => format!("{}: {}", field, app.morphology),
            2 => format!("{}: {}", field, app.syntax),
            _ => field.clone(),
        };
        let style = if Some(i) == app.list_state.selected() {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(content).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().title("Configuration (Tab to select, Enter to generate)").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(list, chunks[0], &mut app.list_state.clone());
    
    f.render_widget(Paragraph::new(app.output.as_str()).block(Block::default().title("Output").borders(Borders::ALL)), chunks[1]);
}
