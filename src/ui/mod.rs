//! UI ratatui: `draw_view` dispatch per view, panel, theme, layout.
//!
//! `06-ui.md`. UI hanya **membaca** snapshot `App`/`GameState`, tak memutasi.
//! `buffer_to_text` dipakai snapshot headless (`agent/test/README.md`).
#![allow(dead_code)]

pub mod galaxy_anim;
pub mod galaxy_pixel;
pub mod galaxy_sim_view;
pub mod layout;
pub mod panels;
pub mod particles;
pub mod portrait;
pub mod raster;
pub mod sprite;
pub mod theme;

use crate::app::App;
use crate::game::defs::ResourceTier;
use crate::game::research;
use crate::game::state::{Biome, MerchantOffer, ShipStatus};
use crate::ui::layout::{LayoutMode, layout_mode};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

/// Ubah `Buffer` → `Vec<String>` (satu baris per string, warna dibuang, trailing space dipangkas).
pub fn buffer_to_text(buf: &Buffer) -> Vec<String> {
    let area = buf.area();
    (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| buf[(x, y)].symbol().chars().next().unwrap_or(' '))
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

/// Nama view valid untuk `draw_view` (dipakai `bin/screenshot.rs` utk validasi arg + pesan error).
/// **Jaga tetap sinkron** dengan cabang `view_main` di bawah bila menambah/menghapus view.
pub const KNOWN_VIEWS: &[&str] = &[
    "main_menu",
    "planet_view",
    "research",
    "galaxy_map",
    "warp",
    "settings",
    "help",
    "merchant",
    "splash",
    "title",
];

/// Renderer panel utama per view (mengisi area detail/main).
type MainFn = fn(&mut Frame, &App, Rect);

/// Petakan nama view → (renderer panel utama, tampilkan strip RESOURCES di atas?).
fn view_main(view: &str) -> Option<(MainFn, bool)> {
    match view {
        "main_menu" => Some((panels::main_view::render, true)),
        "planet_view" => Some((panels::planet::render, false)),
        "research" => Some((panels::research::render, false)),
        "galaxy_map" => Some((panels::galaxy_map::render, false)),
        "warp" => Some((panels::warp::render, false)),
        "settings" => Some((panels::flow::settings::render, false)),
        "help" => Some((panels::flow::help::render, false)),
        "merchant" => Some((panels::story::merchant::render, false)),
        _ => None,
    }
}

/// Render satu view ke frame, responsif per breakpoint (`06` §Responsive).
/// `view`: main_menu | planet_view | galaxy_map | research | warp | settings.
pub fn draw_view(f: &mut Frame, app: &App, view: &str) {
    // Splash/Title (M2): chrome-less full-bleed, di luar shell Full/Compact/Minimal biasa --
    // dicek SEBELUM `view_main` krn keduanya sengaja tak py entri di sana (bukan gap, lihat
    // `every_known_view_resolves_to_a_real_renderer_not_placeholder`'s cek berbasis-render).
    if view == "splash" {
        return panels::flow::splash::render(f, app);
    }
    if view == "title" {
        panels::flow::title::render(f, app);
        if app.new_game_confirm {
            render_new_game_confirm_modal(f);
        }
        return;
    }
    let Some((main, show_res)) = view_main(view) else {
        return draw_placeholder(f, view);
    };
    let area = f.area();
    match layout_mode(area.width, area.height) {
        LayoutMode::Full => shell_full(f, app, main, show_res),
        LayoutMode::Compact => shell_compact(f, app, main, show_res),
        LayoutMode::Minimal => shell_minimal(f, app, view, main),
    }
}

/// Nama planet aktif (unlocked pertama di galaxy aktif) — sama logika `panels::planet::
/// active_planet`, diduplikasi krn cuma butuh `&str` di sini (hindari coupling modul demi 1 baris).
fn active_planet_name(app: &App) -> Option<&str> {
    app.state
        .galaxies
        .iter()
        .find(|g| g.id == app.state.active_galaxy)
        .and_then(|g| g.planets.iter().find(|p| p.unlocked))
        .map(|p| p.name.as_str())
}

/// Detik → `HH:MM:SS` (jam tick game, M12.1 — `1 tick = 1 detik`, `balance::TICK_DURATION_SECS`).
/// `pub(crate)` sejak M13.1 — dipakai jg oleh `panels::resources` (ETA kolom tabel).
pub(crate) fn fmt_hms(total_secs: u64) -> String {
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

fn ship_status_text(app: &App) -> String {
    match app.state.ship.status {
        ShipStatus::Idle => "Ship: Idle".to_string(),
        ShipStatus::Traveling {
            total_secs,
            elapsed_secs,
            ..
        } => {
            let left = (total_secs - elapsed_secs).max(0.0);
            format!("Ship: Traveling ({}s left)", format_num(left))
        }
    }
}

fn research_status_text(app: &App) -> String {
    match app.state.research.active.as_ref() {
        Some(active) => match research::tech_def(&app.content, &active.tech_id) {
            Some(t) => {
                let left = (t.time_secs - active.elapsed_secs).max(0.0);
                format!("Research: {} ({}s left)", t.id, format_num(left))
            }
            None => format!("Research: {}", active.tech_id),
        },
        None => "Research: Idle".to_string(),
    }
}

/// 1 tawaran merchant → teks ringkas "Nama xJumlah harga" (M13.5). Semua data REAL dari
/// `MerchantOffer` (`game/state.rs`) — bukan harga fiktif. `pub(crate)` M1 fix: dipakai jg
/// `panels::story::merchant`'s view detail (bukan cuma footer ringkas) — satu sumber format.
pub(crate) fn offer_text(app: &App, offer: &MerchantOffer) -> String {
    match offer {
        MerchantOffer::SellResource {
            resource,
            amount,
            gain_credits,
        } => {
            let name = &app.content.resources.get(resource.0).name;
            format!(
                "{name} x{} +{}cr",
                compact_num(*amount),
                compact_num(*gain_credits)
            )
        }
        MerchantOffer::BuyResource {
            resource,
            amount,
            cost_cores,
        } => {
            let name = &app.content.resources.get(resource.0).name;
            format!("{name} x{} {cost_cores}wc", compact_num(*amount))
        }
        MerchantOffer::Blueprint {
            recipe, cost_cores, ..
        } => {
            let rec_id = &app.content.recipes.get(recipe.0).id;
            format!("{rec_id} BP {cost_cores}wc")
        }
    }
}

/// Maks tawaran ditampilkan di footer 1-baris sblm dipotong "+N lagi" — cegah bug kelas-sama
/// M13.1/M13.3 (baris/teks kepanjangan diam² lampaui area).
const MAX_MARKET_OFFERS: usize = 2;

/// Footer MARKET (M12.5/M13.5, ref `riftborne/42UN3F.png` §MARKET). Riftborne pakai ticker
/// harga exchange global (V/Au/D/As supply-demand) — game ini TAK py mekanik exchange global,
/// cuma kunjungan `Merchant` episodik nyata (`state.merchant`: `active`/`stock`/`expires_at_
/// tick`/`restock_at_tick`, `game/events.rs`). M13.5: saat aktif, tampilkan KOMODITAS+HARGA
/// ringkas dari `stock` NYATA (maks `MAX_MARKET_OFFERS`, sisanya "+N lagi") — bukan reka-reka
/// exchange-rate baru, murni format ulang data `MerchantOffer` yg sudah ada.
fn market_footer_text(app: &App) -> String {
    let m = &app.state.merchant;
    if m.active {
        let left = m.expires_at_tick.saturating_sub(app.state.tick);
        let shown: Vec<String> = m
            .stock
            .iter()
            .take(MAX_MARKET_OFFERS)
            .map(|o| offer_text(app, o))
            .collect();
        let mut offers = shown.join(" | ");
        if m.stock.len() > MAX_MARKET_OFFERS {
            offers.push_str(&format!(" +{} lagi", m.stock.len() - MAX_MARKET_OFFERS));
        }
        format!("MARKET: {offers} — tutup dlm {}", fmt_hms(left))
    } else {
        let left = m.restock_at_tick.saturating_sub(app.state.tick);
        format!("MARKET: tutup — kunjungan berikut dlm {}", fmt_hms(left))
    }
}

fn render_market_footer(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    f.render_widget(
        Paragraph::new(Line::styled(
            market_footer_text(app),
            Style::default().fg(th.dim),
        )),
        area,
    );
}

/// Top status bar (M12.1, ref `riftborne/42UN3F.png`): lokasi+waktu baris 1, status
/// ship/research (padanan "Build/Training" Riftborne — mekanik nyata game ini, bukan reka-reka
/// konsep baru) baris 2.
fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let location = active_planet_name(app).unwrap_or("(no planet)");
    let clock = fmt_hms(app.state.tick);
    let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(area);

    f.render_widget(
        Paragraph::new(Line::styled(location, Style::default().fg(th.focus).bold())),
        rows[0],
    );
    f.render_widget(
        Paragraph::new(Line::styled(
            format!("T+{clock}"),
            Style::default().fg(th.dim),
        ))
        .alignment(Alignment::Right),
        rows[0],
    );

    let (status, status_style) = match feedback_text(app) {
        Some((msg, ok)) => (msg, feedback_style(&th, ok)),
        None => match tutorial_hint_text(app) {
            Some(hint) => (hint, tutorial_hint_style(&th)),
            None => (
                format!(
                    "{}  |  {}",
                    ship_status_text(app),
                    research_status_text(app)
                ),
                Style::default().fg(th.basic),
            ),
        },
    };
    f.render_widget(Paragraph::new(Line::styled(status, status_style)), rows[1]);
}

/// M1#6: aksi terbaru (`App::feedback`) — dulu SEMUA `let _ = ...` (sukses/gagal SAMA-SAMA tak
/// py sinyal player-facing). Timpa baris status ship/research SAAT ADA (bukan tambah baris
/// baru -- nol perubahan layout/geometri, jd nol risiko overflow breakpoint manapun); waktu-
/// based fade-out (biar otomatis balik ke status normal) itu scope M4, bukan di sini.
fn feedback_text(app: &App) -> Option<(String, bool)> {
    app.last_action.as_ref().map(|(ok, msg)| (msg.clone(), *ok))
}

fn feedback_style(th: &theme::Theme, ok: bool) -> Style {
    Style::default()
        .fg(if ok { th.good } else { th.alert })
        .bold()
}

/// M3: hint onboarding aktif (`state.tutorial_step`), kecuali sedang di-dismiss player (`T`,
/// lihat `app.rs`'s `tutorial_hint_dismissed_for`). Feedback (M1#6) diprioritaskan di atas hint
/// bila dua-duanya ada (sama slot teks, feedback lebih urgent/transien) -- lihat pemanggil.
fn tutorial_hint_text(app: &App) -> Option<String> {
    let step = app.state.tutorial_step?;
    if app.tutorial_hint_dismissed_for == Some(step) {
        return None;
    }
    Some(format!(
        "TUTORIAL: {} [T sembunyikan]",
        crate::game::tutorial::step_hint(step)
    ))
}

fn tutorial_hint_style(th: &theme::Theme) -> Style {
    Style::default().fg(th.focus)
}

/// Versi 1-baris `render_status_bar` (compact ≥70w — tak cukup ruang utk 2 baris penuh).
fn render_status_bar_compact(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let (text, style) = match feedback_text(app) {
        Some((msg, ok)) => (msg, feedback_style(&th, ok)),
        None => match tutorial_hint_text(app) {
            Some(hint) => (hint, tutorial_hint_style(&th)),
            None => {
                let location = active_planet_name(app).unwrap_or("(no planet)");
                let clock = fmt_hms(app.state.tick);
                (
                    format!("{location}  |  T+{clock}  |  {}", ship_status_text(app)),
                    Style::default().fg(th.focus),
                )
            }
        },
    };
    f.render_widget(Paragraph::new(Line::styled(text, style)), area);
}

/// M4: border disorot `th.focus` selagi `view_transition_frames > 0` (disulut `event_loop` saat
/// `app.view` berpindah) — lightweight transition feel tanpa animasi kompleks/timer real-time;
/// `demo()`/snapshot/screenshot-bin selalu `0` (tak pernah lewat `event_loop`), jd nol resiko
/// regresi golden snapshot (dites `outer_block_border_highlights_during_view_transition`).
fn outer_block(app: &App) -> Block<'static> {
    let th = theme::theme(app.state.settings.theme);
    let border_color = if app.view_transition_frames > 0 {
        th.focus
    } else {
        th.header
    };
    Block::bordered()
        .title(Span::styled(
            " STELLAR IDLE ",
            Style::default().fg(th.header).bold(),
        ))
        .border_style(Style::default().fg(border_color))
}

