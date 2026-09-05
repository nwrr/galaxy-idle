//! Title (`view="title"`, M2 item 2): chrome-less, menu New Game/Continue/Settings/Help/Quit.
//! Continue hanya enabled bila save nyata ada (`crate::save::save_path()` exists di disk) --
//! bukan selalu enabled spt tombol mati, dan bukan selalu disabled (baru main pertama kali).
use crate::app::App;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

pub const ENTRIES: [&str; 5] = ["New Game", "Continue", "Settings", "Help", "Quit"];

pub fn save_exists() -> bool {
    crate::save::save_path().exists()
}

/// M4: SAMA backdrop `galaxy_sim` spt `flow::splash` (lihat komentarnya) — konsisten first-
/// impression premium di Splash DAN Title, bukan cuma satu layar.
fn render_backdrop(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let (seed, is_anchor) = app
        .state
        .galaxies
        .first()
        .map(crate::ui::galaxy_sim_view::galaxy_sim_key)
        .unwrap_or((0, true));
    let t_myr = app.anim_secs * crate::ui::galaxy_sim_view::MYR_PER_SEC;
    app.galaxy_sim.render(f, area, seed, is_anchor, t_myr);
}

pub fn render(f: &mut Frame, app: &App) {
    let th = theme::theme(app.state.settings.theme);
    let area = f.area();
    render_backdrop(f, app, area);
    let has_save = save_exists();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(ENTRIES.len() as u16),
            Constraint::Min(0),
        ])
        .split(area);
    let logo = Paragraph::new(Line::styled(
        "G A L A X Y   I D L E",
        Style::default().fg(th.header).bold(),
    ))
    .alignment(Alignment::Center);
    f.render_widget(logo, rows[1]);

    let lines: Vec<Line> = ENTRIES
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let disabled = *label == "Continue" && !has_save;
            let active = i == app.title_sel;
            let text = if disabled {
                format!("  {label} (tak ada save)")
            } else if active {
                format!("\u{25ba} {label}")
            } else {
                format!("  {label}")
            };
            let mut style = Style::default().fg(th.basic);
            if disabled {
                style = Style::default().fg(th.dim);
            } else if active {
                style = Style::default()
                    .fg(th.header)
                    .add_modifier(Modifier::REVERSED);
            }
            Line::styled(text, style)
        })
        .collect();
    let menu = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(menu, rows[3]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn rendered(app: &App, w: u16, h: u16) -> String {
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        term.draw(|f| render(f, app)).unwrap();
        crate::ui::buffer_to_text(term.backend().buffer()).join("\n")
    }

    #[test]
    fn lists_all_five_entries() {
        let app = App::demo();
        let text = rendered(&app, 80, 24);
        for label in ENTRIES {
            assert!(text.contains(label), "entry {label:?} hilang: {text:?}");
        }
    }

    #[test]
    fn marker_follows_title_sel() {
        let mut app = App::demo();
        app.title_sel = 2;
        let text = rendered(&app, 80, 24);
        let line = text.lines().find(|l| l.contains("Settings")).unwrap();
        assert!(
            line.contains('\u{25ba}'),
            "marker harus di baris Settings: {line:?}"
        );
    }
}
