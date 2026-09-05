//! Panel MAIN VIEW: nama galaksi aktif + animasi galaksi spiral (`14`) + legend. `06` §MAIN VIEW.
#![allow(dead_code)]

use crate::app::App;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Wrap};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let gname = app
        .state
        .galaxies
        .iter()
        .find(|g| g.id == app.state.active_galaxy)
        .map(|g| format!("{} (Lvl {})", g.name, g.level))
        .unwrap_or_else(|| "-".into());

    let outer = Block::bordered().border_style(Style::default().fg(th.dim));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(0),
        Constraint::Length(2),
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

    // M20.8 follow-up (galaxy_sim Phase 5): backdrop UTAMA skrng density-wave simulation
    // (GPU/CPU), MENGGANTIKAN M16/M17's spiral prosedural+PNG-statis-via-sixel. `active_galaxy`
    // (Fixed/Home ATAU Procedural/frontier) menentukan `(seed,is_anchor)` via `galaxy_sim_key`.
    // `force_procedural_galaxy` (M17.5) TETAP dihormati -- paksa `galaxy_anim`'s spiral lama
    // [fallback PALING DALAM, bukan lg mode utama] bila user genuinely minta.
    if app.force_procedural_galaxy {
        app.anim.render(f, rows[1], app.anim_secs, &th);
    } else {
        let (seed, is_anchor) = app
            .state
            .galaxies
            .iter()
            .find(|g| g.id == app.state.active_galaxy)
            .map(crate::ui::galaxy_sim_view::galaxy_sim_key)
            .unwrap_or((0, true));
        // `anim_secs` detik nyata -> Myr [skala LAMBAT ala reference's `MYR_PER_SEC=1.5`,
        // rotasi galaksi HARUS terasa perlahan-halus, bukan orbit "muter cepat" tak masuk akal].
        let t_myr = app.anim_secs * crate::ui::galaxy_sim_view::MYR_PER_SEC;
        app.galaxy_sim.render(f, rows[1], seed, is_anchor, t_myr);
    }
    // Overlay partikel (mis. engine exhaust saat Traveling); kosong saat Idle → no-op.
    app.particles.render(f, rows[1], &th);

    // M20.8 follow-up: legend LAMA ("★ Star ✦ Bright · Field") ditulis utk `galaxy_anim`'s
    // glyph-based rendering -- STALE utk backdrop simulation BARU (piksel truecolor asli,
    // tak py "simbol" utk dijelaskan). Kondisional: `force_procedural_galaxy` msh pakai
    // legend LAMA [genuinely masih glyph-based di mode itu], else legend BARU (nama backend).
    let legend_text = if app.force_procedural_galaxy {
        "Legend: ★ Star  ✦ Bright  · Field  (core/arm/edge by color)".to_string()
    } else {
        format!(
            "Density-wave simulation — {}",
            app.galaxy_sim.backend_name()
        )
    };
    f.render_widget(
        Paragraph::new(Line::styled(legend_text, Style::default().fg(th.dim)))
            .wrap(Wrap { trim: false }),
        rows[2],
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn rendered_text(app: &App) -> String {
        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term.draw(|f| render(f, app, f.area())).unwrap();
        crate::ui::buffer_to_text(term.backend().buffer()).join("\n")
    }

    /// M20.8 follow-up: legend HARUS genuinely BARU ("Density-wave simulation") saat backdrop
    /// simulation aktif -- teks LAMA ("★ Star...") ditulis utk `galaxy_anim`'s glyph-based
    /// rendering, STALE utk piksel truecolor asli, dites LANGSUNG bukan diasumsikan diperbaiki.
    #[test]
    fn legend_shows_density_wave_text_when_simulation_active() {
        let app = App::demo();
        let text = rendered_text(&app);
        assert!(
            text.contains("Density-wave simulation"),
            "legend HARUS genuinely tampilkan teks simulation baru: {text:?}"
        );
        assert!(
            !text.contains("★ Star"),
            "legend LAMA (glyph-based) TAK BOLEH tampil saat simulation aktif: {text:?}"
        );
    }

    /// `force_procedural_galaxy=true` HARUS genuinely PAKAI legend LAMA (fallback msh
    /// glyph-based, teks lama genuinely masih benar di jalur ITU).
    #[test]
    fn legend_shows_old_text_when_force_procedural() {
        let mut app = App::demo();
        app.force_procedural_galaxy = true;
        let text = rendered_text(&app);
        assert!(
            text.contains("★ Star"),
            "legend LAMA HARUS genuinely tampil saat force_procedural_galaxy: {text:?}"
        );
    }

    /// Render dgn galaxy_sim backdrop aktif HARUS genuinely gambar sesuatu (sel non-kosong)
    /// di area bidang bintang -- bukan cuma "tak panic".
    #[test]
    fn render_paints_galaxy_sim_backdrop() {
        let app = App::demo();
        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let buf = term.backend().buffer();
        let has_halfblock = (0..40u16).any(|y| (0..120u16).any(|x| buf[(x, y)].symbol() == "▀"));
        assert!(
            has_halfblock,
            "backdrop HARUS genuinely gambar half-block, bukan kosong"
        );
    }
}
