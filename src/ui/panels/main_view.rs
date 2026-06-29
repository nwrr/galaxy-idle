//! Panel MAIN VIEW: nama galaksi aktif + animasi galaksi spiral (`14`) + legend. `06` §MAIN VIEW.
#![allow(dead_code)]

use crate::app::App;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let gname = app
        .state
        .galaxies
        .iter()
        .find(|g| g.id == app.state.active_galaxy)
        .map(|g| format!("{} (Lvl {})", g.name, g.level))
        .unwrap_or_else(|| "-".into());

    let outer = Block::bordered();
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner);

    // Header: judul + nama galaksi aktif (tetap terbaca di atas bidang bintang).
    f.render_widget(
        Paragraph::new(vec![
            Line::styled("MAIN VIEW", Style::default().fg(th.header).bold()),
            Line::styled(format!("* {gname} *"), Style::default().fg(th.focus).bold()),
        ]),
        rows[0],
    );

    // Bidang bintang spiral (kosmetik, dianimasikan dari `app.anim_secs`).
    app.anim.render(f, rows[1], app.anim_secs, &th);
    // Overlay partikel (mis. engine exhaust saat Traveling); kosong saat Idle → no-op.
    app.particles.render(f, rows[1], &th);

    f.render_widget(
        Paragraph::new(Line::styled(
            "Legend: ★ Star  ✦ Bright  · Field  (core/arm/edge by color)",
            Style::default().fg(th.dim),
        )),
        rows[2],
    );
}
