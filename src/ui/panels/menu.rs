//! Panel MENU (sidebar): daftar view + penanda fokus `►`. `06` §MENU.
#![allow(dead_code)]

use crate::app::App;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

const ENTRIES: [&str; 10] = [
    "\u{25ba} Galaxy Map",
    "  Planets",
    "   * Earth",
    "   o Mars",
    "   o Jupiter",
    "  Fleet",
    "  Research",
    "  Merchant",
    "  Settings",
    "  Save",
];

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let lines: Vec<Line> =
        std::iter::once(Line::styled("MENU", Style::default().fg(th.header).bold()))
            .chain(
                ENTRIES
                    .iter()
                    .map(|e| Line::styled(*e, Style::default().fg(th.sidebar))),
            )
            .collect();
    f.render_widget(Paragraph::new(lines).block(Block::bordered()), area);
}
