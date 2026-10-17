//! Ratatui rendering logic. Composes the main layout (configuration panels and
//! output), the full-screen designer and lexicon browser, and the help overlay.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use crate::tui::app::App;
use crate::tui::components::Component;

/// Render a single frame.
pub fn render(f: &mut Frame, app: &App) {
    if app.designer.active {
        app.designer.render(f, f.size());
        return;
    }
    if app.show_lexicon {
        render_lexicon(f, app, f.size());
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(f.size());
    app.config.render(f, chunks[0]);

    let mut text = app.summary.clone();
    if !app.status.is_empty() {
        if !text.is_empty() { text.push_str("\n\n"); }
        text.push_str(&app.status);
    }
    f.render_widget(
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .block(Block::default()
                .title("Output  [Enter generate · l lexicon · s save · p designer · h help · q quit]")
                .borders(Borders::ALL)),
        chunks[1],
    );

    if app.show_help {
        let area = centered_rect(60, 70, f.size());
        app.help.render(f, area);
    }
}

/// Full-screen lexicon browser with a scrolling list of headwords.
fn render_lexicon(f: &mut Frame, app: &App, area: Rect) {
    let lines = app.lexicon_lines();
    let title = format!(
        "Lexicon: {} entries  [Up/Down, PgUp/PgDn, Home/End scroll · l/Esc close · q quit]",
        lines.len()
    );
    let block = Block::default().title(title).borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let visible = inner.height.saturating_sub(1) as usize;
    let header = format!("{:<18} {:<22} {:<10} {}", "Headword", "IPA", "POS", "Definition");
    let mut body = vec![header];
    body.extend(lines.iter().skip(app.lexicon_scroll).take(visible).cloned());
    f.render_widget(Paragraph::new(body.join("\n")), inner);
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