/// Full (≥100×30): 3 kolom PERSISTEN Riftborne — kiri RESOURCES+Building/Training/Assets
/// (M12.2), tengah MAIN (M12.3), **kanan OPTIONS (M12.4, ~18 lebar)** = menu+shortcuts
/// dipindah dari tengah ke kanan (padanan numbered-menu OPTIONS Riftborne — sama-sama navigasi
/// view/page, cuma beda gaya render). `_show_res` tak dipakai lagi — left column SELALU tampil.
fn shell_full(f: &mut Frame, app: &App, main: MainFn, _show_res: bool) {
    let th = theme::theme(app.state.settings.theme);
    let outer = outer_block(app);
    let inner = outer.inner(f.area());
    f.render_widget(outer, f.area());

    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner);
    render_status_bar(f, app, rows[0]);
    let body = rows[1];
    render_market_footer(f, app, rows[2]);

    let cols = Layout::horizontal([
        Constraint::Length(22),
        Constraint::Min(0),
        Constraint::Length(18),
    ])
    .split(body);
    render_left_column(f, app, cols[0]);
    main(f, app, cols[1]);
    // M4: burst partikel (build/upgrade/riset selesai/trade/warp jump — `App::celebrate`)
    // dulu cuma kelihatan di main_menu (`particles.render` sblm ini HANYA dipanggil
    // `main_view.rs`) -- di-overlay di sini jg spy burst di planet_view/research/warp/merchant
    // genuinely TAMPIL, bukan cuma state partikel berubah tanpa efek visual. Aman: `render`
    // no-op total saat `particles` kosong (dicek `is_empty()`), nol resiko dgn snapshot lama.
    app.particles.render(f, cols[1], &th);
    // M12.8: MENU/SHORTCUTS top-align + Min(0) GAP kosong (bukan Min(0) di MENU sendiri) —
    // ref `riftborne/42UN3F.png` §OPTIONS: konten numpuk di atas, celah kosong TANPA border
    // di bawahnya (bukan kotak MENU direntang penuh sampai nempel SHORTCUTS, yg keliatan
    // "berongga"/tak proporsional). M13.4: OPTIONS jadi berkelompok (14 baris isi: header+3
    // grup label+10 item) — `Length(13)`→`Length(16)` (14 isi+2 border).
    // M1 fix: `SHORTCUTS` `Length(7)` HANYA muat 5 baris interior (border 2) — cukup utk
    // KEYS lama (5 entri+judul=6 baris, "?:Help" SUDAH terpotong diam2 SEBELUM perubahan
    // ini, bug nyata bukan diasumsikan). KEYS SKRNG 6 entri+judul=7 baris → `Length(9)`.
    let side = Layout::vertical([
        Constraint::Length(16),
        Constraint::Min(0),
        Constraint::Length(9),
    ])
    .split(cols[2]);
    panels::menu::render(f, app, side[0]);
    panels::shortcuts::render(f, app, side[2]);
}

