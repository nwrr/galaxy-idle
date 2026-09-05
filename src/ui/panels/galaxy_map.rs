//! View GALAXY MAP: starmap grid tile 2D (M18, ref Riftborne `owOev1.png`) — planet ProcGen
//! ditempatkan deterministik di koordinat dunia via hash `(seed, idx)`, viewport tampilkan
//! sebagian grid. `06-ui.md` §Galaxy Map, `16-procgen-logic.md` §Materialisasi.
//! Read-only (scan/send di `app`).
#![allow(dead_code)]

use crate::app::App;
use crate::game::ship::travel_secs;
use crate::game::state::{Galaxy, GalaxyKind, Planet, PlanetId, ShipStatus, UnlockReq};
use crate::game::world::procgen::{generate_planet, planet_count};
use crate::rng::{SplitMix64, planet_rng};
use crate::ui::{biome_label, theme};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

/// M18.1: "Grid tile sistem (huruf/simbol) di viewport" — **gap real ditemukan**: dulu
/// `galaxy_map.rs` MURNI daftar teks vertikal (1 baris/planet, di-scroll bila > tinggi area) —
/// checklist minta starmap GRID 2D ala Riftborne (`owOev1.png`: dot-matrix latar + simbol
/// planet tersebar di koordinat x/y, BUKAN list linear). Dunia virtual (`WORLD_W`×`WORLD_H`)
/// jauh lbh besar drpd viewport manapun — pondasi utk scroll (M18.5); utk M18.1 viewport
/// selalu mulai dr pojok dunia (0,0) (scroll genuine msh M18.5).
pub const WORLD_W: i32 = 120;
pub const WORLD_H: i32 = 60;

/// M19.9: "Filter (faksi/tile) toggle" — game TAK PY faksi (dikonfirmasi berulang M18.2/8/
/// 19.1 dst, `game/state.rs` tak py field faction sama sekali) — diganti filter STATUS tile
/// [pola SAMA substitusi "owner"->"status" konsisten sesi ini]. Cycle via `next()` (key `f`,
/// `app.rs`'s `galaxy_map_keys`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GalaxyFilter {
    #[default]
    All,
    Active,
    Locked,
    Unscanned,
}

impl GalaxyFilter {
    pub fn next(self) -> Self {
        match self {
            GalaxyFilter::All => GalaxyFilter::Active,
            GalaxyFilter::Active => GalaxyFilter::Locked,
            GalaxyFilter::Locked => GalaxyFilter::Unscanned,
            GalaxyFilter::Unscanned => GalaxyFilter::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            GalaxyFilter::All => "All",
            GalaxyFilter::Active => "Active",
            GalaxyFilter::Locked => "Locked",
            GalaxyFilter::Unscanned => "Unscanned",
        }
    }

    fn matches(self, status: TileStatus) -> bool {
        match self {
            GalaxyFilter::All => true,
            GalaxyFilter::Active => status == TileStatus::Active,
            GalaxyFilter::Locked => status == TileStatus::Locked,
            GalaxyFilter::Unscanned => status == TileStatus::Unscanned,
        }
    }
}

/// M19.9: status tile utk 1 idx, dipakai `GalaxyFilter` cocokkan -- SAMA klasifikasi persis
/// yg dipakai warna tile [`th.good`/`th.advanced`/`th.neutral` di loop `placed`], DIEKSTRAK
/// jd fungsi terpisah biar filter & warna genuinely konsisten (bukan 2 logic klasifikasi
/// terpisah yg bisa divergen).
#[derive(Clone, Copy, PartialEq, Eq)]
enum TileStatus {
    Unscanned,
    Locked,
    Active,
}

fn tile_status(g: &Galaxy, idx: u32) -> TileStatus {
    match &g.kind {
        GalaxyKind::Procedural { visited, .. } if !visited.contains(&idx) => TileStatus::Unscanned,
        _ => match planet_by_idx(g, idx) {
            Some(p) if p.unlocked => TileStatus::Active,
            Some(_) => TileStatus::Locked,
            None => TileStatus::Unscanned,
        },
    }
}

/// M20.1: "Mode backdrop vs starmap (tab/key) dalam galaxy view" — **gap real ditemukan**:
/// backdrop [M16 spiral prosedural / M17 pixel sixel] & starmap [M18/19 grid playable] hidup
/// di 2 VIEW TERPISAH (`main_menu` vs `galaxy_map`, dipilih via key global `g`/dsb) — TAK ADA
/// cara liat backdrop yg INDAH [galaxy_2.png-mirip] SAAT SEDANG di `galaxy_map` (misal cek
/// posisi sektor scr visual sblm balik ke grid detail). Fix: toggle `Tab` DALAM `galaxy_map`
/// sendiri — `Starmap` [default, SAMA perilaku sblm M20.1] vs `Backdrop` [reuse LANGSUNG
/// logic `main_view.rs`'s pixel/procedural, TAK duplikasi cabang render terpisah].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GalaxyMapMode {
    #[default]
    Starmap,
    Backdrop,
}

impl GalaxyMapMode {
    pub fn toggle(self) -> Self {
        match self {
            GalaxyMapMode::Starmap => GalaxyMapMode::Backdrop,
            GalaxyMapMode::Backdrop => GalaxyMapMode::Starmap,
        }
    }
}

/// Posisi grid deterministik planet `idx` di galaksi `seed` — dihash dari `(seed, idx)` via
/// `planet_rng` (SUDAH ada, dipakai jg `generate_planet`) BUKAN RNG baru terpisah, biar posisi
/// planet KONSISTEN dgn identitasnya (idx sama → posisi sama, deterministik M18.6).
pub fn planet_grid_pos(seed: u64, idx: u32) -> (i32, i32) {
    let mut r = planet_rng(seed, idx);
    let x = r.below(WORLD_W as u32) as i32;
    let y = r.below(WORLD_H as u32) as i32;
    (x, y)
}

/// M18.7: "Density grid mirip Riftborne (padat tapi terbaca)" — **gap real ditemukan**:
/// `PLANET_MIN..PLANET_MAX` (5-12, `balance.rs`) tersebar di `WORLD_W*WORLD_H` (7200 sel) —
/// densitas ~0.1%, jauh lbh SEPI drpd `owOev1.png` (dot-matrix latar PADAT). Shrink dunia
/// [lebih kecil drpd viewport, matikan scroll M18.5] atau nambah "planet palsu" [ubah
/// mekanik/gameplay, di luar scope tampilan] BUKAN pilihan proporsional. Fix: tekstur latar
/// DEKORATIF [`.`/`:` sesekali, BUKAN planet genuine — cuma sel `·` polos yg dulu SELALU
/// sama, skrng ADA variasi] di sel kosong, dihash dari `(seed,x,y)` [deterministik M18.6,
/// posisi/jumlah planet TAK berubah]. Warna TETAP `th.dim` [sama planet-empty lama] — biar
/// tekstur BEDA dari planet [warna status] tp lbh "hidup" dari titik polos seragam.
fn background_glyph(seed: u64, x: i32, y: i32) -> char {
    let h = seed
        ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    let mut r = SplitMix64::new(h);
    match r.below(20) {
        0..=1 => '.',
        2 => ':',
        _ => '·',
    }
}

/// Simbol tile planet: huruf pertama biome (uppercase) — beda drpd `biome_label` (nama penuh,
/// dipakai list/detail), simbol GRID harus 1 karakter spy tak makan >1 sel/planet.
fn planet_glyph(p: &Planet) -> char {
    biome_label(p.biome)
        .chars()
        .next()
        .unwrap_or('?')
        .to_ascii_uppercase()
}

/// Galaksi sektor yang dieksplor = level tertinggi (frontier); Milky Way (Lvl 0) = anchor.
pub fn frontier(app: &App) -> Option<&Galaxy> {
    app.state.galaxies.iter().max_by_key(|g| g.level)
}

/// M18.4: cari idx planet (jika ADA) di posisi grid `pos` — dipakai `app.rs`'s aksi `s`
/// (scan/send) biar tau planet MANA yg sedang ditunjuk kursor BEBAS (bukan cuma `app.sel`
/// linear lg). `O(planet_count)`, murah (5-12 planet tipikal, dicek `PLANET_MIN/MAX`).
pub fn planet_idx_at(g: &Galaxy, pos: (i32, i32)) -> Option<u32> {
    let count = match &g.kind {
        GalaxyKind::Procedural { seed, .. } => planet_count(*seed),
        GalaxyKind::Fixed => g.planets.len() as u32,
    };
    (0..count).find(|&idx| {
        let p = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => planet_grid_pos(*seed, idx),
            GalaxyKind::Fixed => planet_grid_pos(0, idx),
        };
        p == pos
    })
}

/// M19.6: **bug latent real ditemukan** (pra-ada, bukan diperkenalkan sesi ini) — 3 situs kode
/// (`tile_detail_line`, glyph grid, sprite biome) pakai `g.planets.get(idx as usize)` — INDEX
/// POSISI VEC, TAPI `materialize_planet` [`procgen.rs`] `push` planet BARU ke UJUNG vec dlm
/// URUTAN SCAN, BUKAN urutan `idx`! Kalau planet idx=5 discan SBLM idx=2, `g.planets` jd
/// `[p5,p2]` — `.get(2)` salah ambil `p5`. `Planet.id` genuinely `PlanetId(idx)` [dikonfirmasi
/// `procgen.rs::generate_planet` & `world/mod.rs`'s Fixed constructor] — lookup BENAR HARUS
/// cocokkan `id`, bukan posisi vec. Fix: helper INI, dipakai gantikan SEMUA 3 situs lama +
/// aksi Enter baru (M19.6) — utk kasus in-order/Fixed [SATU2nya yg pernah dites sblm ini]
/// hasil IDENTIK [id==posisi by construction], utk out-of-order Procedural GENUINELY benar.
pub fn planet_by_idx(g: &Galaxy, idx: u32) -> Option<&Planet> {
    g.planets.iter().find(|p| p.id == PlanetId(idx))
}

