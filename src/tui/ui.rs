//! Ratatui rendering logic. Composes the layout from config panels, output,
//! and optional help overlay. Delegates rendering to component `render` methods.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::tui::app::App;
use crate::tui::components::Component;

/// Render a single frame: either the main view or the full-screen phonology designer.
pub fn render(f: &mut Frame, app: &App) {
    if app.designer.active {
        app.designer.render(f, f.size());
        return;
    }

    // Split screen: 80% config panels, 20% output display
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(f.size());
    app.config.render(f, chunks[0]);
    f.render_widget(
        Paragraph::new(app.output.as_str())
            .block(Block::default().title("Output [p: designer, h: help]").borders(Borders::ALL)),
        chunks[1],
    );

    // Help overlay centred on screen
    if app.show_help {
        let area = centered_rect(60, 40, f.size());
        app.help.render(f, area);
    }
}

/// Compute a centred sub-rectangle given percentage dimensions.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
