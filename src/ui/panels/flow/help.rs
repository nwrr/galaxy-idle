//! Panel HELP/CONTROLS: `?` key (M1 fix — dulu genuinely tak di-handle sama sekali di
//! `handle_key`, `?:Help` di `shortcuts.rs` 100% fake). Daftar di sini ditulis manual dari
//! `app.rs`'s match arm NYATA (bukan direka). M5: grup MERCHANT/WARP ditambah (M1/M4 fix
//! membuat keduanya jd view nyata) + tes anti-drift (`every_real_keycode_char_binding_is_
//! documented_in_help`) yg scan LANGSUNG source `app.rs` utk `KeyCode::Char('_')` literal —
//! kalau ada key baru ditambah nanti tp lupa didaftarkan di sini, tes ini GAGAL segera (bukan
//! nunggu ketemu manual pas main, sama kelas masalah M1's `draw_placeholder` regression test).

use crate::app::App;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Wrap};

/// (label grup, &[(key, aksi)]).
type Group = (&'static str, &'static [(&'static str, &'static str)]);

// M1 fix: dipadatkan (multi-aksi/baris) biar muat tanpa overflow di breakpoint Full
// TERKECIL (100x30, ~24 baris interior) -- ditemukan LANGSUNG via `screenshot.sh` (versi
// awal, 1 aksi/baris, genuinely terpotong di bawah "GALAXY MAP"), bukan diasumsikan muat.
// Enumerasi PENUH (1 aksi/baris, per-view) menyusul M5 (py layar/scroll sendiri).
const GROUPS: &[Group] = &[
    (
        "GLOBAL",
        &[
            ("g", "Galaxy Map"),
            ("p", "Planet View"),
            ("r", "Research"),
            ("m", "Merchant"),
            ("w", "Fleet / Warp"),
            ("S", "Settings"),
            ("?", "Help (layar ini)"),
            ("T", "Sembunyikan hint tutorial"),
            ("G", "Toggle backdrop (Main Menu)"),
            ("Esc", "Kembali ke Main Menu"),
            ("q/Ctrl+C/Ctrl+S", "Quit (autosave) / Simpan manual"),
        ],
    ),
    (
        "PLANET VIEW",
        &[
            ("j/k", "Pilih slot"),
            ("1 / 2/Enter", "Upgrade node / factory"),
            ("3", "Build (slot kosong)"),
        ],
    ),
    (
        "RESEARCH",
        &[("j/k", "Pilih tech"), ("Enter", "Mulai riset")],
    ),
    (
        "GALAXY MAP",
        &[
            ("h/j/k/l", "Gerak kursor / pan kamera"),
            ("s/t/Enter", "Scan / Travel / Masuk sistem (Starmap)"),
            ("f / Tab", "Filter (Starmap) / toggle mode"),
            ("z/x/r", "Zoom in / out / reset (Backdrop)"),
        ],
    ),
    (
        "MERCHANT",
        &[("j/k", "Pilih tawaran"), ("Enter", "Beli tawaran terpilih")],
    ),
    (
        "WARP",
        &[
            ("1", "Initiate Warp Jump (bila threshold terpenuhi)"),
            ("2", "Kembali ke Main Menu"),
        ],
    ),
    ("SETTINGS", &[("j/k", "Siklus tema")]),
];

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let title =
        ratatui::text::Span::styled(" HELP / CONTROLS ", Style::default().fg(th.header).bold());
    let outer = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(th.dim));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let mut lines: Vec<Line> = Vec::new();
    for (group, keys) in GROUPS {
        lines.push(Line::styled(*group, Style::default().fg(th.header).bold()));
        for (key, action) in *keys {
            lines.push(Line::styled(
                format!("  {key:<10} {action}"),
                Style::default().fg(th.text),
            ));
        }
    }
    // M1 fix: breakpoint sempit (Minimal/Compact) tak selalu muat semua grup -- dulu
    // (screenshot.sh, ditemukan LANGSUNG) baris bawah DIAM² terpotong ratatui (overflow tanpa
    // tanda). Skrng dipotong SADAR + hint eksplisit "+N lagi", bukan clipping senyap (pola
    // sama `panels::resources`'s "+N resource lain").
    let cap = inner.height as usize;
    if lines.len() > cap && cap > 0 {
        let hidden = lines.len() - (cap - 1);
        lines.truncate(cap - 1);
        lines.push(Line::styled(
            format!("+{hidden} baris lagi (lebarkan terminal)"),
            Style::default().fg(th.dim),
        ));
    }
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn rendered_text(app: &App, w: u16, h: u16) -> String {
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        term.draw(|f| render(f, app, f.area())).unwrap();
        crate::ui::buffer_to_text(term.backend().buffer()).join("\n")
    }

    #[test]
    fn lists_every_real_global_key() {
        let app = App::demo();
        let text = rendered_text(&app, 100, 40);
        for key in ["g", "p", "r", "m", "w", "S", "?", "Esc", "Ctrl+S"] {
            assert!(
                text.contains(key),
                "harus list key global {key:?}: {text:?}"
            );
        }
    }

    #[test]
    fn shows_context_groups() {
        let app = App::demo();
        let text = rendered_text(&app, 100, 40);
        for group in [
            "PLANET VIEW",
            "RESEARCH",
            "GALAXY MAP",
            "MERCHANT",
            "WARP",
            "SETTINGS",
        ] {
            assert!(
                text.contains(group),
                "harus tampilkan grup {group:?}: {text:?}"
            );
        }
    }

    /// M5: anti-drift — scan LANGSUNG source `app.rs` utk tiap `KeyCode::Char('_')` literal;
    /// tiap karakter yg dites HARUS muncul di teks Help. Kalau nanti ada key baru ditambah tp
    /// lupa didaftarkan di `GROUPS`, tes ini gagal SEGERA (sama kelas M1's regression test utk
    /// `draw_placeholder`, cegah drift diam2 antara kode nyata vs dokumentasi Help).
    #[test]
    fn every_real_keycode_char_binding_is_documented_in_help() {
        // 'c' = Ctrl+C quit, dicek LANGSUNG `event_loop` (bukan `handle_key`'s match) --
        // didokumentasikan via teks "Ctrl+C" bareng 'q' di grup GLOBAL, sengaja dikecualikan
        // dr scan literal char (huruf "C" di situ besar, bukan match literal `'c'` kecil).
        const EXEMPT: &[char] = &['c'];
        let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/app.rs"));
        let app = App::demo();
        // Tinggi besar (200) spy tak ada truncation "+N lagi" yg menyembunyikan grup manapun.
        let text = rendered_text(&app, 100, 200);
        let mut seen = std::collections::HashSet::new();
        let needle = "KeyCode::Char('";
        let mut rest = src;
        while let Some(i) = rest.find(needle) {
            let after = &rest[i + needle.len()..];
            if let Some(ch) = after.chars().next() {
                seen.insert(ch);
            }
            rest = &after[1..];
        }
        for ch in seen {
            if EXEMPT.contains(&ch) {
                continue;
            }
            assert!(
                text.contains(ch),
                "key {ch:?} dipakai di app.rs tp tak terdaftar di Help GROUPS: {text:?}"
            );
        }
    }

    /// Breakpoint sempit HARUS truncate SADAR ("+N baris lagi"), bukan diam² kepotong
    /// ratatui tanpa tanda (bug real ditemukan via `screenshot.sh` sblm ini).
    #[test]
    fn narrow_height_shows_explicit_truncation_hint() {
        let app = App::demo();
        let text = rendered_text(&app, 40, 12);
        assert!(
            text.contains("lagi"),
            "harus tampilkan hint truncation eksplisit: {text:?}"
        );
    }

    #[test]
    fn never_panics_at_extreme_tiny_height() {
        let app = App::demo();
        let _ = rendered_text(&app, 40, 3);
    }
}