/// Kolom kiri Riftborne (M12.2, ~22 lebar): RESOURCES (panel nyata, M4) + Building/Training
/// (M13.2, panel nyata `panels::building`) + Assets (M13.3, panel nyata).
/// M13.10: RESOURCES skrng `Min(0)` (PROPORSIONAL ke tinggi SESUNGGUHNYA) — REVISI dari
/// `Length(22)` hardcode (M13.1): 22 cuma pas persis di target 120×40 (body~35, 22+3+4+6=35),
/// tp di BOUNDARY Full-mode minimum (100×30, body~25) overflow NYATA (RESOURCES+Assets kepotong
/// diam², ditemukan M13.3/dikonfirmasi ulang M13.10). Fix: Building(3)/Training(4)/Assets(6)
/// TETAP fixed (kecil, minimal, selalu muat), RESOURCES `Min(0)` serap SISA — di 120×40 tetap
/// dpt 22 (backward-compatible persis), di 100×30 dpt ~12 (`panels::resources::render` skrng
/// hitung max resource yg MUAT dari tinggi NYATA yg diterima, potong sisanya "+N lain" —
/// proporsional, bukan hardcode).
fn render_left_column(f: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(3),
        Constraint::Length(4),
        Constraint::Length(6),
    ])
    .split(area);
    panels::resources::render(f, app, rows[0]);
    let building_inner = titled_box(f, app, rows[1], "Building");
    panels::building::render_building(f, app, building_inner);
    let training_inner = titled_box(f, app, rows[2], "Training");
    panels::building::render_training(f, app, training_inner);
    let assets_inner = titled_box(f, app, rows[3], "Assets");
    panels::building::render_assets(f, app, assets_inner);
}

