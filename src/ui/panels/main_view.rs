//! Panel MAIN VIEW: nama galaksi aktif + peta bintang kosmetik + legend. `06` §MAIN VIEW.
#![allow(dead_code)]

use crate::app::App;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

const MAP: [&str; 8] = [
    "        *    .",
    "   .        =======      *",
    "        ===#=======",
    r"   *    ===/|\===          .",
    r"        ===\|/===",
    "   .        =======      *",
    "      [Planet View]",
    "Legend: # Planet  . Star  * Nebula  = Route",
];

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let gname = app
        .state
        .galaxies
        .iter()
        .find(|g| g.id == app.state.active_galaxy)
        .map(|g| format!("{} (Lvl {})", g.name, g.level))
        .unwrap_or_else(|| "-".into());
    let mut lines = vec![
        Line::styled("MAIN VIEW", Style::default().fg(th.header).bold()),
        Line::styled(format!("* {gname} *"), Style::default().fg(th.focus).bold()),
        Line::raw(""),
    ];
    lines.extend(
        MAP.iter()
            .map(|m| Line::styled(*m, Style::default().fg(th.dim))),
    );
    f.render_widget(Paragraph::new(lines).block(Block::bordered()), area);
}
