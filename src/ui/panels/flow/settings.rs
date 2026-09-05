//! Panel SETTINGS: ganti tema LIVE (`06-ui.md` §Keybindings, `s` = Settings). M1 fix: dulu
//! `menu.rs`'s entri "Settings" py view id `None` (mati total) — sekarang view NYATA, tema
//! ganti seketika (dibaca `ui::theme::theme` di semua panel lain, bukan sistem preview baru).

use crate::app::App;
use crate::game::state::ThemeChoice;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

/// Urutan siklus tema (dipakai jg oleh `App::settings_keys`, harus SINKRON).
pub const THEME_ORDER: [ThemeChoice; 3] = [
    ThemeChoice::Default,
    ThemeChoice::HighContrast,
    ThemeChoice::Mono,
];

fn theme_label(t: ThemeChoice) -> &'static str {
    match t {
        ThemeChoice::Default => "Default",
        ThemeChoice::HighContrast => "High Contrast",
        ThemeChoice::Mono => "Mono",
    }
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let title = ratatui::text::Span::styled(" SETTINGS ", Style::default().fg(th.header).bold());
    let outer = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(th.dim));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner);

    f.render_widget(
        Paragraph::new(Line::styled(
            "Theme:",
            Style::default().fg(th.header).bold(),
        )),
        rows[0],
    );

    let lines: Vec<Line> = THEME_ORDER
        .iter()
        .map(|t| {
            let active = *t == app.state.settings.theme;
            let label = format!(
                "{} {}",
                if active { "\u{25ba}" } else { " " },
                theme_label(*t)
            );
            let style = if active {
                Style::default()
                    .fg(th.focus)
                    .add_modifier(Modifier::REVERSED)
            } else {
                Style::default().fg(th.text)
            };
            Line::styled(label, style)
        })
        .collect();
    f.render_widget(Paragraph::new(lines), rows[1]);

    f.render_widget(
        Paragraph::new(Line::styled(
            "[j/k] ganti tema  [Esc] kembali",
            Style::default().fg(th.dim),
        )),
        rows[2],
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn rendered_text(app: &App) -> String {
        let mut term = Terminal::new(TestBackend::new(80, 24)).unwrap();
        term.draw(|f| render(f, app, f.area())).unwrap();
        crate::ui::buffer_to_text(term.backend().buffer()).join("\n")
    }

    #[test]
    fn shows_active_theme_marker() {
        let app = App::demo();
        let text = rendered_text(&app);
        assert!(
            text.contains("Default"),
            "harus tampilkan tema Default: {text:?}"
        );
        assert!(
            text.contains("\u{25ba}"),
            "harus tandai tema aktif: {text:?}"
        );
    }

    #[test]
    fn shows_all_three_themes() {
        let app = App::demo();
        let text = rendered_text(&app);
        for label in ["Default", "High Contrast", "Mono"] {
            assert!(text.contains(label), "harus list tema {label}: {text:?}");
        }
    }
}
