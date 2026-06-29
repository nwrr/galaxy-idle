//! Panel SHORTCUTS (sidebar bawah): keybinding ringkas. `06` §Keybindings.
#![allow(dead_code)]

use crate::app::App;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

const KEYS: [&str; 5] = ["g:Galaxy", "p:Planet", "r:Research", "m:Merchant", "?:Help"];

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let lines: Vec<Line> = std::iter::once(Line::styled(
        "SHORTCUTS",
        Style::default().fg(th.header).bold(),
    ))
    .chain(
        KEYS.iter()
            .map(|e| Line::styled(*e, Style::default().fg(th.dim))),
    )
    .collect();
    f.render_widget(Paragraph::new(lines).block(Block::bordered()), area);
}
