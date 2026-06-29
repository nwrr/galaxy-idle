//! View GALAXY MAP: daftar planet sektor frontier (galaksi level tertinggi). ProcGen → preview
//! per index dari seed (nama/biome/jarak/travel/status); planet visited pakai versi materialisasi.
//! `06-ui.md` §Galaxy Map, `16-procgen-logic.md` §Materialisasi. Read-only (scan/send di `app`).
#![allow(dead_code)]

use crate::app::App;
use crate::game::ship::travel_secs;
use crate::game::state::{Biome, Galaxy, GalaxyKind, Planet, UnlockReq};
use crate::game::world::procgen::{generate_planet, planet_count};
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

/// Galaksi sektor yang dieksplor = level tertinggi (frontier); Milky Way (Lvl 0) = anchor.
pub fn frontier(app: &App) -> Option<&Galaxy> {
    app.state.galaxies.iter().max_by_key(|g| g.level)
}

fn biome_label(b: Biome) -> &'static str {
    match b {
        Biome::IronWorld => "Iron",
        Biome::OceanPlanet => "Ocean",
        Biome::GasGiant => "GasGiant",
        Biome::DeadWorld => "Dead",
        Biome::CrystalWorld => "Crystal",
        Biome::Terran => "Terran",
        Biome::AsteroidBelt => "Asteroid",
        Biome::IceWorld => "Ice",
        Biome::LavaWorld => "Lava",
    }
}

/// Detik → "Xm" / "XhYm" ringkas untuk kolom travel.
fn fmt_eta(secs: f64) -> String {
    let m = (secs / 60.0).round() as u64;
    if m >= 60 {
        format!("{}h{:02}m", m / 60, m % 60)
    } else {
        format!("{m}m")
    }
}

fn status_label(p: &Planet, materialized: bool) -> String {
    if p.unlocked {
        "Active".into()
    } else if materialized {
        "Scanned".into()
    } else {
        match p.unlock_req {
            UnlockReq::WarpTier(t) => format!("Locked WT{t}"),
            _ => "Locked".into(),
        }
    }
}

/// Satu baris planet: simbol, nama, biome, jarak, travel time, status.
fn planet_row(idx: u32, p: &Planet, engine: u32, materialized: bool, sel: bool) -> String {
    let cursor = if sel { '>' } else { ' ' };
    let glyph = if p.unlocked { '■' } else { '·' };
    format!(
        "{cursor}{glyph} {:>2} {:<12} {:<9} d={:>4.1} t={:>5} {}",
        idx,
        truncate(&p.name, 12),
        biome_label(p.biome),
        p.distance,
        fmt_eta(travel_secs(p.distance, engine)),
        status_label(p, materialized),
    )
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n.saturating_sub(1)).collect::<String>() + "…"
    }
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let engine = app.state.ship.engine;
    let Some(g) = frontier(app) else {
        f.render_widget(
            Paragraph::new("Tidak ada galaksi.").block(Block::bordered()),
            area,
        );
        return;
    };

    let mut lines: Vec<Line> = vec![
        Line::styled(
            format!("GALAXY MAP — {} (Lvl {})", g.name, g.level),
            Style::default().fg(th.focus).bold(),
        ),
        Line::styled(
            "Legend: ■ Planet · ◆ Star · ═ Route",
            Style::default().fg(th.dim),
        ),
        Line::raw(""),
    ];

    // Banyaknya baris: ProcGen → planet_count(seed); Fixed → jumlah planet termuat.
    let count = match &g.kind {
        GalaxyKind::Procedural { seed, .. } => planet_count(*seed),
        GalaxyKind::Fixed => g.planets.len() as u32,
    };
    let max_rows = (area.height.saturating_sub(7)) as u32; // sisakan header/legend/footer
    let shown = count.min(max_rows.max(1));

    for idx in 0..shown {
        let (row, color) = match &g.kind {
            GalaxyKind::Procedural { seed, visited } if !visited.contains(&idx) => {
                let p = generate_planet(*seed, idx, g.level, &app.content);
                (
                    planet_row(idx, &p, engine, false, idx == app.sel as u32),
                    th.neutral,
                )
            }
            _ => {
                // Fixed, atau ProcGen index sudah dimaterialisasi → pakai planet tersimpan.
                let p = g.planets.get(idx as usize);
                match p {
                    Some(p) => {
                        let c = if p.unlocked { th.good } else { th.advanced };
                        (planet_row(idx, p, engine, true, idx == app.sel as u32), c)
                    }
                    None => (format!(" · {idx:>2} (unscanned)"), th.dim),
                }
            }
        };
        let style = if idx == app.sel as u32 {
            Style::default().fg(color).bold()
        } else {
            Style::default().fg(color)
        };
        lines.push(Line::styled(row, style));
    }
    if count > shown {
        lines.push(Line::styled(
            format!("  …+{} more", count - shown),
            Style::default().fg(th.dim),
        ));
    }

    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "[s] SCAN/SEND ship to selected   [j/k] Select   [Esc] Back",
        Style::default().fg(th.neutral),
    ));

    f.render_widget(Paragraph::new(lines).block(Block::bordered()), area);
}