/// Gambar border+title bertema, kembalikan area dalam utk panel nyata (M13.2/M13.3 — semua
/// slot kolom kiri skrng py konten nyata, tak ada lagi placeholder statis polos).
fn titled_box(f: &mut Frame, app: &App, area: Rect, title: &str) -> Rect {
    let th = theme::theme(app.state.settings.theme);
    let block = Block::bordered()
        .title(Span::styled(title, Style::default().fg(th.header)))
        .border_style(Style::default().fg(th.dim));
    let inner = block.inner(area);
    f.render_widget(block, area);
    inner
}

/// Compact (≥70w): RESOURCES satu baris + sidebar tipis (menu saja) + panel detail. M13.8:
/// "kolom kanan ringkas/sembunyi" — DIVERIFIKASI sudah terpenuhi struktural (bukan diklaim
/// tanpa cek): OPTIONS (`panels::menu`, sama konten M13.4/M13.6, grouped+highlight) jadi
/// **ringkas** krn merangkap posisi kiri (`cols[0]`, `Length(14)`) tanpa kehilangan info vs
/// full — dites 70×24 (batas sempit Compact) s/d 99×29 (batas lebar), tak overflow. SHORTCUTS
/// box (terpisah di full-mode) **sembunyi total** di compact (sengaja, bukan lupa) — ruang
/// `Length(14)` tak cukup utknya + OPTIONS sekaligus, dan shortcut key tetap berfungsi via
/// keybinding tanpa perlu ditampilkan permanen.
fn shell_compact(f: &mut Frame, app: &App, main: MainFn, show_res: bool) {
    let outer = outer_block(app);
    let inner = outer.inner(f.area());
    f.render_widget(outer, f.area());

    let rows = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(inner);
    render_status_bar_compact(f, app, rows[0]);
    let inner = rows[1];

    let body = if show_res {
        let rows = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(inner);
        panels::resources::render_compact(f, app, rows[0]);
        rows[1]
    } else {
        inner
    };
    let cols = Layout::horizontal([Constraint::Length(14), Constraint::Min(0)]).split(body);
    panels::menu::render(f, app, cols[0]);
    main(f, app, cols[1]);
    // M4: SAMA overlay burst partikel spt `shell_full` (lihat komentarnya) — konsisten di
    // breakpoint Compact jg.
    let th = theme::theme(app.state.settings.theme);
    app.particles.render(f, cols[1], &th);
}

