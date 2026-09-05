//! Panel Building/Training/Assets (M13.2/M13.3, ref `riftborne/42UN3F.png` kolom kiri). Satu
//! file krn ketiganya kecil+terkait erat (sama kolom, sama constraint mekanik) — `panels/`
//! sudah pas 10 file (batas CONVENTIONS §1) sejak M13.2, jadi file BARU per-fungsi kecil
//! dihindari (subfolder split lbh masuk akal kalau nanti ada panel BESAR baru, bukan skrng).
//!
//! Riftborne py mekanik build-queue & training-queue BERWAKTU (timer countdown, banyak item
//! antre). Game ini TAK py mekanik itu — `build_factory`/`upgrade_factory` INSTAN (`game::
//! actions`), tak ada queue/timer tersimpan di state. `jangan ubah mekanik` melarang reka2
//! sistem timer baru cuma demi kemiripan visual placeholder (§M12.2). Jadi:
//! - **Building**: status "Idle" (jujur, bukan bug) + ringkasan slot factory planet aktif
//!   (data REAL: filled/total, dari `Planet::factory_slots`) — bukan queue, cuma status apa
//!   adanya.
//! - **Training**: dipetakan ke mekanik NYATA yg genuinely berwaktu di game ini — riset aktif
//!   (`ResearchState::active`, py `elapsed_secs`/`time_secs` sungguhan) — padanan plg dekat
//!   "sedang mengerjakan sesuatu dgn timer", bukan training ship (yg tak py mekanik).
//! - **Assets** (M13.3): Buildings (factory terisi + level, `Planet::factory_slots`) + Ship
//!   (level tiap subsistem, `GameState::ship`) — SEMUA data REAL, tak ada reka-reka (beda dari
//!   Building/Training yg genuinely tak py data queue — Assets justru py banyak data nyata).
#![allow(dead_code)]

use crate::app::App;
use crate::game::research;
use crate::game::state::Planet;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

/// Planet aktif (unlocked pertama galaksi aktif) — sama pola `panels::planet::active_planet`
/// (diduplikasi, cuma butuh 1 planet di sini, hindari coupling modul demi ~5 baris).
fn active_planet(app: &App) -> Option<&Planet> {
    app.state
        .galaxies
        .iter()
        .find(|g| g.id == app.state.active_galaxy)
        .and_then(|g| g.planets.iter().find(|p| p.unlocked))
}

pub fn render_building(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let text = match active_planet(app) {
        Some(p) => {
            let filled = p.factory_slots.iter().filter(|s| s.is_some()).count();
            format!("Idle ({filled}/{} slot)", p.factory_slots.len())
        }
        None => "Idle".to_string(),
    };
    f.render_widget(
        Paragraph::new(Line::styled(text, Style::default().fg(th.dim))),
        area,
    );
}

/// Bar `[###...]` lebar 6 (ringkas, slot sempit) dari fraksi 0..1.
fn mini_bar(frac: f64) -> String {
    let w = 6usize;
    let filled = (frac.clamp(0.0, 1.0) * w as f64).round() as usize;
    let mut s = String::with_capacity(w + 2);
    s.push('[');
    for i in 0..w {
        s.push(if i < filled { '#' } else { '.' });
    }
    s.push(']');
    s
}

pub fn render_training(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let lines: Vec<Line> = match app.state.research.active.as_ref() {
        Some(active) => match research::tech_def(&app.content, &active.tech_id) {
            Some(t) => {
                let time_frac = if t.time_secs > 0.0 {
                    active.elapsed_secs / t.time_secs
                } else {
                    1.0
                };
                let left = (t.time_secs - active.elapsed_secs).max(0.0);
                vec![
                    Line::styled(t.id.clone(), Style::default().fg(th.good).bold()),
                    Line::styled(
                        format!("{} {}s left", mini_bar(time_frac), left.round() as i64),
                        Style::default().fg(th.good),
                    ),
                ]
            }
            None => vec![Line::styled(
                active.tech_id.clone(),
                Style::default().fg(th.good),
            )],
        },
        None => vec![Line::styled("Idle", Style::default().fg(th.dim))],
    };
    f.render_widget(Paragraph::new(lines), area);
}

/// Maks baris building ditampilkan sblm dipotong "+N lain" — cegah bug kelas-sama M13.1
/// (baris tersembunyi diam² lampaui `area`, ditemukan lwt screenshot bukan diasumsikan aman).
const MAX_BUILDING_LINES: usize = 3;

/// Assets (M13.3): Buildings (factory terisi + level, planet aktif) + Ship (level tiap
/// subsistem, 1 baris ringkas), SEMUA data REAL. Judul "Buildings"/"Ship" TAK diulang di
/// konten (border title `titled_box` "Assets" sudah cukup) — hemat baris, budget kolom kiri
/// SANGAT terbatas (body 120×40 cuma ~35 baris utk Resources(22)+Building(3)+Training(4)+
/// Assets — ditemukan REAL overflow saat pertama coba 9 baris, total lampaui 35; fix jadi
/// ringkas genuinely muat `Length(6)`).
pub fn render_assets(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let mut lines = Vec::new();
    if let Some(p) = active_planet(app) {
        let built: Vec<_> = p.factory_slots.iter().flatten().collect();
        for fac in built.iter().take(MAX_BUILDING_LINES) {
            let name = &app.content.buildings.get(fac.building.0).name;
            lines.push(Line::styled(
                format!("{name} L{}", fac.level),
                Style::default().fg(th.basic),
            ));
        }
        if built.len() > MAX_BUILDING_LINES {
            lines.push(Line::styled(
                format!("+{} lain", built.len() - MAX_BUILDING_LINES),
                Style::default().fg(th.dim),
            ));
        }
    }
    let ship = &app.state.ship;
    lines.push(Line::styled(
        format!(
            "Ship WT{} E{} C{} S{}",
            ship.warp_tier, ship.engine, ship.cargo, ship.scanner
        ),
        Style::default().fg(th.good),
    ));
    f.render_widget(Paragraph::new(lines), area);
}