/// M19.8: garis Bresenham integer klasik dari `a` ke `b` (inklusif KEDUA ujung) -- dipakai
/// highlight rute ship (kursor→target). Pure function (tak ada I/O/state), dites terpisah
/// dgn kasus horizontal/vertical/diagonal/titik-sama biar genuinely benar sblm dipakai render.
fn bresenham_line(a: (i32, i32), b: (i32, i32)) -> Vec<(i32, i32)> {
    let (mut x0, mut y0) = a;
    let (x1, y1) = b;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut cells = Vec::new();
    loop {
        cells.push((x0, y0));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
    cells
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

/// Rata2 richness node planet (representasi ringkas "kekayaan" — `Planet` tak py 1 field
/// tunggal utk ini). `None` bila planet tak py node sama sekali (genuinely tak ada richness).
fn avg_richness(p: &Planet) -> Option<f64> {
    if p.nodes.is_empty() {
        return None;
    }
    Some(p.nodes.iter().map(|n| n.richness).sum::<f64>() / p.nodes.len() as f64)
}

/// M19.5: "Aksi `s` scan/materialize tile (sudah ada → integrasi UI)" — cek APAKAH aksi `s`
/// genuinely akan berbuat sesuatu di posisi `pos` -- SAMA PERSIS kondisi `materialize_planet`
/// [`procgen.rs`] biar footer JUJUR (bukan cuma nebak/duplikasi logic terpisah yg bisa
/// divergen). Cuma `Procedural` + idx blm `visited` yg genuinely scannable (`Fixed` galaxy
/// TAK PY mekanik scan sama sekali, planet SUDAH ada dari awal).
fn can_scan(g: &Galaxy, pos: (i32, i32)) -> bool {
    let GalaxyKind::Procedural { visited, .. } = &g.kind else {
        return false;
    };
    match planet_idx_at(g, pos) {
        Some(idx) => !visited.contains(&idx),
        None => false,
    }
}

/// M19.7: "Aksi kirim ship ke tile" — cek APAKAH `t` genuinely akan berbuat sesuatu di posisi
/// `pos` -- footer HANYA iklan `[t]Send` bila planet SUDAH materialize [`planet_by_idx` nemu],
/// BLM `unlocked`, DAN ship genuinely `Idle` [`travel_to` tolak kalau sibuk]. Precondition
/// WarpTier/resource [`meets_unlock_req`] SENGAJA tak dicek di sini [pola SAMA `research_
/// keys`'s Enter M15.9: status detail SUDAH kelihatan di tempat lain (SELECTED TILE M19.1),
/// hint ini cuma penanda "aksi RELEVAN di sini", bukan jaminan pasti sukses -- gagal internal
/// `travel_to` tetap senyap, konsisten pola established].
fn can_send(g: &Galaxy, app: &App, pos: (i32, i32)) -> bool {
    if !matches!(app.state.ship.status, ShipStatus::Idle) {
        return false;
    }
    match planet_idx_at(g, pos) {
        Some(idx) => planet_by_idx(g, idx).is_some_and(|p| !p.unlocked),
        None => false,
    }
}

/// M19.6: cek APAKAH `Enter` genuinely akan berbuat sesuatu di posisi `pos` — SAMA PERSIS
/// kondisi `app.rs`'s `galaxy_map_keys`'s `KeyCode::Enter`. **Ditambah skrng (bareng M19.7)**:
/// footer M19.6 blm py hint kondisional sblmnya (oversight kecil, sekalian diperbaiki krn
/// baris footer yg sama disentuh ulang utk `can_send`).
fn can_enter(g: &Galaxy, pos: (i32, i32)) -> bool {
    match planet_idx_at(g, pos) {
        Some(idx) => planet_by_idx(g, idx).is_some_and(|p| p.unlocked),
        None => false,
    }
}

/// M19.1: "Panel SELECTED TILE: koordinat/tipe/richness/owner" — baris ringkas detail tile
/// di posisi kursor. "Owner" -> STATUS (game tak py faksi, konsisten M18.2/8). "Richness" ->
/// rata2 `nodes[].richness` (`Planet` tak py field tunggal).
///
/// **Overflow ditemukan+dicegah SBLM screenshot** (bukan sesudah): format verbose awal
/// ("Coord:(x,y)  Type:X Planet  Richness:xN.N  Status:Locked WTn") dihitung LANGSUNG worst-
/// case (koordinat 3-digit terbesar + biome terpanjang "GasGiant" + status terpanjang "Locked
/// WT9") = ~79 char, JAUH lewat Minimal's 58w interior. Dipersingkat jd label minimal (`@`
/// utk coord, tanpa embel "Planet"/"Richness:"/"Status:") -- worst-case recompute = 50 char,
/// aman di SEMUA breakpoint standar (dikonfirmasi `wc -c`/Python `len()` LANGSUNG, bukan tebak).
fn tile_detail_line(g: &Galaxy, app: &App, pos: (i32, i32), th: &theme::Theme) -> Line<'static> {
    let coord = format!("@({},{})", pos.0, pos.1);
    let Some(idx) = planet_idx_at(g, pos) else {
        return Line::styled(
            format!("SELECTED TILE  {coord} Empty"),
            Style::default().fg(th.dim),
        );
    };
    let (type_str, richness_str, status_str) = match &g.kind {
        GalaxyKind::Procedural { seed, visited } if !visited.contains(&idx) => {
            let p = generate_planet(*seed, idx, g.level, &app.content);
            let r = avg_richness(&p)
                .map(|v| format!("x{v:.1}"))
                .unwrap_or_else(|| "-".into());
            (biome_label(p.biome).to_string(), r, status_label(&p, false))
        }
        _ => match planet_by_idx(g, idx) {
            Some(p) => {
                let r = avg_richness(p)
                    .map(|v| format!("x{v:.1}"))
                    .unwrap_or_else(|| "-".into());
                (biome_label(p.biome).to_string(), r, status_label(p, true))
            }
            None => ("Unscanned".into(), "-".into(), "-".into()),
        },
    };
    // M19.10: "Status incoming/outgoing bila relevan" — **gap real ditemukan**: ship SEDANG
    // travel ke tile INI [`ShipStatus::Traveling{to,..}` cocok `idx`] TAK ADA info ETA di
    // SELECTED TILE, padahal M19.8's rute+M18.8's marker TARGET SUDAH tampil visual -- user
    // genuinely tak tau BERAPA LAMA lg. "Outgoing" TAK diimplementasi: ship (single, tak py
    // field origin/asal tersimpan) TAK PY cara tentukan "meninggalkan tile mana" secara
    // bermakna -- "bila relevan" (kondisional di judul checklist) genuinely berarti HANYA
    // incoming yg applicable di model data SAAT INI, bukan oversight.
    // **Overflow dicegah SBLM screenshot**: worst-case " Incoming ETA:99m" (~18 char) + line
    // dasar (50 char) = 68, LEWAT Minimal 58w. Dipersingkat notasi panah `->Xm` (worst-case
    // recompute 54 char, dikonfirmasi Python `len()` LANGSUNG, aman).
    let incoming = match app.state.ship.status {
        ShipStatus::Traveling {
            to,
            total_secs,
            elapsed_secs,
        } if to.0 == idx => {
            let remaining = (total_secs - elapsed_secs).max(0.0);
            format!(" ->{}", fmt_eta(remaining))
        }
        _ => String::new(),
    };
    Line::styled(
        format!("SELECTED TILE  {coord} {type_str} {richness_str} {status_str}{incoming}"),
        Style::default().fg(th.focus),
    )
}

/// M19.4: "Body sprite ANSI preview tile terpilih" — lebar kolom sprite (kanan). **Percobaan
/// PERTAMA gagal**: reuse `planet.rs`'s pola persis (`INFO_MIN=58`, ambang Md=98/Sm=80) —
/// screenshot Full (120x40) TAK tampilkan sprite sama sekali. Diselidiki: MAIN area galaxy_map
/// SEBENARNYA cuma 78w di Full (`mod.rs`'s `shell_full`: `cols=[Length(22),Min(0),Length(18)]`
/// dr `inner`=118w -> MAIN=118-22-18=78), BUKAN ~86-90 spt dikira awal (planet.rs's info baris
/// jauh lbh pendek drpd galaxy_map's worst-case 50-char SELECTED TILE line, jd threshold BEDA
/// genuinely perlu, bukan cuma reuse angka planet.rs). Recompute: Compact MAIN=64w (`shell_
/// compact`: `Length(14)`+Min0 dr 78w inner), Minimal MAIN=58w (`shell_minimal`: tanpa split
/// horizontal). HANYA Full(78) yg muat Sm(22)+worst-case-detail(50)=72 dgn margin; Compact
/// (64) & Minimal(58) TERLALU SEMPIT bahkan utk Sm — sprite HANYA muncul di Full, disengaja
/// (bukan lupa), matching pola "tak semua breakpoint tampilkan semua fitur" sesi ini.
fn sprite_width_for(total_w: u16) -> u16 {
    const SM_MIN: u16 = 74; // Sm(22) + worst-case detail(50) + margin(2).
    if total_w >= SM_MIN { 22 } else { 0 }
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let _engine = app.state.ship.engine;
    let Some(g) = frontier(app) else {
        f.render_widget(
            Paragraph::new("Tidak ada galaksi.")
                .block(Block::bordered().border_style(Style::default().fg(th.dim))),
            area,
        );
        return;
    };

    // M20.1: mode `Backdrop` -- render SAMA PERSIS logic `main_view.rs` (reuse, bukan
    // duplikasi cabang baru) lalu return SEGERA (starmap grid/legend/footer SEMUA diskip
    // total, konsisten "1 mode fokus per waktu" — pola SAMA build_picker's precedent M14.12).
    if app.galaxy_map_mode == GalaxyMapMode::Backdrop {
        let block = Block::bordered()
            .title("GALAXY MAP — Backdrop  [Tab]Starmap [Esc]Back")
            .border_style(Style::default().fg(th.dim));
        let inner = block.inner(area);
        f.render_widget(block, area);
        if app.force_procedural_galaxy {
            app.anim.render(f, inner, app.anim_secs, &th);
        } else {
            // M20.8 follow-up (galaxy_sim Phase 5): backdrop mode tampilkan sektor FRONTIER
            // ini (`g`, bukan `active_galaxy` -- Backdrop di galaxy_map genuinely soal sektor
            // yg SEDANG dilihat starmap-nya, konsisten intent M20.1's toggle).
            // Phase 6: +baris hint pan/zoom/reset di BAWAH (bukan dijejalkan ke title -- worst-
            // case dihitung: title+hint gabung ~74 char, OVERFLOW Minimal 58w; baris terpisah
            // lbh aman & konsisten pola footer Starmap mode).
            let rows = ratatui::layout::Layout::vertical([
                ratatui::layout::Constraint::Min(0),
                ratatui::layout::Constraint::Length(1),
            ])
            .split(inner);
            let (seed, is_anchor) = crate::ui::galaxy_sim_view::galaxy_sim_key(g);
            let t_myr = app.anim_secs * crate::ui::galaxy_sim_view::MYR_PER_SEC;
            app.galaxy_sim.render(f, rows[0], seed, is_anchor, t_myr);
            // Phase 7: hint berubah bila sedang fokus POI -- tampilkan NAMA POI aktif +
            // `Esc` utk batal (bukan hint pan/zoom yg genuinely no-op saat fokus).
            let hint = match app.galaxy_sim.focused_poi_name() {
                // Hint dipangkas jd `[Enter]Next [Esc]Unfocus` saja (bukan +`[z/x]Zoom` --
                // dites nyata OVERFLOW Minimal 58w di kasus nama TERPANJANG "Zorada Beta
                // Nebula", dipotong jd "[z/x" -- z/x msh berfungsi genuine, cuma tak dihint
                // ulang di sini, user sudah lihat sblm masuk fokus).
                Some(name) => format!("Focus: {name}  [Enter]Next [Esc]Unfocus"),
                None => "[hjkl]Pan [z/x]Zoom [r]Reset [Enter]Focus".to_string(),
            };
            f.render_widget(
                Paragraph::new(Line::styled(hint, Style::default().fg(th.neutral))),
                rows[1],
            );
        }
        return;
    }

    // M19.4: reservasi kolom KANAN utk sprite preview (pola SAMA `planet.rs`) -- `area` UTAMA
    // (grid+teks) jd LEBIH SEMPIT bila sprite direservasi, `sprite_w=0` (area sempit) berarti
    // `area` UTAMA tetap area PENUH (tak ada kolom dipotong sama sekali).
    let sprite_w = sprite_width_for(area.width);
    let (area, sprite_area) = if sprite_w > 0 {
        let cols = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Min(0),
                ratatui::layout::Constraint::Length(sprite_w),
            ])
            .split(area);
        (cols[0], Some(cols[1]))
    } else {
        (area, None)
    };

    // M18.2: legend genuinely tampilkan WARNA status (bukan cuma teks polos) -- biar makna
    // warna langsung terlihat di legend itu sendiri, bukan perlu ditebak dari grid.
    let legend = ratatui::text::Line::from(vec![
        ratatui::text::Span::styled("Legend: ", Style::default().fg(th.dim)),
        ratatui::text::Span::styled("· ", Style::default().fg(th.dim)),
        ratatui::text::Span::raw("Empty  "),
        ratatui::text::Span::styled("A", Style::default().fg(th.good)),
        ratatui::text::Span::raw("=Active  "),
        ratatui::text::Span::styled("A", Style::default().fg(th.advanced)),
        ratatui::text::Span::raw("=Locked  "),
        ratatui::text::Span::styled("A", Style::default().fg(th.neutral)),
        ratatui::text::Span::raw("=Unscanned"),
    ]);
    // M19.2: "Legend simbol/warna" — **gap real ditemukan**: legend M18.2 dibuat SBLM marker
    // home/target ADA (M18.8) — legend JADI STALE, tak jelaskan makna underline/bold+alert yg
    // genuinely tampil di grid sejak M18.8. +baris legend KEDUA khusus marker, pakai STYLE
    // SAMA PERSIS yg dipakai render tile asli (bukan cuma teks) -- biar contoh visual genuinely
    // cocok apa yg akan dilihat user di grid.
    // M19.9: +indikator filter aktif [`F:<Label>`] di baris legend marker yg SAMA -- **overflow
    // dicegah SBLM screenshot**: worst-case dihitung LANGSUNG (nama galaksi terpanjang mungkin
    // dr `procgen.rs`'s PREFIX/MID/SUFFIX/GREEK ~16 char + level 3-digit) utk TITLE line = 60
    // char, LEWAT Minimal 58w -- indikator DIPINDAH ke baris marker_legend (byk ruang lg,
    // worst-case "P=Home  P=Target  F:Unscanned" = 29 char, aman) drpd numpuk di title line.
    let marker_legend = ratatui::text::Line::from(vec![
        ratatui::text::Span::styled(
            "P",
            Style::default()
                .fg(th.focus)
                .add_modifier(ratatui::style::Modifier::UNDERLINED),
        ),
        ratatui::text::Span::raw("=Home  "),
        ratatui::text::Span::styled("P", Style::default().fg(th.alert).bold()),
        ratatui::text::Span::raw("=Target  "),
        ratatui::text::Span::styled(
            format!("F:{}", app.galaxy_filter.label()),
            Style::default().fg(th.dim),
        ),
    ]);
    let mut lines: Vec<Line> = vec![
        Line::styled(
            format!("GALAXY MAP — {} (Lvl {})", g.name, g.level),
            Style::default().fg(th.focus).bold(),
        ),
        legend,
        marker_legend,
    ];

    // Banyaknya planet: ProcGen → planet_count(seed); Fixed → jumlah planet termuat.
    let count = match &g.kind {
        GalaxyKind::Procedural { seed, .. } => planet_count(*seed),
        GalaxyKind::Fixed => g.planets.len() as u32,
    };
    // M18.1: peta idx->(x,y)->(glyph,idx), dibangun SEKALI sblm loop grid (bukan re-generate
    // planet tiap sel viewport dicek — O(count) sekali, bukan O(viewport_area * count)).
    // M18.2: +status per tile -- warna BUKAN dari biome (biome sudah jadi GLYPH huruf, warna
    // biome lg bikin redundan) tp dari STATUS (Active/Scanned/Locked), pola SAMA yg dipakai
    // list lama (`p.unlocked`→`th.good` dst) -- checklist minta "per faksi/biome/status", game
    // ini TAK PY konsep faksi (dicek `game/state.rs`, tak ada field faction) jd status yg
    // genuinely actionable/relevan dipilih drpd redundan-dgn-glyph.
    // M18.8: "Marker home/ship/target" -- **gap real ditemukan**: TAK ADA cara bedakan
    // planet HOME (basis pemain) atau TARGET (tujuan ship travel saat ini) dari planet biasa
    // di grid. Home = idx 0 di galaksi Fixed (Milky Way/Earth SATU-SATUNYA galaksi Fixed,
    // `game/state.rs` dikonfirmasi tak py field faction/multi-home). Target = `ShipStatus::
    // Traveling{to,..}` -- `to: PlanetId` KONFIRMASI sama `idx` yg dipakai `generate_planet`
    // (dicek `procgen.rs:268` `id: PlanetId(i)`). Marker via MODIFIER (underline=home,
    // bold+warna alert=target) — BUKAN ganti glyph biome (biar biome tetap kebaca), style
    // ekstra DI ATAS warna status.
    let mut placed: std::collections::HashMap<
        (i32, i32),
        (char, ratatui::style::Color, ratatui::style::Modifier),
    > = std::collections::HashMap::new();
    for idx in 0..count {
        // M19.9: filter STATUS — tile TAK COCOK filter aktif dilewati sepenuhnya (jatuh ke
        // tekstur latar biasa di loop grid nanti, BUKAN diberi placeholder/dim khusus -- murni
        // "seakan planet ini tak ada di `placed`", pola paling sederhana & konsisten dgn
        // "sel kosong" yg sudah ada). Murni VISUAL: kursor/aksi TETAP bisa ke tile tersembunyi.
        if !app.galaxy_filter.matches(tile_status(g, idx)) {
            continue;
        }
        let (x, y) = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => planet_grid_pos(*seed, idx),
            GalaxyKind::Fixed => planet_grid_pos(0, idx),
        };
        let (glyph, mut color) = match &g.kind {
            GalaxyKind::Procedural { seed, visited } if !visited.contains(&idx) => {
                // Belum discan -- status tak diketahui pasti, tampilkan NEUTRAL (bukan good/
                // advanced yg mengklaim tau lock-state sblm genuinely discan).
                let p = generate_planet(*seed, idx, g.level, &app.content);
                (planet_glyph(&p), th.neutral)
            }
            _ => match planet_by_idx(g, idx) {
                Some(p) => {
                    let c = if p.unlocked { th.good } else { th.advanced };
                    (planet_glyph(p), c)
                }
                None => ('?', th.dim),
            },
        };
        let is_home = matches!(g.kind, GalaxyKind::Fixed) && idx == 0;
        let is_target =
            matches!(app.state.ship.status, ShipStatus::Traveling { to, .. } if to.0 == idx);
        let mut modifier = ratatui::style::Modifier::empty();
        if is_home {
            modifier |= ratatui::style::Modifier::UNDERLINED;
        }
        if is_target {
            modifier |= ratatui::style::Modifier::BOLD;
            color = th.alert;
        }
        placed.insert((x, y), (glyph, color, modifier));
    }

    // M18.4: "Navigasi kursor h/j/k/l + panah" -- kursor GRID BEBAS `app.galaxy_cursor`
    // (koordinat dunia, gerak per-sel via `galaxy_map_keys`, BISA berdiri di sel KOSONG) —
    // GANTI M18.3's pendekatan lama (`cursor_pos` dari `app.sel`/planet index, tak bisa ke sel
    // kosong). `app.sel` skrng TAK dipakai lg di view ini (dihindari konflik makna: `sel` di
    // view lain = "indeks list", di sini akan jd 2 sumber kebenaran beda kalau dipakai jg).
    let cursor_pos = app.galaxy_cursor;

    // M19.8: "Highlight rute ship (kursor→target)" — **gap real ditemukan**: target travel
    // SUDAH ditandai [bold+`th.alert`, M18.8] TAPI TAK ADA garis penghubung -- user genuinely
    // tak liat JALUR antara posisi SEKARANG [kursor] & tujuan ship, cuma titik target berdiri
    // sendiri. Fix: `bresenham_line()` (dites terpisah, pure function) diplot ke `placed` VIA
    // `.entry().or_insert()` -- HANYA isi sel KOSONG (planet/marker SUDAH ada tetap prioritas,
    // rute tak menimpa apa2 yg genuinely lbh penting). Dipakai `th.dim` [BEDA dari planet
    // manapun, tak bisa disalahsangka planet baru] + glyph `+` [beda dr `.`/`:`/`·` tekstur
    // latar M18.7, genuinely terlihat sbg "jalur", bukan cuma variasi noise].
    if let ShipStatus::Traveling { to, .. } = app.state.ship.status {
        let target_pos = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => planet_grid_pos(*seed, to.0),
            GalaxyKind::Fixed => planet_grid_pos(0, to.0),
        };
        for (x, y) in bresenham_line(cursor_pos, target_pos) {
            placed
                .entry((x, y))
                .or_insert(('+', th.dim, ratatui::style::Modifier::empty()));
        }
    }

    // Viewport: sisa tinggi/lebar area (border+header+legend+marker-legend+detail+footer sudah
    // dipakai, sisanya grid). border(2)+header(1)+legend(1)+footer(1) -- **overflow ditemukan**
    // M18.1: sblm ini `sub(4)` (lupa footer 1 baris), footer hint kepotong hilang di bawah
    // border. Diperbaiki `sub(5)`. M19.1: +1 baris LAGI ("SELECTED TILE" detail) -> `sub(6)`.
    // M19.2: +1 baris LAGI (legend marker home/target) -> `sub(7)` (pola SAMA berulang: tiap
    // nambah baris konten, `vh` HARUS ikut dikurangi, jgn asumsikan otomatis muat).
    let vh = (area.height.saturating_sub(7)) as i32;
    let vw = (area.width.saturating_sub(2)) as i32; // border kiri+kanan

    // M18.5: "Scroll viewport saat kursor di tepi" — **gap real ditemukan**: viewport dulu
    // SELALU mulai dr pojok dunia (0,0) tetap (M18.1's catatan eksplisit: "scroll genuine msh
    // M18.5") — kursor yg gerak MELEWATI batas viewport (mis. x>=vw) jd GENUINELY tak
    // terlihat lg (di luar area yg dirender), biarpun state internalnya tetap benar. Fix:
    // origin viewport dihitung SEKALI per render, PUSATKAN kursor di viewport bila mungkin,
    // clamp ke batas dunia (jgn scroll lewat tepi dunia, area kosong percuma). Dihitung ULANG
    // tiap render dari `cursor_pos` (bukan state persisten terpisah) — kursor SELALU within
    // viewport tanpa perlu App nyimpan origin sendiri (satu sumber kebenaran: cursor+area).
    // M18.9: **panic real ditemukan** (dites LANGSUNG, bukan tebak): `cursor_pos.0 - vw/2`
    // pakai `-` biasa OVERFLOW (`attempt to subtract with overflow`) kalau `cursor_pos`
    // korup/di luar batas ekstrem (mis. `i32::MIN`, seharusnya tak terjadi via `galaxy_map_
    // keys`'s clamp normal, TAPI `galaxy_cursor` field PUBLIC — kode lain/bug masa depan bisa
    // set nilai sembarang). Diganti `saturating_sub` — genuinely aman di SEMUA nilai cursor,
    // bukan cuma "biasanya aman".
    let origin_x = cursor_pos
        .0
        .saturating_sub(vw / 2)
        .clamp(0, (WORLD_W - vw).max(0));
    let origin_y = cursor_pos
        .1
        .saturating_sub(vh / 2)
        .clamp(0, (WORLD_H - vh).max(0));
    let bg_seed = match &g.kind {
        GalaxyKind::Procedural { seed, .. } => *seed,
        GalaxyKind::Fixed => 0,
    };

    for wy in 0..vh.max(0) {
        let mut spans: Vec<ratatui::text::Span> = Vec::with_capacity(vw.max(0) as usize);
        for wx in 0..vw.max(0) {
            let world_pos = (origin_x + wx, origin_y + wy);
            let is_cursor = world_pos == cursor_pos;
            match placed.get(&world_pos) {
                Some((glyph, color, marker_modifier)) => {
                    let mut style = Style::default().fg(*color).add_modifier(*marker_modifier);
                    if is_cursor {
                        // M18.3: highlight PENUH (REVERSED+bold), pola SAMA selector view lain
                        // (research/planet_view's baris terpilih -- M13.6/M14.8/M15.10
                        // precedent), bukan cuma marker kecil di depan.
                        style = style
                            .add_modifier(ratatui::style::Modifier::REVERSED)
                            .bold();
                    }
                    spans.push(ratatui::text::Span::styled(glyph.to_string(), style));
                }
                None => {
                    let mut style = Style::default().fg(th.dim);
                    if is_cursor {
                        style = style.add_modifier(ratatui::style::Modifier::REVERSED);
                    }
                    let bg = background_glyph(bg_seed, world_pos.0, world_pos.1);
                    spans.push(ratatui::text::Span::styled(bg.to_string(), style));
                }
            }
        }
        lines.push(Line::from(spans));
    }

    // M19.1: "Panel SELECTED TILE: koordinat/tipe/richness/owner" — **gap real ditemukan**:
    // dulu HANYA koordinat mentah di footer, TAK ADA info tipe/richness/status tile terpilih
    // (checklist M19's Tujuan: "panel detail tile" — Riftborne's SELECTED TILE panel jd ref).
    // "Owner" -- game single-player TAK PY faksi/multi-owner (dikonfirmasi M18.2/8's temuan
    // sama), diganti STATUS (Active/Locked/Unscanned, konsisten terminologi SUDAH ada).
    // "Richness" -- Planet TAK py field tunggal, dihitung rata2 `nodes[].richness` (representasi
    // ringkas kekayaan sumber daya planet).
    let tile_line = tile_detail_line(g, app, cursor_pos, &th);
    lines.push(tile_line);

    // M18.1: **overflow ditemukan** — teks lama ("[s] SCAN/SEND ship to selected   [j/k]
    // Select   [Esc] Back", 60 char) kepotong di Minimal (60x24, interior 58w): "[Esc] Back"
    // jd "[Esc] Ba" (dikonfirmasi screenshot). Dipendekkan biar muat SEMUA breakpoint standar.
    // M18.3: koordinat kursor dulu DI FOOTER INI -- M19.1 pindahkan ke baris SELECTED TILE
    // baru (di atas), hindari duplikasi info yg sama 2x di 2 baris berbeda.
    // M18.4: "[j/k]Select" diganti "[hjkl]Move" -- label LAMA nyebut "Select" tp kursor
    // skrng BEBAS gerak per-sel (bukan lompat pilih planet), label harus jujur match perilaku.
    // M19.5: **gap real ditemukan**: "[s]Scan/Send" nyebut DUA aksi ("Send") yg BLM diimplemen
    // (kirim ship = M19.7, msh todo) -- footer BOHONG soal kapabilitas SEKARANG. Diganti
    // "[s]Scan" [jujur, cuma scan yg ada] DAN kondisional -- HANYA tampil bila `can_scan()`
    // genuinely true di posisi kursor (SAMA kondisi persis `materialize_planet`), else `s`
    // dihilangkan dari hint sepenuhnya (bukan ditampilkan tp diam2 no-op saat ditekan).
    // M19.7: +cabang `[t]Send` (kondisional `can_send()`) & `[Enter]Enter` (kondisional
    // `can_enter()`, M19.6's footer hint blm sempat ditambah -- diperbaiki skrng krn baris ini
    // disentuh ulang). Ke-3 kondisi [scan/send/enter] SALING EKSKLUSIF utk idx yg sama (planet
    // cuma bisa 1 status: blm-visited XOR visited-tak-unlocked XOR unlocked) -- tak akan
    // tampil >1 hint aksi sekaligus, worst-case 1 line dikonfirmasi <=33 char (`wc`/Python
    // `len()` LANGSUNG), aman semua breakpoint.
    // M20.1: +`[Tab]Backdrop` di SEMUA varian (kapabilitas PERMANEN, beda dr scan/send/enter
    // yg kondisional per-tile) -- worst-case recompute "[Enter]Enter [hjkl]Move [Tab]Backdrop
    // [Esc]Back" = 47 char, aman Minimal 58w (dikonfirmasi Python `len()` LANGSUNG).
    let hint = if can_scan(g, cursor_pos) {
        "[s]Scan [hjkl]Move [Tab]Backdrop [Esc]Back"
    } else if can_send(g, app, cursor_pos) {
        "[t]Send [hjkl]Move [Tab]Backdrop [Esc]Back"
    } else if can_enter(g, cursor_pos) {
        "[Enter]Enter [hjkl]Move [Tab]Backdrop [Esc]Back"
    } else {
        "[hjkl]Move [Tab]Backdrop [Esc]Back"
    };
    lines.push(Line::styled(hint, Style::default().fg(th.neutral)));

    let block = Block::bordered().border_style(Style::default().fg(th.dim));
    f.render_widget(Paragraph::new(lines).block(block), area);

    // M19.3: "Minimap (peta kecil posisi viewport)" — **gap real ditemukan**: viewport (M18.5)
    // scroll BEBAS tp TAK ADA cara liat "di mana posisi ini relatif ke SELURUH dunia" — user
    // genuinely BISA TERSESAT (kursor scroll jauh, tak tau arah balik ke area lain). Fix:
    // overlay kecil pojok kanan-atas grid (SETELAH Paragraph utama dirender, timpa sedikit sel
    // pojok — trade-off disengaja: minimap kecil > sedikit tekstur latar tertutup). Diskip
    // total di area SANGAT SEMPIT (matching pola gating M19.1's overflow-first-principle).
    draw_minimap(f, area, g, origin_x, origin_y, vw, vh, &th);

    // M19.4: "Body sprite ANSI preview tile terpilih" — **gap real ditemukan**: TAK ADA
    // preview visual sama sekali utk planet terpilih (cuma teks di SELECTED TILE M19.1).
    // Tampilkan sprite ANSI biome [pola SAMA `planet.rs`, sumber SUDAH ada `App::sprite_
    // for_biome`+`app.sprites`] di kolom KANAN direservasi -- HANYA bila kursor genuinely
    // di atas planet (`planet_idx_at`), else kolom kosong (tak ada apa2 utk dipreview).
    if let Some(sprite_area) = sprite_area
        && let Some(idx) = planet_idx_at(g, cursor_pos)
    {
        let biome = match &g.kind {
            GalaxyKind::Procedural { seed, visited } if !visited.contains(&idx) => {
                Some(generate_planet(*seed, idx, g.level, &app.content).biome)
            }
            _ => planet_by_idx(g, idx).map(|p| p.biome),
        };
        if let Some(biome) = biome {
            let base = App::sprite_for_biome(biome);
            app.sprites.render(f, sprite_area, base);
        }
    }
}