/// Minimal (<70w): tab bar atas + panel detail lebar penuh + footer shortcut. M13.9:
/// DIVERIFIKASI tab bar SUDAH sinkron dinamis (`render_tab_bar` bandingkan param `view` — beda
/// dari bug M13.6 di `panels::menu` yg baca `app.view` STATE: di sini `view` DAN `app.view`
/// SELALU sama nilainya krn `app.rs` event loop panggil `draw_view(f, app, &app.view.clone())`
/// — jadi highlight tab genuinely bisa di-screenshot-test langsung, dikonfirmasi 3 view beda).
fn shell_minimal(f: &mut Frame, app: &App, view: &str, main: MainFn) {
    let outer = outer_block(app);
    let inner = outer.inner(f.area());
    f.render_widget(outer, f.area());

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner);
    render_tab_bar(f, app, rows[0], view);
    main(f, app, rows[1]);
    // M4: SAMA overlay burst partikel spt `shell_full`.
    let th = theme::theme(app.state.settings.theme);
    app.particles.render(f, rows[1], &th);
    render_footer(f, app, rows[2]);
}

const TABS: [(&str, &str); 5] = [
    ("main_menu", "Menu"),
    ("planet_view", "Planet"),
    ("research", "Research"),
    ("galaxy_map", "Galaxy"),
    ("warp", "Warp"),
];

fn render_tab_bar(f: &mut Frame, app: &App, area: Rect, view: &str) {
    let th = theme::theme(app.state.settings.theme);
    let mut spans = Vec::new();
    for (i, (id, label)) in TABS.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" | ", Style::default().fg(th.dim)));
        }
        let st = if *id == view {
            Style::default().fg(th.header).bold()
        } else {
            Style::default().fg(th.sidebar)
        };
        spans.push(Span::styled(*label, st));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    // M1#6: breakpoint Minimal tak py baris status terpisah spt Full/Compact -- footer
    // keybind-hint dipakai ulang utk feedback SAAT ADA (timpa, bukan tambah baris).
    if let Some((msg, ok)) = feedback_text(app) {
        f.render_widget(
            Paragraph::new(Line::styled(msg, feedback_style(&th, ok))),
            area,
        );
        return;
    }
    if let Some(hint) = tutorial_hint_text(app) {
        f.render_widget(
            Paragraph::new(Line::styled(hint, tutorial_hint_style(&th))),
            area,
        );
        return;
    }
    let txt = "g:Galaxy p:Planet r:Research m:Merchant S:Settings ?:Help q:Quit";
    f.render_widget(
        Paragraph::new(Line::styled(txt, Style::default().fg(th.dim))),
        area,
    );
}

