//! Panel OPTIONS (sidebar kanan): navigasi berkelompok, penanda fokus `►`+reverse. `06` §MENU.
//! M13.4: dikelompokkan (ref `riftborne/42UN3F.png` §OPTIONS — COLONY/EMPIRE/MILITARY/INTEL/
//! SYSTEM). Game ini tak py fitur military/intel terpisah — grup diadaptasi ke item NYATA yg
//! ada: NAVIGATION (view galaksi/planet/armada), PROGRESS (riset/dagang), SYSTEM (pengaturan/
//! simpan). M13.6: penanda `►` SKRNG SINKRON `app.view` sungguhan (dulu statis selalu di
//! "Galaxy Map", terlepas view aktif — bug nyata, bukan cuma "belum diimplementasi": dicek
//! `app.rs` `handle_key` — `g/p/r/m/w` set `self.view` ke id string beda, tp menu tak pernah
//! baca balik). M1 fix: "Settings" SKRNG py view id nyata (`app.rs`'s `S` key, view NYATA
//! `panels::flow::settings`). M1#5: "Save" msh `view_id` `None` SENGAJA — bukan transisi VIEW
//! tp AKSI instan (`App::manual_save`, sudah nyata dipakai `Ctrl+S`/quit/autosave sejak M1#1);
//! label diubah "Save (Ctrl+S)" biar JUJUR (nunjuk cara nyata memicu, bukan entry "mati" yg
//! kelihatan bisa diklik tp tak ke mana-mana — panel ini genuinely tak py cursor/Enter-select
//! sendiri, SEMUA entry lain jg cuma cermin `app.view` via hotkey global, bukan navigasi klik).
#![allow(dead_code)]

use crate::app::App;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

/// (label tampil, id view utk dicocokkan `app.view` — `None` = tak pernah bisa aktif/dipilih).
type Entry = (&'static str, Option<&'static str>);

const GROUPS: [(&str, &[Entry]); 3] = [
    (
        "NAVIGATION",
        &[
            ("Galaxy Map", Some("galaxy_map")),
            ("Planets", Some("planet_view")),
            (" * Earth", None),
            (" o Mars", None),
            (" o Jupiter", None),
            ("Fleet", Some("warp")),
        ],
    ),
    (
        "PROGRESS",
        &[
            ("Research", Some("research")),
            ("Merchant", Some("merchant")),
        ],
    ),
    (
        "SYSTEM",
        &[("Settings", Some("settings")), ("Save (Ctrl+S)", None)],
    ),
];

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let mut lines = vec![Line::styled(
        "OPTIONS",
        Style::default().fg(th.header).bold(),
    )];
    for (group, entries) in GROUPS {
        lines.push(Line::styled(group, Style::default().fg(th.header)));
        for (label, view_id) in entries {
            let active = *view_id == Some(app.view.as_str());
            let (prefix, style) = if active {
                (
                    "\u{25ba} ",
                    Style::default()
                        .fg(th.sidebar)
                        .add_modifier(Modifier::REVERSED),
                )
            } else {
                ("  ", Style::default().fg(th.sidebar))
            };
            lines.push(Line::styled(format!("{prefix}{label}"), style));
        }
    }
    let block = Block::bordered().border_style(Style::default().fg(th.dim));
    f.render_widget(Paragraph::new(lines).block(block), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    /// M13.6: `scripts/screenshot.sh <view>` TAK BISA verifikasi ini — CLI arg `view` cuma
    /// pilih panel MAIN via parameter langsung (`draw_view`), terpisah dari `app.view` (state
    /// `App::demo()` selalu "main_menu" tetap, dicek `app.rs`). Jadi highlight per-`app.view`
    /// diuji lgs lewat `Buffer` (bukan Read screenshot) — cek `Modifier::REVERSED` nempel PAS
    /// di baris yg cocok, TAK di baris lain (bukti bukan reversed serampangan/selalu-nyala).
    fn reversed_row(app: &App) -> Option<usize> {
        let mut term = Terminal::new(TestBackend::new(24, 20)).unwrap();
        term.draw(|f| render(f, app, f.area())).unwrap();
        let buf = term.backend().buffer().clone();
        (0..buf.area().height)
            .find(|&y| {
                (0..buf.area().width).any(|x| buf[(x, y)].modifier.contains(Modifier::REVERSED))
            })
            .map(|y| y as usize)
    }

    #[test]
    fn highlight_follows_active_view() {
        let mut app = App::demo();
        app.view = "galaxy_map".into();
        let galaxy_row = reversed_row(&app).expect("galaxy_map harus ada baris reversed");

        app.view = "research".into();
        let research_row = reversed_row(&app).expect("research harus ada baris reversed");

        assert_ne!(
            galaxy_row, research_row,
            "baris reversed harus PINDAH ikut app.view, bukan statis"
        );
    }

    #[test]
    fn no_highlight_for_view_without_menu_entry() {
        let mut app = App::demo();
        app.view = "main_menu".into(); // tak match view_id manapun di GROUPS
        assert_eq!(
            reversed_row(&app),
            None,
            "main_menu tak py entry menu — jujur tak ada yg reversed"
        );
    }

    #[test]
    fn save_never_highlights() {
        // "Save" msh py view_id `None` sengaja (bukan transisi view, item terpisah M1#5) —
        // pastikan tak ada string view lain yg diam2 membuatnya reversed.
        let mut app = App::demo();
        for v in ["save", "Settings", "Save"] {
            app.view = v.into();
            assert_eq!(
                reversed_row(&app),
                None,
                "view {v:?} seharusnya tak match apa pun"
            );
        }
    }

    /// M1#5: label HARUS genuinely nunjuk cara nyata memicu Save (`Ctrl+S`) -- bukan cuma
    /// kata "Save" polos yg keliatan spt entry mati/dead (sama kelas keluhan "ui tak jelas").
    #[test]
    fn save_label_shows_real_trigger_hint() {
        let mut term = Terminal::new(TestBackend::new(24, 20)).unwrap();
        let app = App::demo();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let text = crate::ui::buffer_to_text(term.backend().buffer()).join("\n");
        assert!(
            text.contains("Ctrl+S"),
            "label Save harus sebut trigger nyata: {text:?}"
        );
    }

    #[test]
    fn settings_view_highlights_its_own_entry() {
        // M1 fix: "settings" SKRNG view NYATA (dulu dites justru "tak boleh reversed" --
        // ekspektasi lama sudah stale sejak `panels::flow::settings` dikawinkan ke sini).
        let mut app = App::demo();
        app.view = "settings".into();
        assert!(
            reversed_row(&app).is_some(),
            "Settings entry harus reversed saat app.view == \"settings\""
        );
    }
}
