//! Splash (`view="splash"`, M2 item 1): layar intro chrome-less full-bleed. Tekan tombol
//! apa saja -> Title (`app.rs`'s `handle_key`, guard SEBELUM match global g/p/r/m/w spy tak
//! diam2 lompat ke game view sblm New Game/Continue dipilih -- lihat M2 item 2).
use crate::app::App;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

/// M4: backdrop reuse `galaxy_sim` (density-wave sim, SAMA `main_view.rs`'s Fixed/Home galaxy)
/// -- bukan bikin animasi baru, first-impression premium via engine yg SUDAH ADA. Digambar
/// SEBELUM teks (baris judul/hint di-render di atas, timpa cuma baris masing2 -- baris lain
/// tetap kelihatan bidang bintang, pola SAMA `main_view.rs`'s header-di-atas-backdrop).
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
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(area);
    let logo = Paragraph::new(vec![Line::styled(
        "G A L A X Y   I D L E",
        Style::default().fg(th.header).bold(),
    )])
    .alignment(Alignment::Center);
    f.render_widget(logo, rows[1]);
    let hint = Paragraph::new(Line::styled(
        "tekan tombol apa saja untuk lanjut",
        Style::default().fg(th.dim),
    ))
    .alignment(Alignment::Center);
    f.render_widget(hint, rows[2]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn rendered(w: u16, h: u16) -> String {
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let app = App::demo();
        term.draw(|f| render(f, &app)).unwrap();
        crate::ui::buffer_to_text(term.backend().buffer()).join("\n")
    }

    #[test]
    fn shows_title_and_hint() {
        let text = rendered(80, 24);
        assert!(text.contains("GALAXY") || text.contains("G A L A X Y"));
        assert!(text.contains("tombol apa saja"));
    }

    #[test]
    fn never_panics_at_tiny_size() {
        let text = rendered(10, 3);
        let _ = text;
    }
}