pub(crate) fn format_num(v: f64) -> String {
    let n = v.round() as i64;
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

/// Format ringkas k/M (mis. "1.2k", "3.4M") utk KUANTITAS besar (resource/kredit/data) — M13.1
/// (asal, scoped tabel RESOURCES) → M13.7 (dipindah kesini, `pub(crate)`, dipakai lintas panel
/// utk KONSISTENSI: kuantitas besar seragam k/M, BUKAN durasi/detik yg tetap `format_num` —
/// "48s left" vs "4.0k s left" utk durasi lama justru kurang terbaca, jadi durasi SENGAJA tak
/// disentuh (di luar cakupan "konsisten" yg dimaksud: kuantitas sejenis harus seragam, bukan
/// SEMUA angka apa pun jenisnya dipaksa 1 format).
pub(crate) fn compact_num(v: f64) -> String {
    let sign = if v < 0.0 { "-" } else { "" };
    let v = v.abs();
    if v >= 1_000_000.0 {
        format!("{sign}{:.1}M", v / 1_000_000.0)
    } else if v >= 1_000.0 {
        format!("{sign}{:.1}k", v / 1_000.0)
    } else {
        format!("{sign}{}", v.round() as i64)
    }
}

/// Label ringkas biome (`galaxy_map.rs`'s label internal M18, dipindah kesini M14.2 — dipakai
/// jg `planet.rs`'s header, konsisten sama nama drpd duplikasi match terpisah).
pub(crate) fn biome_label(b: Biome) -> &'static str {
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

/// Bar `[####......]` lebar 10 dari fraksi 0..1. `06` §Progress Bars. Dipindah dari
/// `research.rs` (M14.11, pola sama `compact_num`/`biome_label`) — dipakai jg `planet.rs`'s
/// bar kapasitas stockpile, satu implementasi bar bukan duplikasi per panel.
pub(crate) fn progress_bar(frac: f64) -> String {
    let w = 10usize;
    let filled = (frac.clamp(0.0, 1.0) * w as f64).round() as usize;
    let mut s = String::with_capacity(w + 2);
    s.push('[');
    for i in 0..w {
        s.push(if i < filled { '#' } else { '.' });
    }
    s.push(']');
    s
}

/// Warna per `ResourceTier`, dipindah dari `resources.rs` (M14.17, pola sama `compact_num`/
/// `biome_label`/`progress_bar`) — dipakai jg `planet.rs`'s NODES (M14.17: dulu hardcode
/// `th.basic` utk SEMUA node terlepas tier asli, kebetulan tak ketauan krn 3 node demo Earth
/// semua Basic — satu implementasi tier→warna, bukan duplikasi/divergen per panel).
pub(crate) fn tier_color(th: &theme::Theme, tier: ResourceTier) -> Color {
    match tier {
        ResourceTier::Basic => th.basic,
        ResourceTier::Advanced => th.advanced,
        ResourceTier::Rare | ResourceTier::Special => th.rare,
    }
}

/// M2: modal konfirmasi overwrite (`app.new_game_confirm`) — cegah New Game diam2 timpa save
/// lama tanpa peringatan (save sudah ada = pemain py progress nyata). Pola exclusive-input
/// SAMA `BuildPicker` (semua key dialihkan sampai modal ditutup, lihat `handle_key`).
fn render_new_game_confirm_modal(f: &mut Frame) {
    let area = f.area();
    let w = area.width.min(50);
    let h = 5.min(area.height);
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let modal = Rect::new(area.x + x, area.y + y, w, h);
    let block = Block::bordered().title(" New Game ").bold();
    let text = Paragraph::new(vec![
        Line::from("Save lama akan DITIMPA. Lanjut?"),
        Line::from("[Enter] Ya, timpa   [Esc] Batal"),
    ])
    .block(block)
    .alignment(Alignment::Center);
    f.render_widget(ratatui::widgets::Clear, modal);
    f.render_widget(text, modal);
}

fn draw_placeholder(f: &mut Frame, view: &str) {
    let area = f.area();
    let b = Block::bordered().title(Span::styled(
        format!(" STELLAR IDLE — {view} (TODO) "),
        Style::default().bold(),
    ));
    f.render_widget(b, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::events::restock_merchant;

    /// M13.5: `App::demo()` tak pernah aktifkan merchant (dicatat sbg batas M12.5) — tes ini
    /// isi kekosongan itu via `restock_merchant` NYATA (bukan `MerchantOffer` fiktif tangan),
    /// pastikan cabang aktif `market_footer_text` genuinely render komoditas+harga, bukan cuma
    /// diasumsikan benar dari baca kode.
    #[test]
    fn market_footer_shows_real_offers_when_active() {
        let mut app = App::demo();
        restock_merchant(&mut app.state.merchant, &app.content, app.state.tick);
        assert!(app.state.merchant.active, "restock harus aktifkan merchant");
        assert!(
            !app.state.merchant.stock.is_empty(),
            "restock harus isi stock"
        );
        let text = market_footer_text(&app);
        assert!(text.starts_with("MARKET: "));
        assert!(text.contains("tutup dlm"));
        // Minimal 1 tawaran nyata (Iron jual, dari `restock_merchant`) muncul di teks.
        assert!(
            text.contains("Iron"),
            "tawaran Iron seharusnya tampil: {text}"
        );
    }

    #[test]
    fn market_footer_shows_closed_status_when_inactive() {
        let app = App::demo();
        assert!(!app.state.merchant.active);
        let text = market_footer_text(&app);
        assert!(text.contains("tutup"));
        assert!(text.contains("kunjungan berikut"));
    }

    /// M13.7: `compact_num` skrng dipakai lintas panel (resources/warp/research/planet/market)
    /// — dites langsung di titik batas (999/1000/999_999/1_000_000) + negatif, bukan cuma
    /// diuji tak langsung lewat 1 panel spt M13.1. **Quirk jujur ditemukan+didokumentasikan**
    /// (bukan disembunyikan): `999_999.0` → `"1000.0k"` bukan `"1.0M"` — cek ambang `>=1_000_000`
    /// terjadi SEBELUM pembulatan 1-desimal, jadi nilai persis di bawah 1jt yg PEMBULATANNYA
    /// jadi 1000.0 tetap masuk cabang k. Pita sempit (~999_950-999_999), TAK terjadi di data
    /// game nyata (cap demo 100_000, jauh di bawah); tak diperbaiki (di luar scope "konsisten
    /// LINTAS panel" M13.7 — bkn soal presisi ambang k/M itu sendiri).
    /// M1#6: feedback msg (`app.last_action`) harus GANTIKAN teks status normal, bukan cuma
    /// disimpan tp tak pernah tampil. Cek 3 breakpoint (Full/Compact/Minimal) krn tiap mode py
    /// slot teks beda (`render_status_bar`/`render_status_bar_compact`/`render_footer`).
    fn render_planet_view(app: &App, w: u16, h: u16) -> Vec<String> {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        term.draw(|f| draw_view(f, app, "planet_view")).unwrap();
        buffer_to_text(term.backend().buffer())
    }

    #[test]
    fn feedback_overrides_status_bar_full() {
        let mut app = App::demo();
        app.last_action = Some((true, "Dibangun: Extractor".into()));
        let text = render_planet_view(&app, 100, 30).join("\n");
        assert!(
            text.contains("Dibangun: Extractor"),
            "feedback harus tampil di status bar Full: {text:?}"
        );
    }

    #[test]
    fn feedback_overrides_status_bar_compact() {
        let mut app = App::demo();
        app.last_action = Some((false, "Upgrade gagal: Insufficient".into()));
        let text = render_planet_view(&app, 60, 20).join("\n");
        assert!(
            text.contains("Upgrade gagal: Insufficient"),
            "feedback harus tampil di status bar Compact: {text:?}"
        );
    }

    #[test]
    fn feedback_overrides_footer_minimal() {
        let mut app = App::demo();
        app.last_action = Some((true, "Riset dimulai: x".into()));
        let text = render_planet_view(&app, 40, 15).join("\n");
        assert!(
            text.contains("Riset dimulai: x"),
            "feedback harus tampil di footer Minimal: {text:?}"
        );
    }

    #[test]
    fn no_feedback_shows_normal_status_text() {
        let app = App::demo();
        assert!(app.last_action.is_none());
        let text = render_planet_view(&app, 100, 30).join("\n");
        assert!(
            !text.contains("Dibangun") && !text.contains("gagal"),
            "tanpa feedback, teks status normal (bukan pesan aksi): {text:?}"
        );
    }

    /// M1#7: cegah regresi kelas "advertised di `KNOWN_VIEWS`/`shortcuts.rs` tp jatuh ke
    /// `draw_placeholder`" (persis bug Merchant/Help sblm M1). Setiap view NYATA yg terjangkau
    /// (dilist `KNOWN_VIEWS`, dipakai `bin/screenshot.rs` + validasi arg) HARUS py entri
    /// `view_main` — kalau nanti ada penambahan view di `KNOWN_VIEWS` tanpa `view_main` diisi,
    /// tes ini gagal SEGERA (bukan ketemu manual pas main).
    #[test]
    fn every_known_view_resolves_to_a_real_renderer_not_placeholder() {
        // Berbasis-RENDER (bukan cuma `view_main(v).is_some()`) krn Splash/Title (M2) chrome-
        // less, sengaja di luar `view_main`'s shell-dispatch tp TETAP renderer nyata -- cek
        // literal marker placeholder (`" (TODO) "`, lihat `draw_placeholder`) tak muncul.
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;
        let app = App::demo();
        for view in KNOWN_VIEWS {
            let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
            term.draw(|f| draw_view(f, &app, view)).unwrap();
            let text = buffer_to_text(term.backend().buffer()).join("\n");
            assert!(
                !text.contains("(TODO)"),
                "KNOWN_VIEWS {view:?} jatuh ke draw_placeholder — view mati/belum \
                 diimplementasi tp diiklankan sbg reachable: {text:?}"
            );
        }
    }

    #[test]
    fn unknown_view_name_still_falls_back_to_placeholder_without_panic() {
        // Kontrol negatif: nama sembarang (bukan di KNOWN_VIEWS) MEMANG boleh placeholder —
        // tes di atas cuma menjaga daftar YG DIIKLANKAN, bukan melarang placeholder sama sekali.
        assert!(view_main("some_nonexistent_view_xyz").is_none());
    }

    /// M3: hint tutorial harus benar2 tampil (bukan cuma disimpan di state) dan hilang saat
    /// di-dismiss -- 3 breakpoint, pola SAMA tes feedback (`feedback_overrides_status_bar_*`).
    #[test]
    fn tutorial_hint_shows_at_all_breakpoints_and_hides_when_dismissed() {
        let mut app = App::demo();
        app.state.tutorial_step = Some(0);
        let text = render_planet_view(&app, 100, 30).join("\n");
        assert!(
            text.contains("TUTORIAL:"),
            "hint step 0 harus tampil: {text:?}"
        );

        app.tutorial_hint_dismissed_for = Some(0);
        let text = render_planet_view(&app, 100, 30).join("\n");
        assert!(
            !text.contains("TUTORIAL:"),
            "hint harus hilang setelah dismiss: {text:?}"
        );
    }

    #[test]
    fn feedback_takes_priority_over_tutorial_hint_in_same_slot() {
        let mut app = App::demo();
        app.state.tutorial_step = Some(0);
        app.last_action = Some((true, "Dibangun: Extractor".into()));
        let text = render_planet_view(&app, 100, 30).join("\n");
        assert!(text.contains("Dibangun: Extractor"));
        assert!(!text.contains("TUTORIAL:"));
    }

    /// M4: border outer_block harus GENUINELY berubah warna selama transisi (bukan cuma field
    /// disimpan tanpa efek visual) — bandingkan warna cell border top-left di 0 vs >0 frame.
    #[test]
    fn outer_block_border_highlights_during_view_transition() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;
        let mut app = App::demo();
        app.view_transition_frames = 0;
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term.draw(|f| draw_view(f, &app, "planet_view")).unwrap();
        let normal_fg = term.backend().buffer()[(0, 0)].fg;

        app.view_transition_frames = 4;
        let mut term2 = Terminal::new(TestBackend::new(100, 30)).unwrap();
        term2.draw(|f| draw_view(f, &app, "planet_view")).unwrap();
        let highlighted_fg = term2.backend().buffer()[(0, 0)].fg;

        assert_ne!(
            normal_fg, highlighted_fg,
            "border harus beda warna selagi transisi aktif"
        );
    }

    #[test]
    fn compact_num_formats_boundaries_correctly() {
        assert_eq!(compact_num(0.0), "0");
        assert_eq!(compact_num(850.0), "850");
        assert_eq!(compact_num(999.0), "999");
        assert_eq!(compact_num(1000.0), "1.0k");
        assert_eq!(compact_num(1240.0), "1.2k");
        assert_eq!(compact_num(999_999.0), "1000.0k"); // quirk, lihat doc komentar di atas
        assert_eq!(compact_num(1_000_000.0), "1.0M");
        assert_eq!(compact_num(-1500.0), "-1.5k");
    }
}