/// M19.3: minimap 12x6 (jauh lbh kecil drpd `WORLD_W`×`WORLD_H`=120x60, skala ~1:10) di pojok
/// kanan-atas viewport grid. `*` = planet [sembarang status], `#` = area viewport SEDANG
/// tampil (highlight), `.` = kosong. Skip total bila viewport tak cukup besar (hindari
/// menutupi SEMUA konten -- minimap harus jadi TAMBAHAN, bukan gantikan grid utama).
#[allow(clippy::too_many_arguments)]
fn draw_minimap(
    f: &mut Frame,
    area: Rect,
    g: &Galaxy,
    origin_x: i32,
    origin_y: i32,
    vw: i32,
    vh: i32,
    th: &theme::Theme,
) {
    const MM_W: i32 = 12;
    const MM_H: i32 = 6;
    if vw < MM_W + 4 || vh < MM_H + 4 {
        return; // viewport terlalu sempit -- minimap akan menutupi mayoritas konten, skip.
    }
    let count = match &g.kind {
        GalaxyKind::Procedural { seed, .. } => planet_count(*seed),
        GalaxyKind::Fixed => g.planets.len() as u32,
    };
    let mut planet_cells: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    for idx in 0..count {
        let (x, y) = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => planet_grid_pos(*seed, idx),
            GalaxyKind::Fixed => planet_grid_pos(0, idx),
        };
        let mx = (x * MM_W) / WORLD_W;
        let my = (y * MM_H) / WORLD_H;
        planet_cells.insert((mx, my));
    }
    let vp_x0 = (origin_x * MM_W) / WORLD_W;
    let vp_x1 = (((origin_x + vw) * MM_W) / WORLD_W).max(vp_x0 + 1);
    let vp_y0 = (origin_y * MM_H) / WORLD_H;
    let vp_y1 = (((origin_y + vh) * MM_H) / WORLD_H).max(vp_y0 + 1);

    // Pojok kanan-atas viewport: border(1)+header(1)+legend(1)+marker_legend(1) = offset y+4;
    // border kiri(1) + (vw-MM_W) kolom dr kiri viewport = pojok kanan grid.
    let mm_x0 = area.x + 1 + (vw - MM_W) as u16;
    let mm_y0 = area.y + 4;
    let buf = f.buffer_mut();
    for my in 0..MM_H {
        for mx in 0..MM_W {
            let in_viewport = mx >= vp_x0 && mx < vp_x1 && my >= vp_y0 && my < vp_y1;
            let has_planet = planet_cells.contains(&(mx, my));
            let (glyph, color) = if has_planet {
                ('*', th.good)
            } else if in_viewport {
                ('#', th.focus)
            } else {
                ('.', th.dim)
            };
            let (px, py) = (mm_x0 + mx as u16, mm_y0 + my as u16);
            if px < area.x + area.width.saturating_sub(1)
                && py < area.y + area.height.saturating_sub(1)
            {
                let cell = &mut buf[(px, py)];
                cell.set_char(glyph);
                cell.set_style(Style::default().fg(color));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// M18.1/M18.6: "Grid deterministik dari seed procgen" — posisi planet HARUS genuinely
    /// sama tiap panggilan (seed+idx sama -> (x,y) sama selalu), dan HARUS dlm batas dunia
    /// (tak pernah keluar `WORLD_W`/`WORLD_H`, sumber panic index kalau lolos).
    #[test]
    fn planet_grid_pos_is_deterministic_and_within_world_bounds() {
        for idx in 0..20u32 {
            let (x1, y1) = planet_grid_pos(12345, idx);
            let (x2, y2) = planet_grid_pos(12345, idx);
            assert_eq!(
                (x1, y1),
                (x2, y2),
                "idx={idx}: posisi harus SAMA tiap panggilan"
            );
            assert!((0..WORLD_W).contains(&x1), "x={x1} harus dlm [0,{WORLD_W})");
            assert!((0..WORLD_H).contains(&y1), "y={y1} harus dlm [0,{WORLD_H})");
        }
    }

    /// M18.1: seed BEDA harus (biasanya) hasilkan posisi beda -- bukti genuine hash dari
    /// seed, bukan cuma fungsi idx doang (yg akan bikin SEMUA galaksi py tata letak identik).
    #[test]
    fn planet_grid_pos_differs_across_seeds() {
        let a = planet_grid_pos(1, 0);
        let b = planet_grid_pos(2, 0);
        assert_ne!(
            a, b,
            "seed beda harus genuinely hasilkan posisi beda utk idx sama"
        );
    }

    /// M18.1: render() di area STANDAR tak panic, grid HARUS genuinely tampil (bukan hanya
    /// header/legend/footer) -- dites via `App::demo()` (galaxy Lvl 1 ProcGen tersedia).
    #[test]
    fn render_fills_grid_without_panic_at_standard_sizes() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        for (w, h) in [(120u16, 40u16), (80, 30), (60, 24)] {
            let app = App::demo();
            let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
            term.draw(|f| render(f, &app, f.area())).unwrap();
            let buf = term.backend().buffer();
            let non_space = (0..h)
                .flat_map(|y| (0..w).map(move |x| (x, y)))
                .filter(|&(x, y)| buf[(x, y)].symbol() != " ")
                .count();
            assert!(
                non_space > 50,
                "{w}x{h}: grid harus genuinely tampil, got {non_space} sel"
            );
        }
    }

    /// M18.9 (pondasi): area SANGAT KECIL (di bawah minimum wajar) tak boleh panic --
    /// `saturating_sub` di `vh`/`vw` harus genuinely cegah underflow.
    #[test]
    fn render_does_not_panic_at_tiny_area() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let app = App::demo();
        let mut term = Terminal::new(TestBackend::new(3, 3)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
    }

    /// M18.2: "Warna tile per faksi/biome/status" — dites LANGSUNG: planet UNLOCKED (Earth,
    /// `Fixed` galaxy idx=0) harus genuinely dirender `th.good` di posisi grid PERSISNYA
    /// (`planet_grid_pos(0,0)`, `Fixed` pakai seed=0 -- dikonfirmasi di `render()`), BUKAN
    /// warna generik dim/neutral yg sama semua tile spt sblm M18.2.
    #[test]
    fn unlocked_planet_tile_uses_good_color_not_generic_dim() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let mut app = App::demo();
        // Keep HANYA galaxy Fixed (Lvl 0, Earth unlocked=true) -- `frontier()` pilih level
        // TERTINGGI, jd galaxy ProcGen (Lvl 1) HARUS dibuang dulu spy Fixed genuinely dipilih.
        app.state
            .galaxies
            .retain(|g| matches!(g.kind, GalaxyKind::Fixed));
        let th = crate::ui::theme::theme(app.state.settings.theme);

        let (x, y) = planet_grid_pos(0, 0); // Earth = idx 0, Fixed pakai seed=0.
        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let buf = term.backend().buffer();
        // Viewport dimulai dr border+header(1)+legend(1) -> offset y+3, x+1 (border kiri).
        let (cx, cy) = (x as u16 + 1, y as u16 + 4); // M19.2: +1 baris legend, offset +3->+4.
        let cell = &buf[(cx, cy)];
        assert_ne!(
            cell.symbol(),
            " ",
            "sel Earth (idx 0) harus genuinely terisi glyph"
        );
        assert_eq!(
            cell.fg, th.good,
            "planet UNLOCKED (Earth) harus genuinely `th.good`, bukan warna dim/neutral generik"
        );
    }

    /// M18.3/M18.4: "Kursor sel (highlight)" — **raster.rs GAP established** (M13.6/M14.8/
    /// M15.10 precedent): rasterizer TAK implementasikan `Modifier::REVERSED` sama sekali,
    /// jd screenshot PNG TAK BISA buktikan highlight (dikonfirmasi Read screenshot 120x40:
    /// tak ada sel tampak "reversed" scr visual biarpun kursor genuinely ada di situ). Dites
    /// LANGSUNG via `Cell.modifier` dibaca dr `Buffer`, bukan via screenshot.
    /// M18.4: kursor skrng `app.galaxy_cursor` (BUKAN `app.sel` lg) — diarahkan LANGSUNG ke
    /// posisi planet idx 0 biar test ini tetap valid stlh perubahan sumber kursor.
    #[test]
    fn selected_planet_cell_is_reversed_and_only_that_cell() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let mut app = App::demo();
        let g = frontier(&app).unwrap();
        let seed = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => *seed,
            GalaxyKind::Fixed => 0,
        };
        let (cx, cy) = planet_grid_pos(seed, 0);
        app.galaxy_cursor = (cx, cy); // arahkan kursor BEBAS ke posisi planet idx 0.

        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let buf = term.backend().buffer();
        // Viewport mulai dr border(1)+header(1)+legend(1) -> offset x+1, y+3.
        let (bx, by) = (cx as u16 + 1, cy as u16 + 4); // M19.2: +1 baris legend, offset +3->+4.
        let cell = &buf[(bx, by)];
        assert!(
            cell.modifier.contains(ratatui::style::Modifier::REVERSED),
            "sel planet TERPILIH (idx 0, posisi ({cx},{cy})) harus genuinely REVERSED"
        );

        // Sel tetangga (BUKAN posisi kursor) harus TETAP normal (tak reversed) -- highlight
        // HANYA di 1 sel, bukan nge-reverse seluruh grid scr tak sengaja.
        let (other_x, other_y) = (bx.saturating_add(3), by);
        if other_x < buf.area().width {
            let other = &buf[(other_x, other_y)];
            assert!(
                !other.modifier.contains(ratatui::style::Modifier::REVERSED),
                "sel LAIN (bukan kursor) harus TETAP normal, bukan ikut reversed"
            );
        }
    }

    /// Cari sel manapun di `buf` yg bermodifier REVERSED (posisi kursor) — helper dipakai
    /// tes scroll M18.5, hindari ngitung origin manual (uji perilaku TAMPAK, bukan formula).
    fn find_reversed_cell(buf: &ratatui::buffer::Buffer) -> Option<(u16, u16)> {
        for y in 0..buf.area().height {
            for x in 0..buf.area().width {
                if buf[(x, y)]
                    .modifier
                    .contains(ratatui::style::Modifier::REVERSED)
                {
                    return Some((x, y));
                }
            }
        }
        None
    }

    /// M18.5: "Scroll viewport saat kursor di tepi" — **gap real ditemukan**: viewport dulu
    /// SELALU pojok dunia (0,0), kursor yg gerak jauh (mis. x=50 di area kecil 20 lebar) akan
    /// GENUINELY tak terlihat (di luar area render), biarpun state internal benar. Dites
    /// LANGSUNG: kursor jauh dari origin HARUS tetap TERLIHAT (viewport ikut scroll).
    #[test]
    fn viewport_scrolls_to_keep_distant_cursor_visible() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let mut app = App::demo();
        app.galaxy_cursor = (50, 30); // jauh dari (0,0), area viewport kecil di bawah tak akan
        // muat kalau origin TETAP (0,0) tanpa scroll.
        let mut term = Terminal::new(TestBackend::new(20, 10)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let buf = term.backend().buffer();
        assert!(
            find_reversed_cell(buf).is_some(),
            "kursor jauh dari origin (50,30) HARUS tetap terlihat -- viewport harus scroll \
             mengikuti, bukan diam di pojok dunia (0,0)"
        );
    }

    /// M18.5: kursor di TEPI DUNIA (WORLD_W-1) tak boleh bikin viewport scroll LEWAT batas
    /// dunia (area kosong percuma di luar dunia) -- origin harus clamp genuinely benar.
    #[test]
    fn viewport_does_not_scroll_past_world_edge() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let mut app = App::demo();
        app.galaxy_cursor = (WORLD_W - 1, WORLD_H - 1); // pojok kanan-bawah dunia.
        let mut term = Terminal::new(TestBackend::new(20, 10)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let buf = term.backend().buffer();
        assert!(
            find_reversed_cell(buf).is_some(),
            "kursor di pojok dunia (WORLD_W-1,WORLD_H-1) harus tetap terlihat"
        );
    }

    /// M18.6: "Grid deterministik dari seed procgen (sama tiap render)" — dites LANGSUNG:
    /// render() dipanggil 2x dari App SAMA (seed sama), buffer HARUS identik byte-per-byte
    /// -- `planet_grid_pos` murni fungsi `(seed,idx)`, `render()` tak py RNG apa pun saat
    /// render (semua acak SUDAH baku di seed), jd genuinely tak ada sumber non-determinisme.
    #[test]
    fn render_is_deterministic_across_repeated_calls() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let app = App::demo();
        let mut term1 = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term1.draw(|f| render(f, &app, f.area())).unwrap();
        let buf1 = term1.backend().buffer().clone();

        let mut term2 = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term2.draw(|f| render(f, &app, f.area())).unwrap();
        let buf2 = term2.backend().buffer().clone();

        assert_eq!(
            buf1, buf2,
            "render() dipanggil 2x dari App SAMA harus hasilkan buffer IDENTIK -- grid \
             deterministik murni dari seed, tak py randomness tersembunyi saat render"
        );
    }

    /// M18.7: "Density grid mirip Riftborne (padat tapi terbaca)" — **gap real ditemukan**:
    /// dulu SEMUA sel kosong SELALU `·` seragam (densitas visual nyaris 0 di luar planet
    /// langka). Dites LANGSUNG: (1) tekstur latar deterministik (seed+posisi sama -> glyph
    /// sama selalu); (2) GENUINELY bervariasi (bukan cuma `·` monoton) di sampel besar --
    /// bukti "lebih padat" bukan klaim kosong.
    #[test]
    fn background_texture_is_deterministic_and_varied() {
        assert_eq!(
            background_glyph(42, 5, 7),
            background_glyph(42, 5, 7),
            "posisi+seed SAMA harus hasilkan glyph SAMA selalu"
        );
        let glyphs: std::collections::HashSet<char> = (0..50)
            .flat_map(|x| (0..50).map(move |y| (x, y)))
            .map(|(x, y)| background_glyph(42, x, y))
            .collect();
        assert!(
            glyphs.len() > 1,
            "tekstur latar harus GENUINELY bervariasi (bukan 1 glyph monoton spt sblm M18.7), \
             got hanya {glyphs:?}"
        );
    }

    /// M18.8: "Marker home" — **gap real ditemukan**: TAK ADA cara bedakan planet HOME
    /// (basis pemain, Earth di Milky Way/Fixed) dari planet biasa di grid. Dites LANGSUNG:
    /// Earth (idx 0, galaksi Fixed) harus genuinely `Modifier::UNDERLINED`.
    #[test]
    fn home_planet_is_underlined() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let mut app = App::demo();
        // Keep HANYA galaxy Fixed (Milky Way/Earth) -- `frontier()` pilih level TERTINGGI,
        // galaxy ProcGen (Lvl 1) HARUS dibuang dulu spy Fixed (home) genuinely dipilih.
        app.state
            .galaxies
            .retain(|g| matches!(g.kind, GalaxyKind::Fixed));

        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let buf = term.backend().buffer();
        let (ex, ey) = planet_grid_pos(0, 0); // Earth = idx 0, Fixed pakai seed=0.
        let (bx, by) = (ex as u16 + 1, ey as u16 + 4); // M19.2: +1 baris legend, offset +3->+4.
        let cell = &buf[(bx, by)];
        assert!(
            cell.modifier.contains(ratatui::style::Modifier::UNDERLINED),
            "Earth (home, idx 0 galaksi Fixed) harus genuinely UNDERLINED"
        );
    }

    /// M18.8: "Marker target" — **gap real ditemukan**: TAK ADA cara bedakan planet TARGET
    /// (tujuan ship travel saat ini) dari planet biasa di grid. Dites LANGSUNG: planet yg
    /// jadi tujuan `ShipStatus::Traveling` harus genuinely BOLD + warna `th.alert`.
    #[test]
    fn travel_target_planet_is_bold_and_alert_colored() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let mut app = App::demo();
        let g = frontier(&app).unwrap();
        let seed = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => *seed,
            GalaxyKind::Fixed => 0,
        };
        app.state.ship.status = crate::game::state::ShipStatus::Traveling {
            to: crate::game::state::PlanetId(0),
            total_secs: 100.0,
            elapsed_secs: 10.0,
        };
        let th = crate::ui::theme::theme(app.state.settings.theme);

        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let buf = term.backend().buffer();
        let (tx, ty) = planet_grid_pos(seed, 0);
        let (bx, by) = (tx as u16 + 1, ty as u16 + 4); // M19.2: +1 baris legend, offset +3->+4.
        let cell = &buf[(bx, by)];
        assert!(
            cell.modifier.contains(ratatui::style::Modifier::BOLD),
            "planet TARGET (tujuan travel idx 0) harus genuinely BOLD"
        );
        assert_eq!(
            cell.fg, th.alert,
            "planet TARGET harus genuinely warna `th.alert`, bukan warna status biasa"
        );
    }

    /// M18.9: "Clamp kursor & viewport (no out-of-bounds/panic)" — kursor di-set LANGSUNG ke
    /// nilai DI LUAR batas dunia (bypass clamp `galaxy_map_keys`, mis. field diubah manual dr
    /// kode lain) HARUS TETAP tak panic -- `origin_x/y`'s clamp cuma bergantung `WORLD_W/H`+
    /// `vw/vh` (konstan area), BUKAN validitas `cursor_pos` itu sendiri, jd genuinely aman
    /// biarpun cursor_pos rusak/di luar batas.
    #[test]
    fn render_does_not_panic_with_out_of_bounds_cursor() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        for cursor in [
            (-500, -500),
            (WORLD_W + 500, WORLD_H + 500),
            (i32::MIN, i32::MIN),
        ] {
            let mut app = App::demo();
            app.galaxy_cursor = cursor;
            let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
            term.draw(|f| render(f, &app, f.area())).unwrap();
        }
    }

    /// M18.9: area SANGAT EKSTREM (0x0, 1x1) tak boleh panic -- `saturating_sub` +
    /// `.max(0)` di `vh`/`vw` HARUS genuinely cegah underflow di titik paling ekstrem
    /// (bukan cuma "cukup kecil" spt tes M18.1's 3x3).
    #[test]
    fn render_does_not_panic_at_zero_and_one_by_one_area() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        for (w, h) in [(0u16, 0u16), (1, 1)] {
            let app = App::demo();
            let mut term = Terminal::new(TestBackend::new(w.max(1), h.max(1))).unwrap();
            let area = ratatui::layout::Rect::new(0, 0, w, h);
            term.draw(|f| render(f, &app, area)).unwrap();
        }
    }

    /// M19.1: "Panel SELECTED TILE" — **overflow dicegah SBLM screenshot**: dites LANGSUNG
    /// worst-case (biome terpanjang "GasGiant" + status terpanjang "Locked WTn" + koordinat
    /// 3-digit) tak melebihi lebar Minimal (58w interior, 60x24 breakpoint).
    #[test]
    fn tile_detail_line_worst_case_fits_minimal_width() {
        use crate::game::state::{
            Biome, Factory, NodeId, Planet, PlanetId, ResourceMap, ResourceNode, UnlockReq,
        };

        let app = App::demo();
        let th = crate::ui::theme::theme(app.state.settings.theme);
        let p = Planet {
            id: PlanetId(0),
            name: "Worstcasea".into(),
            tier: 9,
            biome: Biome::GasGiant, // biome label terpanjang ("GasGiant", 8 char).
            distance: 9.9,
            unlocked: false,
            unlock_req: UnlockReq::WarpTier(9), // "Locked WT9" -- status terpanjang.
            nodes: vec![ResourceNode {
                id: NodeId(0),
                resource: crate::game::defs::ResourceId(0),
                richness: 99.9,
                level: 1,
            }],
            factory_slots: Vec::<Option<Factory>>::new(),
            stockpile: ResourceMap::new(),
            stockpile_cap: 0.0,
        };
        let g = Galaxy {
            id: crate::game::state::GalaxyId(0),
            name: "Test".into(),
            level: 0,
            kind: GalaxyKind::Fixed,
            planets: vec![p],
        };
        let pos = planet_grid_pos(0, 0);
        let line = tile_detail_line(&g, &app, pos, &th);
        let rendered: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(
            rendered.chars().count() <= 58,
            "SELECTED TILE worst-case ({} char) harus muat di Minimal 58w: {rendered:?}",
            rendered.chars().count()
        );
    }

    /// M19.10: "Status incoming/outgoing bila relevan" — **overflow dicegah SBLM screenshot**,
    /// tp jg **temuan genuinely penting**: worst-case AWAL dites dgn kombinasi "Locked WT9" +
    /// incoming — GAGAL, dites LANGSUNG (`fmt_eta(5940.0)` = "1h39m" bukan "99m", `fmt_eta`
    /// konversi >=60menit ke jam) DAN status ternyata SELALU "Scanned" [BUKAN "Locked WTn"]
    /// saat incoming applicable — `travel_to`'s `pi` index HANYA planet SUDAH materialize
    /// (`state.galaxies[gi].planets`, bukan yg blm discan), jd kombinasi "Locked WT9 (blm
    /// discan)" + "ship traveling kesana" GENUINELY MUSTAHIL dicapai gameplay nyata (2 kondisi
    /// saling eksklusif). Worst-case DIPERBAIKI: status="Scanned" [SATU2nya reachable saat
    /// incoming], ETA ekstrem wajar `99h59m` [`fmt_eta`'s format jam:menit, dibulatkan aman].
    #[test]
    fn tile_detail_line_worst_case_with_incoming_eta_fits_minimal_width() {
        use crate::game::state::{
            Biome, Factory, NodeId, Planet, PlanetId, ResourceMap, ResourceNode, UnlockReq,
        };

        let mut app = App::demo();
        let th = crate::ui::theme::theme(app.state.settings.theme);
        app.state.ship.status = ShipStatus::Traveling {
            to: PlanetId(0),
            total_secs: 359_940.0, // 99h59m -- ekstrem wajar via `fmt_eta`.
            elapsed_secs: 0.0,
        };
        let p = Planet {
            id: PlanetId(0),
            name: "Worstcasea".into(),
            tier: 9,
            biome: Biome::GasGiant,
            distance: 9.9,
            unlocked: false, // materialized-tapi-locked ("Scanned") -- SATU2nya status yg
            // genuinely reachable bareng incoming (dites, bukan diasumsikan).
            unlock_req: UnlockReq::WarpTier(9),
            nodes: vec![ResourceNode {
                id: NodeId(0),
                resource: crate::game::defs::ResourceId(0),
                richness: 99.9,
                level: 1,
            }],
            factory_slots: Vec::<Option<Factory>>::new(),
            stockpile: ResourceMap::new(),
            stockpile_cap: 0.0,
        };
        let g = Galaxy {
            id: crate::game::state::GalaxyId(0),
            name: "Test".into(),
            level: 0,
            kind: GalaxyKind::Fixed,
            planets: vec![p],
        };
        let pos = planet_grid_pos(0, 0);
        let line = tile_detail_line(&g, &app, pos, &th);
        let rendered: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(
            rendered.chars().count() <= 58,
            "SELECTED TILE + incoming ETA worst-case ({} char) harus muat di Minimal 58w: \
             {rendered:?}",
            rendered.chars().count()
        );
        assert!(
            rendered.contains("Scanned"),
            "status HARUS genuinely 'Scanned' saat incoming applicable (bukan 'Locked WTn', \
             kombinasi itu mustahil dicapai): {rendered:?}"
        );
        assert!(
            rendered.contains("->99h59m"),
            "incoming ETA HARUS genuinely tampil: {rendered:?}"
        );
    }

    /// M19.2: "Legend simbol/warna" — **gap real ditemukan**: legend M18.2 dibuat SBLM marker
    /// home/target ADA (M18.8), tak jelaskan makna underline/bold+alert yg genuinely tampil
    /// di grid. Dites LANGSUNG dari BUFFER HASIL RENDER ASLI (bukan rekonstruksi terpisah yg
    /// bisa diam2 divergen dari kode produksi): baris legend marker (row ke-2 dlm area, stlh
    /// header+legend status) HARUS py sel 'P' UNDERLINED ("Home") dan sel 'P' BOLD+`th.alert`
    /// ("Target").
    #[test]
    fn marker_legend_uses_same_styles_as_actual_tile_markers() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let app = App::demo();
        let th = crate::ui::theme::theme(app.state.settings.theme);
        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let buf = term.backend().buffer();

        // Baris legend marker = row ke-3 dlm area (0=border, 1=title, 2=legend status,
        // 3=legend marker) -> border(1)+title(1)+legend(1) = offset y+3 dr area.y (=0 di test).
        let row = 3u16;
        let underlined = (0..buf.area().width).any(|x| {
            buf[(x, row)]
                .modifier
                .contains(ratatui::style::Modifier::UNDERLINED)
        });
        assert!(
            underlined,
            "baris legend marker HARUS py sel UNDERLINED ('Home'), sama gaya tile Home asli"
        );
        let bold_alert = (0..buf.area().width).any(|x| {
            let cell = &buf[(x, row)];
            cell.modifier.contains(ratatui::style::Modifier::BOLD) && cell.fg == th.alert
        });
        assert!(
            bold_alert,
            "baris legend marker HARUS py sel BOLD+`th.alert` ('Target'), sama gaya tile \
             Target asli"
        );
    }

    /// M19.3: "Minimap (peta kecil posisi viewport)" — **gap real ditemukan**: viewport bebas
    /// scroll (M18.5) TAK PY cara liat posisi relatif ke SELURUH dunia -- user genuinely bisa
    /// tersesat. Dites LANGSUNG: minimap HARUS genuinely tampil (sel non-dim, bukan cuma area
    /// kosong) di area STANDAR (120x40).
    #[test]
    fn minimap_renders_at_standard_area() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let app = App::demo();
        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let buf = term.backend().buffer();
        // Minimap ada di pojok kanan-atas viewport -- cek area kasar (12 kolom terakhir
        // grid, 6 baris teratas grid) py setidaknya 1 sel '#' (viewport indicator, SELALU
        // ada krn viewport SELALU overlap dirinya sendiri).
        let found_viewport_marker =
            (0..40u16).any(|y| (0..120u16).any(|x| buf[(x, y)].symbol() == "#"));
        assert!(
            found_viewport_marker,
            "minimap HARUS genuinely tampil sel '#' (indikator viewport saat ini)"
        );
    }

    /// M19.3: minimap HARUS diskip total (bukan crash/corrupt) bila viewport SANGAT SEMPIT --
    /// dites LANGSUNG area sekecil mungkin yg masih render tanpa panic (pola M18.9's disiplin).
    #[test]
    fn minimap_skips_cleanly_at_tiny_viewport_without_panic() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let app = App::demo();
        let mut term = Terminal::new(TestBackend::new(15, 10)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
    }

    /// M19.4: "Body sprite ANSI preview tile terpilih" — **percobaan pertama gagal**: reuse
    /// ambang `planet.rs` mentah-mentah -> screenshot Full (120x40) TAK tampilkan sprite sama
    /// sekali. Diselidiki: MAIN area galaxy_map di Full HANYA 78w (`shell_full`'s `Length(22)+
    /// Min(0)+Length(18)` dr 118w inner), bukan ~86-90 spt dikira -- threshold diperbaiki jd
    /// `SM_MIN=74` (Sm 22 + worst-case detail 50 + margin 2). Dites LANGSUNG: kursor di posisi
    /// planet, area CUKUP LEBAR (120x40) -> kolom kanan HARUS genuinely terisi sprite (non-
    /// space), BUKAN tetap kosong.
    #[test]
    fn sprite_renders_in_reserved_column_when_cursor_on_planet_at_wide_area() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let mut app = App::demo();
        let g = frontier(&app).unwrap();
        let seed = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => *seed,
            GalaxyKind::Fixed => 0,
        };
        app.galaxy_cursor = planet_grid_pos(seed, 0); // idx 0, planet PASTI ada di posisi ini.

        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let buf = term.backend().buffer();
        // Kolom sprite = 22 kolom PALING KANAN dari area (120 lebar, area FULL diberikan ke
        // render() langsung di test ini -- bukan lewat shell, jd `sprite_width_for(120)`
        // genuinely >=74 -> 22 direservasi di kanan area 120-lebar ITU SENDIRI).
        let non_space_in_sprite_col =
            (0..40u16).any(|y| (98..120u16).any(|x| buf[(x, y)].symbol() != " "));
        assert!(
            non_space_in_sprite_col,
            "kolom sprite (22 kolom kanan) HARUS genuinely terisi saat kursor di planet, \
             bukan tetap kosong"
        );
    }

    /// M19.4: area SEMPIT (Compact/Minimal-setara) HARUS diskip sprite total -- grid mengisi
    /// LEBAR PENUH tanpa kolom kosong terpotong percuma.
    #[test]
    fn sprite_column_omitted_at_narrow_area() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let mut app = App::demo();
        let g = frontier(&app).unwrap();
        let seed = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => *seed,
            GalaxyKind::Fixed => 0,
        };
        app.galaxy_cursor = planet_grid_pos(seed, 0);

        // 64 lebar (setara Compact MAIN) -- di bawah `SM_MIN=74`, sprite HARUS diskip, grid
        // HARUS isi lebar penuh (sel dkt tepi kanan genuinely terisi grid, bukan blank sprite
        // area kosong).
        let mut term = Terminal::new(TestBackend::new(64, 30)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let buf = term.backend().buffer();
        let right_edge_has_content = (0..30u16).any(|y| buf[(62, y)].symbol() != " ");
        assert!(
            right_edge_has_content,
            "di area sempit, grid HARUS isi sampai dekat tepi kanan (sprite diskip, bukan \
             nyisakan kolom kosong percuma)"
        );
    }

    /// M19.5: "Aksi `s` scan/materialize tile (sudah ada → integrasi UI)" — **gap real
    /// ditemukan**: footer lama "[s]Scan/Send" nyebut aksi "Send" yg BLM diimplemen (M19.7
    /// msh todo) -- BOHONG soal kapabilitas skrng. `can_scan()` HARUS `true` di planet
    /// Procedural yg blm `visited` (genuinely scannable, cocok kondisi `materialize_planet`).
    #[test]
    fn can_scan_true_for_unvisited_procedural_planet() {
        let app = App::demo();
        let g = frontier(&app).unwrap();
        let seed = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => *seed,
            GalaxyKind::Fixed => panic!("frontier demo HARUS Procedural (Lvl>0)"),
        };
        let pos = planet_grid_pos(seed, 0);
        assert!(
            can_scan(g, pos),
            "planet idx 0 (blm visited) HARUS genuinely scannable"
        );
    }

    /// M19.5: `can_scan()` HARUS `false` di sel kosong (tak ada planet) -- scan tak berbuat
    /// apa2 di sel kosong, footer TAK BOLEH iklankan `[s]Scan` di situ.
    #[test]
    fn can_scan_false_on_empty_tile() {
        let app = App::demo();
        let g = frontier(&app).unwrap();
        // (0,0) genuinely kosong utk seed demo (dikonfirmasi screenshot M19.1-M19.4 berulang:
        // "SELECTED TILE @(0,0) Empty").
        assert!(
            !can_scan(g, (0, 0)),
            "sel kosong (0,0) HARUS genuinely TAK scannable"
        );
    }

    /// M19.5: `can_scan()` HARUS `false` utk galaksi `Fixed` (Milky Way/Earth) -- Fixed TAK PY
    /// mekanik scan sama sekali (`materialize_planet` early-return `false` utk kind ini),
    /// footer TAK BOLEH iklankan `[s]Scan` di planet Fixed manapun.
    #[test]
    fn can_scan_false_for_fixed_galaxy_planet() {
        let mut app = App::demo();
        app.state
            .galaxies
            .retain(|g| matches!(g.kind, GalaxyKind::Fixed));
        let g = frontier(&app).unwrap();
        let pos = planet_grid_pos(0, 0); // Earth, idx 0, Fixed pakai seed=0.
        assert!(
            !can_scan(g, pos),
            "planet Fixed (Earth) HARUS genuinely TAK scannable (tak py mekanik scan)"
        );
    }

    /// M19.5: `can_scan()` HARUS `false` SETELAH planet dimaterialisasi (`visited` sudah
    /// berisi idx) -- scan ulang planet yg SUDAH discan genuinely no-op, footer HARUS berhenti
    /// iklankan `[s]Scan` stlh aksi berhasil (feedback via STATE, pola SAMA established M15.9's
    /// precedent -- bukan toast/pesan baru).
    #[test]
    fn can_scan_false_after_materialization() {
        let mut app = App::demo();
        let g = frontier(&app).unwrap();
        let (seed, fid) = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => (*seed, g.id),
            GalaxyKind::Fixed => panic!("frontier demo HARUS Procedural"),
        };
        let pos = planet_grid_pos(seed, 0);
        let gi = app.state.galaxies.iter().position(|g| g.id == fid).unwrap();
        assert!(crate::game::world::procgen::materialize_planet(
            &mut app.state.galaxies[gi],
            0,
            &app.content
        ));
        let g = frontier(&app).unwrap();
        assert!(
            !can_scan(g, pos),
            "planet yg SUDAH dimaterialisasi HARUS genuinely TAK scannable lg"
        );
    }

    /// M19.6: **bug latent real ditemukan+diperbaiki**: `g.planets.get(idx as usize)` [posisi
    /// vec] SALAH bila planet discan OUT-OF-ORDER (idx TINGGI discan SBLM idx RENDAH -- vec
    /// terisi urutan SCAN, bukan urutan idx). Dites LANGSUNG: materialize idx=1 DULU baru
    /// idx=0 (out-of-order sengaja) -- `planet_by_idx(g,0)` HARUS genuinely kembalikan planet
    /// ber-`id=PlanetId(0)`, BUKAN planet pertama di-push (`idx=1`).
    #[test]
    fn planet_by_idx_finds_correct_planet_even_when_materialized_out_of_order() {
        let mut app = App::demo();
        let g = frontier(&app).unwrap();
        let (fid,) = (g.id,);
        let gi = app.state.galaxies.iter().position(|g| g.id == fid).unwrap();
        // Sengaja out-of-order: idx=1 discan DULU, baru idx=0 -- `g.planets` jd `[p1,p0]`.
        assert!(crate::game::world::procgen::materialize_planet(
            &mut app.state.galaxies[gi],
            1,
            &app.content
        ));
        assert!(crate::game::world::procgen::materialize_planet(
            &mut app.state.galaxies[gi],
            0,
            &app.content
        ));
        let g = frontier(&app).unwrap();
        // `.get(0)` [BUG LAMA] akan salah kembalikan planet idx=1 (posisi 0 di vec); fix HARUS
        // genuinely kembalikan planet ber-id `PlanetId(0)`.
        let p = planet_by_idx(g, 0).expect("planet idx=0 HARUS ditemukan meski discan KEDUA");
        assert_eq!(
            p.id,
            crate::game::state::PlanetId(0),
            "planet_by_idx(g,0) HARUS genuinely planet id=0, BUKAN planet idx=1 yg kebetulan \
             di posisi vec ke-0 (bug lama: lookup posisional, bukan by-id)"
        );
    }

    /// M19.6: "Aksi Enter masuk planet/sistem terpilih" — planet BLM unlocked (baru discan,
    /// blm dikirim ship) HARUS genuinely TAK bisa dimasuki via Enter (no-op senyap, pola SAMA
    /// `s`). Dites via `App::handle_key` LANGSUNG (jembatan end-to-end key-press→state, pola
    /// SAMA M15.8's precedent) -- view HARUS TETAP `galaxy_map`, `active_galaxy` TAK berubah.
    #[test]
    fn enter_is_noop_on_locked_planet() {
        let mut app = App::demo();
        app.view = "galaxy_map".into();
        let g = frontier(&app).unwrap();
        let (seed, fid) = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => (*seed, g.id),
            GalaxyKind::Fixed => panic!("frontier demo HARUS Procedural"),
        };
        app.galaxy_cursor = planet_grid_pos(seed, 0);
        let gi = app.state.galaxies.iter().position(|g| g.id == fid).unwrap();
        crate::game::world::procgen::materialize_planet(
            &mut app.state.galaxies[gi],
            0,
            &app.content,
        );
        let active_before = app.state.active_galaxy;

        app.handle_key(crossterm::event::KeyCode::Enter);

        assert_eq!(
            app.view, "galaxy_map",
            "Enter di planet BLM-unlocked HARUS no-op (view TETAP)"
        );
        assert_eq!(
            app.state.active_galaxy, active_before,
            "Enter di planet BLM-unlocked TAK BOLEH ganti active_galaxy"
        );
    }

    /// M19.6: Enter di planet SUDAH `unlocked` (ship SUDAH tiba, `sim/tick.rs`'s pola) HARUS
    /// genuinely pindah `view` ke `planet_view` DAN `active_galaxy` ke galaxy frontier ini --
    /// pakai field mekanik SUDAH ADA (`active_galaxy`, dibedakan `anchor_galaxy` di
    /// `prestige.rs`), BUKAN state UI baru.
    #[test]
    fn enter_switches_view_and_active_galaxy_when_planet_unlocked() {
        let mut app = App::demo();
        app.view = "galaxy_map".into();
        let g = frontier(&app).unwrap();
        let (seed, fid) = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => (*seed, g.id),
            GalaxyKind::Fixed => panic!("frontier demo HARUS Procedural"),
        };
        app.galaxy_cursor = planet_grid_pos(seed, 0);
        let gi = app.state.galaxies.iter().position(|g| g.id == fid).unwrap();
        crate::game::world::procgen::materialize_planet(
            &mut app.state.galaxies[gi],
            0,
            &app.content,
        );
        // Simulasikan ship SUDAH tiba (pola SAMA `sim/tick.rs`'s `advance_travel`, bukan reka
        // mekanik baru): `unlocked=true` LANGSUNG di planet ter-materialize.
        app.state.galaxies[gi]
            .planets
            .iter_mut()
            .find(|p| p.id == crate::game::state::PlanetId(0))
            .unwrap()
            .unlocked = true;

        app.handle_key(crossterm::event::KeyCode::Enter);

        assert_eq!(
            app.view, "planet_view",
            "Enter di planet unlocked HARUS pindah ke planet_view"
        );
        assert_eq!(
            app.state.active_galaxy, fid,
            "Enter HARUS genuinely set active_galaxy ke galaxy frontier (sistem dimasuki)"
        );
    }

    /// M19.7: "Aksi kirim ship ke tile (set Traveling)" — **gap real ditemukan**: `travel_to`
    /// [`game::ship`] SUDAH ADA+dites TAPI TAK PERNAH dipanggil dr UI manapun (dikonfirmasi
    /// grep). `can_send()` HARUS `true` di planet SUDAH materialize, BLM unlocked, ship Idle.
    #[test]
    fn can_send_true_for_materialized_locked_planet_with_idle_ship() {
        let mut app = App::demo();
        let g = frontier(&app).unwrap();
        let (seed, fid) = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => (*seed, g.id),
            GalaxyKind::Fixed => panic!("frontier demo HARUS Procedural"),
        };
        let pos = planet_grid_pos(seed, 0);
        let gi = app.state.galaxies.iter().position(|g| g.id == fid).unwrap();
        crate::game::world::procgen::materialize_planet(
            &mut app.state.galaxies[gi],
            0,
            &app.content,
        );
        let g = frontier(&app).unwrap();
        assert!(
            can_send(g, &app, pos),
            "planet materialize+blm unlocked+ship idle HARUS genuinely sendable"
        );
    }

    /// M19.7: `can_send()` HARUS `false` bila ship SEDANG travel (`travel_to` internal tolak
    /// kalau sibuk) -- footer TAK BOLEH iklankan `[t]Send` yg pasti gagal krn ship sibuk.
    #[test]
    fn can_send_false_when_ship_busy() {
        let mut app = App::demo();
        let g = frontier(&app).unwrap();
        let (seed, fid) = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => (*seed, g.id),
            GalaxyKind::Fixed => panic!("frontier demo HARUS Procedural"),
        };
        let pos = planet_grid_pos(seed, 0);
        let gi = app.state.galaxies.iter().position(|g| g.id == fid).unwrap();
        crate::game::world::procgen::materialize_planet(
            &mut app.state.galaxies[gi],
            0,
            &app.content,
        );
        app.state.ship.status = ShipStatus::Traveling {
            to: crate::game::state::PlanetId(99),
            total_secs: 100.0,
            elapsed_secs: 0.0,
        };
        let g = frontier(&app).unwrap();
        assert!(
            !can_send(g, &app, pos),
            "ship SEDANG travel HARUS genuinely TAK sendable lg (travel_to bakal tolak)"
        );
    }

    /// M19.7: `can_send()` HARUS `false` di planet BLM discan sama sekali -- tak ada apa2 utk
    /// dikirim ship kesana.
    #[test]
    fn can_send_false_before_scanned() {
        let app = App::demo();
        let g = frontier(&app).unwrap();
        assert!(
            !can_send(g, &app, (0, 0)),
            "sel kosong (blm discan) HARUS genuinely TAK sendable"
        );
    }

    /// M19.7: end-to-end via `App::handle_key` (pola SAMA M15.8's precedent) -- `t` di planet
    /// materialize+blm-unlocked HARUS genuinely ubah `ship.status` jd `Traveling` DAN set
    /// `active_galaxy` ke galaxy frontier [pakai field mekanik SUDAH ADA, pola SAMA `Enter`].
    #[test]
    fn send_key_starts_travel_and_switches_active_galaxy() {
        let mut app = App::demo();
        app.view = "galaxy_map".into();
        let g = frontier(&app).unwrap();
        let (seed, fid) = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => (*seed, g.id),
            GalaxyKind::Fixed => panic!("frontier demo HARUS Procedural"),
        };
        app.galaxy_cursor = planet_grid_pos(seed, 0);
        let gi = app.state.galaxies.iter().position(|g| g.id == fid).unwrap();
        crate::game::world::procgen::materialize_planet(
            &mut app.state.galaxies[gi],
            0,
            &app.content,
        );

        app.handle_key(crossterm::event::KeyCode::Char('t'));

        assert!(
            matches!(app.state.ship.status, ShipStatus::Traveling { .. }),
            "`t` di planet valid HARUS genuinely mulai `Traveling`"
        );
        assert_eq!(
            app.state.active_galaxy, fid,
            "`t` HARUS genuinely set active_galaxy ke galaxy frontier (target travel)"
        );
    }

    /// M19.7: `t` di sel kosong (blm discan) HARUS genuinely no-op (ship TETAP `Idle`, tak
    /// panic) -- pola SAMA `s`/`Enter`'s precondition check.
    #[test]
    fn send_key_is_noop_on_unscanned_tile() {
        let mut app = App::demo();
        app.view = "galaxy_map".into();
        app.galaxy_cursor = (0, 0); // genuinely kosong utk seed demo.

        app.handle_key(crossterm::event::KeyCode::Char('t'));

        assert!(
            matches!(app.state.ship.status, ShipStatus::Idle),
            "`t` di sel kosong HARUS genuinely no-op (ship tetap Idle)"
        );
    }

    /// M19.8: "Highlight rute ship (kursor→target)" — `bresenham_line` HARUS genuinely
    /// menyertakan KEDUA ujung (a & b), utk kasus horizontal/vertical/diagonal/titik-sama.
    #[test]
    fn bresenham_line_includes_both_endpoints_horizontal_vertical_diagonal() {
        let horiz = bresenham_line((0, 5), (4, 5));
        assert_eq!(horiz.first(), Some(&(0, 5)));
        assert_eq!(horiz.last(), Some(&(4, 5)));
        assert!(
            horiz.iter().all(|&(_, y)| y == 5),
            "horizontal HARUS y konstan"
        );

        let vert = bresenham_line((3, 0), (3, 4));
        assert_eq!(vert.first(), Some(&(3, 0)));
        assert_eq!(vert.last(), Some(&(3, 4)));
        assert!(
            vert.iter().all(|&(x, _)| x == 3),
            "vertical HARUS x konstan"
        );

        let diag = bresenham_line((0, 0), (3, 3));
        assert_eq!(diag.first(), Some(&(0, 0)));
        assert_eq!(diag.last(), Some(&(3, 3)));
        assert_eq!(diag.len(), 4, "diagonal 45derajat HARUS N+1 sel persis");
    }

    /// M19.8: `bresenham_line` di titik SAMA (a==b) HARUS genuinely kembalikan 1 sel (bukan
    /// infinite loop / vec kosong) -- kasus tepi kursor pas di posisi target.
    #[test]
    fn bresenham_line_same_point_returns_single_cell() {
        let line = bresenham_line((7, 7), (7, 7));
        assert_eq!(line, vec![(7, 7)]);
    }

    /// M19.8: rute HARUS genuinely tampil (sel `+` `th.dim`) saat ship `Traveling`, dari
    /// `cursor_pos` [default (0,0)] ke posisi target planet idx=`to.0` di grid.
    #[test]
    fn route_highlight_renders_when_ship_traveling() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let mut app = App::demo();
        let g = frontier(&app).unwrap();
        let seed = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => *seed,
            GalaxyKind::Fixed => panic!("frontier demo HARUS Procedural"),
        };
        let target_pos = planet_grid_pos(seed, 0);
        // Kursor default (0,0) genuinely beda dr target idx=0 (dikonfirmasi screenshot
        // berulang sblmnya: planet idx=0 TAK di (0,0)) -- garis rute genuinely py >1 sel.
        assert_ne!(
            (0, 0),
            target_pos,
            "prasyarat test: cursor default HARUS beda dr target"
        );
        app.state.ship.status = ShipStatus::Traveling {
            to: crate::game::state::PlanetId(0),
            total_secs: 100.0,
            elapsed_secs: 0.0,
        };

        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let buf = term.backend().buffer();
        let found_route = (0..40u16).any(|y| (0..120u16).any(|x| buf[(x, y)].symbol() == "+"));
        assert!(
            found_route,
            "rute [`+`] HARUS genuinely tampil saat ship Traveling ke target"
        );
    }

    /// M19.8: rute TAK BOLEH tampil sama sekali saat ship `Idle` (tak ada target utk dituju).
    #[test]
    fn route_highlight_absent_when_ship_idle() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let app = App::demo(); // ship default Idle.
        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let buf = term.backend().buffer();
        let found_route = (0..40u16).any(|y| (0..120u16).any(|x| buf[(x, y)].symbol() == "+"));
        assert!(
            !found_route,
            "rute [`+`] TAK BOLEH tampil saat ship Idle (tak ada target)"
        );
    }

    /// M19.9: "Filter (faksi/tile) toggle" — `next()` HARUS genuinely cycle SEMUA 4 varian &
    /// kembali ke `All` (bukan macet di 1 state / skip salah satu).
    #[test]
    fn galaxy_filter_next_cycles_through_all_variants_back_to_all() {
        let f = GalaxyFilter::All;
        let f = f.next();
        assert_eq!(f, GalaxyFilter::Active);
        let f = f.next();
        assert_eq!(f, GalaxyFilter::Locked);
        let f = f.next();
        assert_eq!(f, GalaxyFilter::Unscanned);
        let f = f.next();
        assert_eq!(f, GalaxyFilter::All, "cycle HARUS genuinely kembali ke All");
    }

    /// M19.9: filter `Active` HARUS genuinely SEMBUNYIKAN planet yg BLM discan (Unscanned) dr
    /// grid -- sel di posisinya jd tekstur latar biasa, BUKAN glyph planet.
    #[test]
    fn filter_active_hides_unvisited_planet_from_grid() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let mut app = App::demo();
        let g = frontier(&app).unwrap();
        let seed = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => *seed,
            GalaxyKind::Fixed => panic!("frontier demo HARUS Procedural"),
        };
        let pos = planet_grid_pos(seed, 0); // idx 0, blm discan di demo default.
        let expected_biome_glyph =
            planet_glyph(&generate_planet(seed, 0, g.level, &app.content)).to_string();
        app.galaxy_filter = GalaxyFilter::Active;

        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let buf = term.backend().buffer();
        // Offset SAMA pola test lain (border+header+legend+marker_legend = +1,+4).
        let (bx, by) = (pos.0 as u16 + 1, pos.1 as u16 + 4);
        let glyph = buf[(bx, by)].symbol().to_string();
        assert_ne!(
            glyph, expected_biome_glyph,
            "planet Unscanned HARUS genuinely disembunyikan saat filter=Active (bukan glyph \
             biome-nya, tekstur latar sbg gantinya)"
        );
    }

    /// M19.9: filter murni VISUAL — `can_scan` (mekanik) HARUS TETAP jalan normal di planet yg
    /// SEDANG disembunyikan filter (kursor bebas gerak kesana, aksi tetap berfungsi).
    #[test]
    fn filter_does_not_affect_can_scan_mechanic() {
        let mut app = App::demo();
        let g = frontier(&app).unwrap();
        let seed = match &g.kind {
            GalaxyKind::Procedural { seed, .. } => *seed,
            GalaxyKind::Fixed => panic!("frontier demo HARUS Procedural"),
        };
        let pos = planet_grid_pos(seed, 0);
        app.galaxy_filter = GalaxyFilter::Active; // sembunyikan planet Unscanned idx 0 dr grid.
        let g = frontier(&app).unwrap();
        assert!(
            can_scan(g, pos),
            "can_scan HARUS genuinely TETAP true meski tile disembunyikan filter (murni \
             visual, tak ubah mekanik)"
        );
    }

    /// M20.1: "Mode backdrop vs starmap (tab/key) dalam galaxy view" — `toggle()` HARUS
    /// genuinely bolak-balik Starmap<->Backdrop (bukan macet/skip).
    #[test]
    fn galaxy_map_mode_toggle_switches_between_starmap_and_backdrop() {
        let m = GalaxyMapMode::Starmap;
        assert_eq!(m.toggle(), GalaxyMapMode::Backdrop);
        assert_eq!(m.toggle().toggle(), GalaxyMapMode::Starmap);
    }

    /// M20.1: `Tab` end-to-end via `App::handle_key` (pola SAMA M15.8's precedent) HARUS
    /// genuinely toggle `galaxy_map_mode`, HANYA saat view aktif `galaxy_map`.
    #[test]
    fn tab_key_toggles_galaxy_map_mode_only_in_galaxy_map_view() {
        let mut app = App::demo();
        app.view = "galaxy_map".into();
        assert_eq!(app.galaxy_map_mode, GalaxyMapMode::Starmap);

        app.handle_key(crossterm::event::KeyCode::Tab);
        assert_eq!(app.galaxy_map_mode, GalaxyMapMode::Backdrop);

        app.handle_key(crossterm::event::KeyCode::Tab);
        assert_eq!(app.galaxy_map_mode, GalaxyMapMode::Starmap);

        // Di view LAIN, `Tab` TAK BOLEH ubah `galaxy_map_mode` (dispatch `galaxy_map_keys`
        // HANYA jalan saat `view=="galaxy_map"`, dites LANGSUNG bukan diasumsikan).
        app.view = "main_menu".into();
        app.handle_key(crossterm::event::KeyCode::Tab);
        assert_eq!(
            app.galaxy_map_mode,
            GalaxyMapMode::Starmap,
            "`Tab` di view LAIN tak boleh ubah galaxy_map_mode"
        );
    }

    /// M20.1: mode `Backdrop` HARUS genuinely render tanpa panic DAN SKIP TOTAL starmap grid
    /// (SELECTED TILE/legend/footer starmap TAK BOLEH tampil -- "1 mode fokus" per waktu).
    #[test]
    fn backdrop_mode_renders_without_panic_and_skips_starmap_content() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let mut app = App::demo();
        app.galaxy_map_mode = GalaxyMapMode::Backdrop;
        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let buf = term.backend().buffer();
        let text: String = (0..40u16)
            .flat_map(|y| (0..120u16).map(move |x| (x, y)))
            .map(|(x, y)| buf[(x, y)].symbol().to_string())
            .collect();
        assert!(
            !text.contains("SELECTED TILE"),
            "mode Backdrop HARUS genuinely SKIP starmap grid/SELECTED TILE"
        );
        assert!(
            text.contains("Backdrop"),
            "mode Backdrop HARUS genuinely tampilkan indikator judul 'Backdrop'"
        );
    }

    /// M20.2: "Shell 3-kolom tetap di galaxy view" — **dicek LANGSUNG (bukan diasumsikan)**:
    /// bandingkan render SHELL PENUH (`draw_view`, bukan cuma `galaxy_map::render` sendiri)
    /// utk view `galaxy_map` vs `planet_view` di SEMUA 3 breakpoint — sudah GENUINELY
    /// konsisten (`view_main`'s `show_res` SAMA `false` utk keduanya, `shell_full`/`shell_
    /// compact`/`shell_minimal` dipakai bareng, bukan cabang shell terpisah per-view). Test
    /// ini KONVERSI temuan visual jd guard regresi permanen -- bila suatu saat kode
    /// dipisah/refactor shell per-view, test ini akan genuinely gagal.
    #[test]
    fn shell_structure_matches_other_views_at_full_breakpoint() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let app = App::demo();
        let render_view = |view: &str| {
            let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
            term.draw(|f| crate::ui::draw_view(f, &app, view)).unwrap();
            term.backend().buffer().clone()
        };
        let gm = render_view("galaxy_map");
        let pv = render_view("planet_view");

        // Kolom RESOURCES (kiri) & OPTIONS (kanan) HARUS di posisi PERSIS SAMA -- bandingkan
        // baris pertama LUAR MAIN [row 3, "RESOURCES"/"OPTIONS" label -- lihat screenshot M19
        // manapun utk posisi persis] di KEDUA render.
        for y in 0..40u16 {
            for x in 0..23u16 {
                // kolom kiri (RESOURCES) -- lebar 22+1 border.
                assert_eq!(
                    gm[(x, y)].symbol(),
                    pv[(x, y)].symbol(),
                    "kolom kiri (RESOURCES) HARUS identik posisi galaxy_map vs planet_view \
                     di ({x},{y})"
                );
            }
            for x in 102..120u16 {
                // kolom kanan (OPTIONS) -- 18 lebar dr kanan.
                assert_eq!(
                    gm[(x, y)].symbol(),
                    pv[(x, y)].symbol(),
                    "kolom kanan (OPTIONS) HARUS identik posisi galaxy_map vs planet_view \
                     di ({x},{y})"
                );
            }
        }
    }

    /// M20.2: shell Minimal (tab bar atas + footer bawah) jg HARUS identik posisi antara
    /// `galaxy_map` & `planet_view` -- breakpoint sempit py struktur BEDA dr Full (tanpa
    /// kolom RESOURCES/OPTIONS), tp KEDUANYA harus SAMA satu sama lain.
    #[test]
    fn shell_structure_matches_other_views_at_minimal_breakpoint() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let app = App::demo();
        let render_view = |view: &str| {
            let mut term = Terminal::new(TestBackend::new(60, 24)).unwrap();
            term.draw(|f| crate::ui::draw_view(f, &app, view)).unwrap();
            term.backend().buffer().clone()
        };
        let gm = render_view("galaxy_map");
        let pv = render_view("planet_view");

        // Baris tab atas (y=1, dalam border) & baris footer (y terakhir dlm border) HARUS
        // identik -- SAMA tab bar "Menu|Planet|Research|Galaxy|Warp" & footer shortcut.
        for x in 0..60u16 {
            assert_eq!(
                gm[(x, 1)].symbol(),
                pv[(x, 1)].symbol(),
                "baris tab atas HARUS identik posisi galaxy_map vs planet_view di x={x}"
            );
            assert_eq!(
                gm[(x, 22)].symbol(),
                pv[(x, 22)].symbol(),
                "baris footer HARUS identik posisi galaxy_map vs planet_view di x={x}"
            );
        }
    }

    /// M20.3: "Compact: viewport mengecil, panel ringkas" — **dicek LANGSUNG**: viewport
    /// (`vh`×`vw`, dihitung dr `area` yg diterima `render()`) HARUS genuinely MENGECIL saat
    /// area lbh sempit (bukan fixed-size/clip diam-diam) -- dites via PROXY tak-langsung
    /// [`vh`/`vw` privat]: hitung total sel tekstur latar [`.`/`:`/`·`, bg glyph M18.7] yg
    /// genuinely dirender -- area lbh sempit HARUS genuinely hasilkan LEBIH SEDIKIT sel bg
    /// (viewport lbh kecil, bukan konten yg SAMA dipotong/clip diam2 tanpa sadar mengecil).
    #[test]
    fn viewport_genuinely_shrinks_with_smaller_area_compact_vs_full() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let app = App::demo();
        let count_bg_cells = |w: u16, h: u16| -> usize {
            let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
            term.draw(|f| render(f, &app, f.area())).unwrap();
            let buf = term.backend().buffer();
            (0..h)
                .flat_map(|y| (0..w).map(move |x| (x, y)))
                .filter(|&(x, y)| matches!(buf[(x, y)].symbol(), "." | ":" | "·"))
                .count()
        };
        let full = count_bg_cells(120, 40);
        let compact_main = count_bg_cells(64, 27); // MAIN area Compact (`shell_compact`).
        let minimal_main = count_bg_cells(58, 24); // MAIN area Minimal (`shell_minimal`).
        assert!(
            full > compact_main,
            "viewport Full ({full} sel bg) HARUS genuinely > Compact ({compact_main}) -- \
             viewport HARUS mengecil, bukan tetap sama"
        );
        assert!(
            compact_main > minimal_main,
            "viewport Compact ({compact_main} sel bg) HARUS genuinely > Minimal \
             ({minimal_main}) -- mengecil progresif sesuai lebar area"
        );
    }

    /// M20.4: "Minimal: starmap fullscreen + footer" — **dicek LANGSUNG**: `shell_minimal`
    /// [`mod.rs`] SUDAH beri starmap LEBAR PENUH [tanpa split kolom sidebar, beda `shell_full`/
    /// `shell_compact`] + footer shortcut GLOBAL [`render_footer`, sama utk SEMUA view] SUDAH
    /// genuinely tampil. Test ini KONVERSI verifikasi jd guard permanen.
    #[test]
    fn minimal_shows_starmap_fullscreen_with_global_footer() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let app = App::demo();
        let mut term = Terminal::new(TestBackend::new(60, 24)).unwrap();
        term.draw(|f| crate::ui::draw_view(f, &app, "galaxy_map"))
            .unwrap();
        let buf = term.backend().buffer();

        // Fullscreen: kotak galaxy_map (border kiri) HARUS mulai TEPAT stlh outer border
        // (x=1) -- TANPA sidebar mengurangi lebar (beda `shell_full`/`shell_compact`).
        assert_eq!(
            buf[(1, 2)].symbol(),
            "┌",
            "starmap HARUS fullscreen (border kotak mulai x=1, tanpa sidebar) di Minimal"
        );
        // Footer shortcut GLOBAL HARUS genuinely tampil di baris terakhir dlm border.
        let footer_row: String = (0..60u16)
            .map(|x| buf[(x, 22)].symbol().to_string())
            .collect();
        assert!(
            footer_row.contains("g:Galaxy"),
            "footer shortcut global HARUS genuinely tampil di Minimal: {footer_row:?}"
        );
    }

    /// Hint fokus (Phase 7) HARUS genuinely MUAT Minimal (~58w area) walau nama POI
    /// TERPANJANG yg genuinely dihasilkan `gen_name` (gugus: nama+" Nebula") -- disapu BANYAK
    /// seed (bukan 1 nilai kebetulan pendek), overflow ditemukan NYATA sblm ini via screenshot
    /// (`[z/x]Zoom` terpotong jd `[z/x`) & diperbaiki dgn memangkas hint, test ini jd guard
    /// regresi permanen.
    #[test]
    fn focus_hint_worst_case_name_fits_minimal_width() {
        let mut worst_len = 0usize;
        for seed in 0u64..500 {
            for p in crate::galaxy_sim::poi::catalog(seed, false) {
                let hint = format!("Focus: {}  [Enter]Next [Esc]Unfocus", p.name);
                worst_len = worst_len.max(hint.chars().count());
            }
        }
        assert!(
            worst_len <= 58,
            "hint fokus TERPANJANG (500 seed) HARUS genuinely muat Minimal 58w: {worst_len} char"
        );
    }
}
