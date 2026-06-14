use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::tui::app::App;
use crate::tui::components::Component;

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(f.size());

    app.config.render(f, chunks[0]);
    
    f.render_widget(Paragraph::new(app.output.as_str()).block(Block::default().title("Output").borders(Borders::ALL)), chunks[1]);
}
