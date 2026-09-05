//! View RESEARCH: terminal riset — Data/sec + total, riset aktif (progress bar), available techs.
//! `06-ui.md` §Research, `03-progression.md` §5.
#![allow(dead_code)]

use crate::app::App;
use crate::game::defs::{TechBranch, TechDef};
use crate::game::research;
use crate::ui::{compact_num, format_num, progress_bar, theme};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Wrap};

/// M15.1: urutan branch TETAP (bukan urutan katalog acak) — "tree tech per cabang" checklist
/// minta tech DIKELOMPOKKAN per branch, bukan daftar flat spt sblmnya (`AVAILABLE TECHS` cuma
/// filter `is_available`, tak pernah kelompokkan/tampilkan tech locked sama sekali).
const BRANCHES: [TechBranch; 4] = [
    TechBranch::Extraction,
    TechBranch::Manufacturing,
    TechBranch::Aerospace,
    TechBranch::AstroCartography,
];

/// M15.4: ambang `area.height` di bawah mana panel DETAIL disembunyikan — Minimal (60×24,
/// area.height=20) kehabisan ruang, action hint terdorong keluar. Compact (80×30, area.height=
/// 27) py cukup ruang. Dikalibrasi via `render_to_text` empiris, pola sama `planet.rs`'s
/// `SHIP_RESEARCH_MIN_HEIGHT`.
const DETAIL_MIN_HEIGHT: u16 = 24;

/// M15.6: ambang `area.width` di bawah mana suffix connector dependency disembunyikan. Dihitung
/// MAIN width nyata tiap breakpoint (subtraksi sidebar shell): Minimal 60×24→58w, Compact
/// 80×30→64w (14w sidebar MENU), Full-MINIMUM 100×30→58w (22+18w sidebar) — SEMUA di bawah 70,
/// baris terpanjang berisi suffix (mis. "aero_warp_mk3 ... └aero_warp_mk2") butuh ~68w, WRAP di
/// ketiganya, tree list SENDIRI kepotong (dikonfirmasi screenshot). Cuma Full LEBAR (120×40,
/// MAIN 78w) genuinely muat. Trade-off SADAR: fitur ini cuma tampil di layar lebar, TETAP
/// tersedia via panel DETAIL (M15.4, "Depends on: X") di breakpoint manapun.
const DEPS_SUFFIX_MIN_WIDTH: u16 = 70;

fn branch_label(b: TechBranch) -> &'static str {
    match b {
        TechBranch::Extraction => "EXTRACTION",
        TechBranch::Manufacturing => "MANUFACTURING",
        TechBranch::Aerospace => "AEROSPACE",
        TechBranch::AstroCartography => "ASTROCARTOGRAPHY",
    }
}

/// M15.2: checklist minta 4 status NODE tree beda warna/ikon — `Done` (`state.research.
/// completed`, ADA di state tp tak pernah dibaca UI sblm ini), `Active` (`state.research.active`,
/// sblmnya cuma dipakai exclude dari daftar branch, statusnya sendiri tak pernah ditandai DI
/// tree), `Available`/`Locked` (`research::is_available`, SAMA formula persis dipakai `try_
/// start_research` — bukan ambang baru).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TechStatus {
    Locked,
    Available,
    Active,
    Done,
}

fn tech_status(app: &App, id: &str) -> TechStatus {
    if app.state.research.completed.contains(id) {
        TechStatus::Done
    } else if app
        .state
        .research
        .active
        .as_ref()
        .is_some_and(|a| a.tech_id == id)
    {
        TechStatus::Active
    } else if research::is_available(&app.state, &app.content, id) {
        TechStatus::Available
    } else {
        TechStatus::Locked
    }
}

/// M15.2: **overflow real ditemukan** — versi awal `[LOCKED]`/`[ACTIVE]`/dst (7-8 char) bikin
/// baris wrap LEBIH BANYAK drpd sblmnya, di 60×24 (Minimal tersempit) action hint SAMPAI HABIS
/// terdorong keluar area (dikonfirmasi `render_to_string` dump: box tutup PERSIS stlh tech
/// terakhir, hint tak kebagian ruang sama sekali). Diperpendek jd 4 char seragam (`[--]`/`[OK]`/
/// `[>>]`/`[DN]`) — pola SAMA `[OK]`/`[--]` afford indicator (M14.9) drpd bikin istilah baru,
/// lebar SAMA semua status (rata kolom, M14.21's pola).
fn status_marker(s: TechStatus) -> &'static str {
    match s {
        TechStatus::Locked => "[--]",
        TechStatus::Available => "[OK]",
        TechStatus::Active => "[>>]",
        TechStatus::Done => "[DN]",
    }
}

fn status_color(th: &theme::Theme, s: TechStatus) -> ratatui::style::Color {
    match s {
        TechStatus::Locked => th.dim,
        TechStatus::Available => th.advanced,
        TechStatus::Active => th.good,
        TechStatus::Done => th.rare,
    }
}

/// M15.4: SATU sumber urutan tech (branch loop render + hitung index terpilih) — kalau
/// urutannya beda antara render dan `app.sel`'s target, selektor `►` bisa nunjuk BEDA tech drpd
/// yg genuinely dites di detail bawah. Urutan SAMA PERSIS `BRANCHES` const dipakai render loop.
/// M15.9: `pub(crate)` (dulu private) — `app.rs`'s `research_keys` (Enter) butuh urutan SAMA
/// PERSIS ini utk nentuin tech id dari `app.sel`, biar Enter genuinely mulai riset tech yg
/// SAMA persis yg ditunjuk selektor `►`/DETAIL (bukan reka urutan terpisah yg bisa menyimpang).
pub(crate) fn all_techs_ordered(content: &crate::game::defs::Content) -> Vec<&TechDef> {
    BRANCHES
        .iter()
        .flat_map(|&b| content.techs.iter().filter(move |t| t.branch == b))
        .collect()
}

/// M15.4: deskripsi manusiawi `TechUnlock` — sblmnya `unlock` (efek NYATA tech, dipakai genuine
/// `research::tick`'s completion handler) tak pernah ditampilkan ke user sama sekali, cuma
/// "cost"+"time" (harga masuk) yg terlihat, bukan HASIL riset itu (checklist eksplisit minta
/// "efek").
fn describe_unlock(u: &crate::game::defs::TechUnlock) -> String {
    use crate::game::defs::TechUnlock;
    match u {
        TechUnlock::Recipe(id) => format!("Buka resep: {id}"),
        TechUnlock::WarpTier(n) => format!("Warp Tier -> {n}"),
        TechUnlock::UnlockBuilding(id) => format!("Buka bangunan: {id}"),
        TechUnlock::TechMult { resources, add } => {
            format!("+{:.0}% laju [{}]", add * 100.0, resources.join(", "))
        }
        TechUnlock::AccessOuterTier(n) => format!("Akses tier luar {n}"),
    }
}

/// Data/sec dari ResearchLab di galaksi aktif.
fn data_rate(app: &App) -> f64 {
    app.state
        .galaxies
        .iter()
        .find(|g| g.id == app.state.active_galaxy)
        .map(research::data_rate)
        .unwrap_or(0.0)
}

/// M15.7: "Warp Tier saat ini + next unlock" — `state.ship.warp_tier` (dipakai NYATA
/// `research::tick`'s completion handler, cek `TechUnlock::WarpTier(n)` cuma terapkan bila
/// `n > warp_tier` genuinely) ADA di state tp tak pernah ditampilkan di research.rs sama sekali
/// (planet_view's SHIP line nunjukkan tier SAAT INI, tp research view — tempat yg genuinely
/// MENAIKKAN tier via riset — tak py info ini sama sekali). Cari tech `WarpTier` BLM completed
/// dgn `n` TERKECIL (bukan cuma tech pertama di katalog — RON's urutan tak dijamin menaik).
fn next_warp_unlock(app: &App) -> Option<(&str, u8)> {
    app.content
        .techs
        .iter()
        .filter(|t| !app.state.research.completed.contains(&t.id))
        .filter_map(|t| match t.unlock {
            crate::game::defs::TechUnlock::WarpTier(n) => Some((t.id.as_str(), n)),
            _ => None,
        })
        .min_by_key(|&(_, n)| n)
}

/// Baris riset aktif (header + progress bar), atau placeholder bila tak ada.
/// M15.3: **gap real ditemukan** — dulu cuma tampil 1 pct BLENDED (`data_frac.min(time_frac)`),
/// angka Data invested/cost MENTAH (yg checklist eksplisit minta: "Data invested + waktu") tak
/// pernah ditampilkan sama sekali — dites `research::tick` (game/research.rs): completion butuh
/// `data_invested >= data_cost` **DAN** `elapsed_secs >= time_secs` (2 syarat independen), pct
/// blended MENYEMBUNYIKAN yg mana yg jd bottleneck (mis. Data SUDAH penuh tp msh nunggu waktu,
/// user tak akan tau dari 1 angka blended). +baris "Data: X/Y" terpisah dari waktu.
fn active_lines(app: &App, th: &theme::Theme) -> Vec<Line<'static>> {
    let Some(active) = app.state.research.active.as_ref() else {
        return vec![Line::styled(
            "ACTIVE: (none) - pilih tech di bawah",
            Style::default().fg(th.dim),
        )];
    };
    let Some(t) = research::tech_def(&app.content, &active.tech_id) else {
        return vec![];
    };
    // Completion butuh Data DAN waktu → progress bar pakai fraksi pembatas (yang terkecil) sbg
    // ringkasan visual cepat, tp angka MENTAH keduanya TETAP ditampilkan terpisah (baris di bawah)
    // biar user tau PERSIS bottleneck-nya yg mana (Data penuh nunggu waktu, atau sebaliknya).
    let data_frac = frac(active.data_invested, t.data_cost);
    let time_frac = frac(active.elapsed_secs, t.time_secs);
    let p = data_frac.min(time_frac);
    let pct = (p.clamp(0.0, 1.0) * 100.0).round() as u32;
    let left = (t.time_secs - active.elapsed_secs).max(0.0);
    vec![
        Line::styled(
            format!("ACTIVE: {} [{:?}]", t.id, t.branch),
            Style::default().fg(th.good).bold(),
        ),
        Line::styled(
            format!("{} {pct}%", progress_bar(p)),
            Style::default().fg(th.good),
        ),
        Line::styled(
            format!(
                "Data: {}/{}  Time: {}/{}s ({}s left)",
                compact_num(active.data_invested),
                compact_num(t.data_cost),
                format_num(active.elapsed_secs),
                format_num(t.time_secs),
                format_num(left)
            ),
            Style::default().fg(th.good),
        ),
    ]
}

fn frac(cur: f64, max: f64) -> f64 {
    if max > 0.0 { cur / max } else { 1.0 }
}

/// M15.2: **overflow real ditemukan** — status marker (M15.2) + tree penuh (M15.1) bikin baris
/// LEBIH BANYAK drpd versi lama; di 60×24 (Minimal tersempit) action hint TERDORONG KELUAR area
/// SAMA SEKALI (dikonfirmasi `render_to_string` dump: box tutup persis stlh tech terakhir, hint
/// tak kebagian ruang). 2 blank-line KOSMETIK dibuang (antara header+ACTIVE, antara tree+hint) —
/// sisa 1 (antara ACTIVE+tree, pemisah paling berguna: detail progress vs daftar padat) — pola
/// sama M14.11/13's trim.
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let content = &app.content;
    let mut lines: Vec<Line> = vec![
        Line::styled("RESEARCH TERMINAL", Style::default().fg(th.focus).bold()),
        Line::styled(
            format!(
                "Data/sec: +{}   Data total: {}",
                compact_num(data_rate(app)),
                compact_num(app.state.research.data)
            ),
            Style::default().fg(th.rare),
        ),
    ];
    // M15.7: "Warp Tier saat ini + next unlock" — `ship.warp_tier` (dipakai NYATA `research::
    // tick`'s completion handler) ADA di state, tapi research view — tempat yg genuinely
    // MENAIKKAN tier via riset — tak py info ini sama sekali sblm ini (cuma planet_view's SHIP
    // line nunjukkan tier SAAT INI, tanpa konteks "abis ini naik ke berapa").
    // **Overflow real ditemukan**: +1 baris ini di Minimal (60×24, area.height=20) dorong action
    // hint keluar (sama kelas M15.2/4's insiden) — digating sama ambang `DETAIL_MIN_HEIGHT`
    // (ringkasan sekunder, pola sama DETAIL panel M15.4).
    if area.height >= DETAIL_MIN_HEIGHT {
        let warp_line = match next_warp_unlock(app) {
            Some((id, n)) => format!(
                "Warp Tier: {} | Next: {id} -> Tier {n}",
                app.state.ship.warp_tier
            ),
            None => format!(
                "Warp Tier: {} | Next: (semua WarpTier selesai)",
                app.state.ship.warp_tier
            ),
        };
        lines.push(Line::styled(warp_line, Style::default().fg(th.advanced)));
    }
    lines.extend(active_lines(app, &th));
    lines.push(Line::raw(""));

    // M15.1: dulu "AVAILABLE TECHS" 1 daftar FLAT (cuma tech `is_available`, tech locked/done tak
    // PERNAH tampil sama sekali — bukan "tree", cuma potongan). Skrng dikelompokkan per `branch`
    // (urutan TETAP `BRANCHES`), tampilkan SEMUA tech tiap branch — biar struktur TREE genuinely
    // terlihat utuh.
    // M15.2: tiap tech skrng py MARKER+WARNA status (`tech_status`) — dulu SEMUA tech (locked
    // ATAU available) tampil SAMA PERSIS `th.advanced` tanpa beda visual sama sekali (locked tak
    // bisa dibedakan dari available cuma dari teks). Tech AKTIF/DONE jg genuinely muncul DI tree
    // (dulu aktif SENGAJA disaring keluar list, DONE tak pernah dicek state `completed` sama
    // sekali) — checklist minta 4 status hidup BERSAMA di tree yg sama, bukan sebagian
    // disembunyikan.
    // M15.4: "detail tech terpilih" butuh KONSEP terpilih ADA dulu — `app.sel` (field SAMA
    // dipakai planet_view/galaxy_map) dipakai reuse, di-klem ke total tech (`all_techs_ordered`).
    // j/k BELUM genuinely gerakkan `sel` utk view research (itu scope M15.8, item TERPISAH) —
    // skrng `sel` nunjuk tech PERTAMA (default 0) dgn marker `►` polos (highlight PENUH/REVERSED
    // msh scope M15.10, sama pola M14.3→M14.8's split bertahap).
    let ordered = all_techs_ordered(content);
    let sel = if ordered.is_empty() {
        0
    } else {
        app.sel.min(ordered.len() - 1)
    };
    // M15.15: "screenshot vs target → layak" — dibandingkan ref Riftborne `_rTKGm.png` (tabel
    // BUILD/UPGRADE: kolom Level/cost/dst SEJAJAR rapi via nama di-pad) — tech id demo beda
    // panjang (`ext_deep_core`=12 vs `manu_steel`=10 dst) bikin "cost:"/"time:" TAK sejajar
    // antar baris, pola gap SAMA persis M14.21's "angka rata-kanan" planet_view. Dihitung
    // GLOBAL (bukan per-branch) — Minimal (M15.13) tampilkan flat list TANPA grouping branch,
    // jadi 1 lebar kolom seragam berlaku utk KEDUA mode tampilan (tree ATAU flat).
    let name_w = ordered.iter().map(|t| t.id.len()).max().unwrap_or(0);
    let mut flat_idx = 0usize;
    // M15.11: "scroll bila tree > area" — indeks baris (di `lines`) tempat tech TERPILIH
    // genuinely dirender, dipakai hitung scroll offset stlh SEMUA lines selesai dibangun (lihat
    // bawah, dekat `Paragraph::new(lines).scroll(...)`).
    let mut sel_line_idx = 0usize;
    // M15.13: "Minimal: list tech (no grafis tree)" — dulu branch header ("EXTRACTION (2)" dst)
    // SELALU tampil di SEMUA breakpoint termasuk Minimal (60×24), padahal checklist minta
    // Minimal genuinely TANPA struktur tree (flat list tech saja, lbh sederhana). Reuse ambang
    // `DETAIL_MIN_HEIGHT` (SAMA PERSIS dipakai gate DETAIL/Warp Tier) — dikonfirmasi height
    // Minimal(20)/Compact(27)/Full-minimum(25) SEMUA beda, threshold ini genuinely pisahkan
    // Minimal dari 2 breakpoint lain (area.width TAK reliable: Minimal@60×24 & Full-minimum@
    // 100×30 KEBETULAN sama-sama 58w, area.height beda 20 vs 25).
    let show_tree_headers = area.height >= DETAIL_MIN_HEIGHT;
    for branch in BRANCHES {
        let techs: Vec<&TechDef> = content
            .techs
            .iter()
            .filter(|t| t.branch == branch)
            .collect();
        if techs.is_empty() {
            continue;
        }
        if show_tree_headers {
            lines.push(Line::styled(
                format!("{} ({})", branch_label(branch), techs.len()),
                Style::default().fg(th.header).bold(),
            ));
        }
        for t in &techs {
            let selected = flat_idx == sel;
            let marker = if selected { "\u{25ba} " } else { "  " };
            let status = tech_status(app, &t.id);
            // M15.10: "highlight node terpilih" — marker `►` polos (M15.4) ditemukan KURANG
            // jelas, dibandingkan `panels::menu`/`planet_view`'s selector (M13.6/M14.8,
            // `Modifier::REVERSED`+bold) yg jauh lbh menonjol — pola SAMA PERSIS diterapkan di
            // sini (baris terpilih disorot PENUH, bukan cuma marker kecil di depan).
            let mut style = Style::default().fg(status_color(&th, status));
            if selected {
                style = style.add_modifier(Modifier::REVERSED).bold();
            }
            // M15.6: "garis dependency antar node (ASCII connector)" — dulu depends_on CUMA
            // kelihatan di panel DETAIL (M15.4, teks "Depends on: X") utk tech TERPILIH SAJA;
            // tech LAIN yg tak terpilih (mayoritas — 5/7 tech demo py depends_on) tak nunjukkan
            // rantai dependency sama sekali sekilas pandang. +suffix connector "\u{2514} X"
            // (ASCII box-drawing "└", sama keluarga karakter border │┌┐└┘ yg SUDAH dipakai UI
            // ini) INLINE di baris yg SAMA (bukan baris baru) — cross-branch dependency (mis.
            // `ext_deep_core` di EXTRACTION butuh `manu_steel` di MANUFACTURING) tak bisa digambar
            // sbg garis vertikal LITERAL nyambung antar seksi tanpa redesign tree total (di luar
            // scope "(S)" item ini) — suffix teks ringkas TETAP kasih tau rantai sekilas pandang.
            // **Overflow real ditemukan**: di Minimal (60×24, MAIN 58w) suffix bikin baris WRAP
            // (baris >58w), 5/7 tech py deps → 5 baris wrap jd 2 baris = tree list ("list tetap")
            // SENDIRI kepotong (`astro_far_warp` hilang total, dikonfirmasi screenshot). Suffix
            // disembunyikan di bawah `DEPS_SUFFIX_MIN_WIDTH` — list INTI tak pernah dikorbankan
            // demi dekorasi connector.
            let deps_suffix = if t.depends_on.is_empty() || area.width < DEPS_SUFFIX_MIN_WIDTH {
                String::new()
            } else {
                format!("  \u{2514}{}", t.depends_on.join(","))
            };
            if selected {
                sel_line_idx = lines.len();
            }
            lines.push(Line::styled(
                format!(
                    "{marker}{} {:<name_w$}  cost: {} Data  time: {}s{deps_suffix}",
                    status_marker(status),
                    t.id,
                    compact_num(t.data_cost),
                    format_num(t.time_secs)
                ),
                style,
            ));
            flat_idx += 1;
        }
    }
    // M15.4: **gap real ditemukan** — dulu tak ada cara sama sekali liat efek/depends_on tech
    // SEBELUM diriset (cuma cost/time "harga masuk" yg tampil, HASIL riset itu sendiri —
    // `TechDef.unlock` — tak pernah ditampilkan). +panel DETAIL tech terpilih.
    // **Overflow real ditemukan**: DETAIL (5 baris: blank+header+efek+biaya+depends) di Minimal
    // (60×24, area.height=20) bikin action hint TERDORONG HABIS keluar (dites `render_to_string`
    // dump: box tutup sblm hint kebagian ruang — 23 baris logis dibutuhkan vs 18 tersedia).
    // Ambang `DETAIL_MIN_HEIGHT` disembunyikan DETAIL bila area genuinely sempit — tree list
    // ("list tetap", pola M14.18) TAK PERNAH dikurangi, action hint SELALU dijaga tampil.
    if area.height >= DETAIL_MIN_HEIGHT
        && let Some(t) = ordered.get(sel)
    {
        lines.push(Line::raw(""));
        lines.push(Line::styled(
            format!("DETAIL: {} [{:?}]", t.id, tech_status(app, &t.id)),
            Style::default().fg(th.focus).bold(),
        ));
        lines.push(Line::styled(
            format!("Efek: {}", describe_unlock(&t.unlock)),
            Style::default().fg(th.text),
        ));
        lines.push(Line::styled(
            format!(
                "Biaya: {} Data, {}s",
                compact_num(t.data_cost),
                format_num(t.time_secs)
            ),
            Style::default().fg(th.text),
        ));
        let deps = if t.depends_on.is_empty() {
            "(tidak ada)".to_string()
        } else {
            t.depends_on.join(", ")
        };
        lines.push(Line::styled(
            format!("Depends on: {deps}"),
            Style::default().fg(th.text),
        ));
    }
    lines.push(Line::styled(
        "[j/k]Pilih  [Enter]Mulai riset  [p]Planet  [g]Galaxy",
        Style::default().fg(th.dim),
    ));

    // M15.11: "scroll bila tree > area" — dulu tak ada scroll SAMA SEKALI, `Paragraph` yg
    // konten-nya lebih panjang drpd area cuma TERPOTONG diam (baris kelebihan hilang senyap,
    // tak ada indikasi apa pun) — demo (7 tech) TAK PERNAH genuinely overflow di breakpoint
    // standar (dikonfirmasi screenshot M15.1-10), tp checklist minta INFRASTRUKTUR scroll utk
    // saat tech tree BERTAMBAH (bukan cuma demo). Scroll offset dihitung supaya baris tech
    // TERPILIH (`sel_line_idx`) SELALU kebagian ruang terlihat — kalau muat semua, scroll=0
    // (tak ada efek); kalau overflow, geser SEKADAR cukup bawa baris terpilih ke tepi bawah
    // area terlihat (bukan lompat jauh/pusat, biar konteks baris sekitar tetap kelihatan).
    let visible_h = area.height.saturating_sub(2) as usize; // minus block border atas+bawah.
    let scroll_y = if visible_h > 0 && lines.len() > visible_h {
        let max_scroll = lines.len() - visible_h;
        sel_line_idx
            .saturating_sub(visible_h.saturating_sub(1))
            .min(max_scroll)
    } else {
        0
    };

    // M13.10: `.wrap()` — ditemukan REAL clipping ("time: 600s"→"time: 6") di boundary 70w
    // (Compact minimum) & 60w (Minimal) tanpa wrap; baris AVAILABLE TECHS bisa >lebar sempit.
    let block = Block::bordered().border_style(Style::default().fg(th.dim));
    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((scroll_y as u16, 0)),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render_to_text(app: &App, w: u16, h: u16) -> String {
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        term.draw(|f| crate::ui::draw_view(f, app, "research"))
            .unwrap();
        let buf = term.backend().buffer().clone();
        (0..buf.area().height)
            .map(|y| {
                (0..buf.area().width)
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// M15.1: **gap real ditemukan** — dulu cuma tampil tech `is_available` (demo: 1/7, cuma
    /// `ext_efficiency` yg `depends_on: []`), tech LOCKED (mis. `ext_deep_core`/`manu_titanium`/
    /// `aero_warp_mk2` yg butuh `manu_steel` selesai dulu — demo msh ACTIVE, blm completed) TAK
    /// PERNAH tampil — bukan "tree" (struktur utuh), cuma potongan available. Dites LANGSUNG:
    /// tech locked genuinely tampil skrng, dikelompokkan per branch (4 header branch tampil).
    #[test]
    fn all_branches_shown_including_locked_techs_not_just_available() {
        let app = App::demo();
        let text = render_to_text(&app, 120, 40);

        for label in [
            "EXTRACTION",
            "MANUFACTURING",
            "AEROSPACE",
            "ASTROCARTOGRAPHY",
        ] {
            assert!(
                text.contains(label),
                "header branch {label} harus tampil (tree utuh, bukan cuma available): {text}"
            );
        }
        // ext_deep_core & aero_warp_mk2 butuh manu_steel SELESAI (demo msh ACTIVE, blm
        // completed) — LOCKED, tp harus TETAP tampil (tree penuh, beda dari "available only").
        assert!(
            text.contains("ext_deep_core"),
            "tech LOCKED (butuh manu_steel selesai) harus tetap tampil di tree: {text}"
        );
        assert!(
            text.contains("aero_warp_mk2"),
            "tech LOCKED (butuh manu_titanium) harus tetap tampil di tree: {text}"
        );
        // M15.2: tech AKTIF (manu_steel) SKRNG genuinely ikut dihitung+tampil DI daftar branch
        // (bukan disaring keluar spt M15.1's versi awal) — checklist minta 4 status (termasuk
        // "active") hidup BERSAMA dlm tree yg sama, bukan disembunyikan dari situ.
        assert!(
            text.contains("MANUFACTURING (2)"),
            "manu_steel (ACTIVE) HARUS ikut terhitung+tampil di tree MANUFACTURING: {text}"
        );
    }

    /// M15.2: **gap real ditemukan** — SEMUA tech (locked ATAU available) dulu tampil SAMA
    /// PERSIS `th.advanced`, tak ada beda visual sama sekali antara "bisa diriset sekarang" vs
    /// "masih terkunci". Dites LANGSUNG: `tech_status()` genuinely beda utk 4 kombinasi state
    /// (locked/available/active/done), marker teks jg beda (dites via render, bukan cuma logic
    /// unit — biar kebukti genuinely SAMPAI ke layar bukan cuma fungsi terisolasi benar).
    #[test]
    fn tech_status_reflects_real_state_and_shows_distinct_marker() {
        let mut app = App::demo();

        // ext_efficiency: depends_on kosong, blm completed, tak aktif -> Available.
        assert_eq!(tech_status(&app, "ext_efficiency"), TechStatus::Available);
        // manu_steel: sedang ACTIVE (demo default) -> Active.
        assert_eq!(tech_status(&app, "manu_steel"), TechStatus::Active);
        // manu_titanium: depends_on manu_steel yg BLM completed (msh active) -> Locked.
        assert_eq!(tech_status(&app, "manu_titanium"), TechStatus::Locked);

        let text = render_to_text(&app, 120, 40);
        assert!(
            text.contains("[OK] ext_efficiency"),
            "Available harus py marker [OK]: {text}"
        );
        assert!(
            text.contains("[>>] manu_steel"),
            "Active harus py marker [>>]: {text}"
        );
        assert!(
            text.contains("[--] manu_titanium"),
            "Locked harus py marker [--]: {text}"
        );

        // Selesaikan manu_steel manual (tanpa lewat sim tick) -> Done, manu_titanium jd Available.
        app.state.research.completed.insert("manu_steel".into());
        app.state.research.active = None;
        assert_eq!(tech_status(&app, "manu_steel"), TechStatus::Done);
        assert_eq!(tech_status(&app, "manu_titanium"), TechStatus::Available);
        let text2 = render_to_text(&app, 120, 40);
        assert!(
            text2.contains("[DN] manu_steel"),
            "Done harus py marker [DN]: {text2}"
        );
    }

    /// M15.3: **gap real ditemukan** — dulu progress ACTIVE cuma tampil 1 pct BLENDED
    /// (`data_frac.min(time_frac)`), angka Data invested/cost MENTAH (checklist eksplisit minta
    /// "Data invested + waktu") tak pernah ditampilkan. Dites LANGSUNG: manipulasi `active.data_
    /// invested`/`elapsed_secs` manual (demo default beda dari cost/time asli, biar KETAHUAN
    /// kalau kode SALAH baca field yg salah), konfirmasi genuinely tampil "Data: 300/500" dan
    /// "Time: 72/120s" — BUKAN cuma pct blended yg menyembunyikan bottleneck mana yg mentok.
    #[test]
    fn active_shows_raw_data_invested_and_time_separately() {
        let mut app = App::demo();
        app.state.research.active = Some(crate::game::state::ActiveResearch {
            tech_id: "manu_steel".into(),
            data_invested: 300.0,
            elapsed_secs: 72.0,
        });

        let text = render_to_text(&app, 120, 40);
        assert!(
            text.contains("Data: 300/500"),
            "Data invested/cost MENTAH harus tampil terpisah dari pct blended: {text}"
        );
        assert!(
            text.contains("Time: 72/120s"),
            "waktu elapsed/total MENTAH harus tampil terpisah dari pct blended: {text}"
        );
    }

    /// M15.4: **gap real ditemukan** — dulu tak ada cara liat efek/depends_on tech SEBELUM
    /// diriset. Dites LANGSUNG: DETAIL panel genuinely tampil (efek `unlock` manusiawi, biaya,
    /// depends_on) di area cukup lebar/tinggi (120×40).
    #[test]
    fn detail_panel_shows_effect_cost_and_dependencies_for_selected_tech() {
        let app = App::demo();
        let text = render_to_text(&app, 120, 40);

        assert!(
            text.contains("DETAIL: ext_deep_core"),
            "DETAIL harus tampil utk tech terpilih (default sel=0): {text}"
        );
        assert!(
            text.contains("Efek: Buka bangunan: deep_core_miner"),
            "efek `unlock` harus dideskripsikan manusiawi: {text}"
        );
        assert!(
            text.contains("Biaya: 8.0k Data, 1,200s"),
            "biaya harus tampil di DETAIL: {text}"
        );
        assert!(
            text.contains("Depends on: manu_steel"),
            "depends_on harus tampil di DETAIL: {text}"
        );
    }

    /// M15.4: **overflow real ditemukan** — DETAIL (5 baris) di Minimal (60×24, area.height=20)
    /// bikin action hint TERDORONG HABIS keluar. Dites LANGSUNG: DETAIL disembunyikan di bawah
    /// `DETAIL_MIN_HEIGHT`, action hint SELALU tampil (list tetap, pola M14.18).
    #[test]
    fn detail_hidden_when_area_too_short_but_hint_always_visible() {
        let app = App::demo();
        let text = render_to_text(&app, 60, 24);

        assert!(
            !text.contains("DETAIL:"),
            "DETAIL harus disembunyikan @60×24 (area terlalu sempit): {text}"
        );
        assert!(
            text.contains("[j/k]Pilih"),
            "action hint HARUS tetap tampil walau DETAIL disembunyikan: {text}"
        );
        // M15.13: branch header ("ASTROCARTOGRAPHY" dst) SENGAJA disembunyikan @Minimal
        // (60×24, flat list bukan tree) — cek tech ID langsung (bukan header) biar list tetap
        // genuinely dibuktikan LENGKAP tanpa bergantung ke header yg emang sengaja hilang.
        assert!(
            text.contains("astro_far_warp"),
            "tech list ('list tetap') TAK PERNAH dikurangi, tetap lengkap: {text}"
        );
    }

    /// M15.5: "Data/s total tampil" — dicek SUDAH terpenuhi by construction (`data_rate()` +
    /// header "Data/sec: +N"), tp cuma dites demo default (1 Research Lab L2 = tetap 10/s,
    /// TAK MEMBUKTIKAN genuinely SUM dari lebih dari 1 lab). Dites LANGSUNG: tambah 1 Research
    /// Lab L3 lagi (slot 2 demo kosong), konfirmasi "Data/sec" jd genuinely SUM (2+3)×5=25, BUKAN
    /// cuma tampil lab pertama saja (10) — bukti aggregate genuine, bukan kebetulan cocok.
    #[test]
    fn data_rate_header_shows_genuine_sum_across_multiple_labs() {
        let mut app = App::demo();
        let (gi, pi) = app.active_pi().expect("demo harus py planet aktif");
        app.state.galaxies[gi].planets[pi].factory_slots[2] = Some(crate::game::state::Factory {
            id: crate::game::state::FactoryId(99),
            building: crate::game::defs::BuildingId(
                app.content.buildings.id("research_lab").unwrap(),
            ),
            kind: crate::game::state::FactoryKind::ResearchLab,
            level: 3,
            enabled: true,
        });

        let text = render_to_text(&app, 120, 40);
        assert!(
            text.contains("Data/sec: +25"),
            "harus tampil SUM genuine (2+3)*5=25 dari 2 lab, bukan cuma lab pertama (10): {text}"
        );
    }

    /// M15.6: "garis dependency antar node (ASCII connector)" — dulu depends_on cuma kelihatan
    /// di panel DETAIL utk tech TERPILIH. Dites LANGSUNG @120×40 (Full lebar, MAIN 78w>=70):
    /// connector "└" genuinely tampil INLINE di baris tech yg py depends_on, tech TANPA
    /// depends_on (`ext_efficiency`) TAK tampil connector (jujur, bukan reka connector kosong).
    #[test]
    fn dependency_connector_shown_inline_at_wide_area() {
        let app = App::demo();
        let text = render_to_text(&app, 120, 40);

        assert!(
            text.contains("ext_deep_core") && text.contains("\u{2514}manu_steel"),
            "ext_deep_core (depends_on manu_steel) harus tampil connector: {text}"
        );
        assert!(
            !text.contains("ext_efficiency  cost: 4.0k Data  time: 600s\u{2514}"),
            "ext_efficiency (depends_on kosong) TAK boleh tampil connector kosong: {text}"
        );
    }

    /// M15.6: **overflow real ditemukan** — suffix connector di area SEMPIT (60×24 Minimal, 80×
    /// 30 Compact, 100×30 Full-minimum — SEMUA MAIN width <70) bikin baris WRAP, tree list
    /// SENDIRI kepotong (dikonfirmasi screenshot: `astro_far_warp` hilang total). Dites LANGSUNG:
    /// connector disembunyikan di bawah ambang, tree list ("list tetap") TETAP lengkap.
    #[test]
    fn dependency_connector_hidden_at_narrow_area_to_prevent_list_overflow() {
        // Cek substring SPESIFIK "\u{2514}manu_steel" (bukan cuma '\u{2514}' generik) — karakter
        // "└" JUGA genuinely dipakai sbg pojok kiri-bawah SETIAP `Block::bordered()` (border
        // outer/inner ADA di semua render, tak terhindarkan) — assertion generik akan SELALU
        // gagal (false positive), bukan bukti connector genuinely tampil/sembunyi.
        let app = App::demo();
        for (w, h) in [(60, 24), (80, 30), (100, 30)] {
            let text = render_to_text(&app, w, h);
            assert!(
                !text.contains("\u{2514}manu_steel"),
                "connector HARUS disembunyikan @{w}x{h} (MAIN <70w, wrap+list kepotong): {text}"
            );
            // M15.13: header branch ("ASTROCARTOGRAPHY") SENGAJA disembunyikan @Minimal
            // (60×24) — cek tech ID langsung (invarian SEBENARNYA: list tak kepotong),
            // bukan header yg genuinely beda per breakpoint sejak M15.13.
            assert!(
                text.contains("astro_far_warp"),
                "tech list ('list tetap') TAK PERNAH kepotong @{w}x{h}: {text}"
            );
        }
    }

    /// M15.7: "Warp Tier saat ini + next unlock" — `ship.warp_tier` ADA di state (dipakai
    /// genuine `research::tick`) tp research view tak py info ini sama sekali sblm ini. Dites
    /// LANGSUNG: `next_warp_unlock` genuinely cari tech WarpTier BLM completed dgn `n` TERKECIL
    /// (bukan tech pertama katalog) — konfirmasi demo (`aero_warp_mk2`=Tier6 < `aero_warp_mk3`=
    /// Tier10, keduanya blm completed) pilih yg TERKECIL, DAN genuinely tampil di render.
    #[test]
    fn warp_tier_shows_current_and_next_unlock_at_wide_area() {
        let app = App::demo();
        assert_eq!(
            next_warp_unlock(&app),
            Some(("aero_warp_mk2", 6)),
            "harus pilih tech WarpTier TERKECIL blm completed (mk2=6 < mk3=10)"
        );

        let text = render_to_text(&app, 120, 40);
        assert!(
            text.contains("Warp Tier: 3 | Next: aero_warp_mk2 -> Tier 6"),
            "Warp Tier saat ini + next unlock harus tampil: {text}"
        );
    }

    /// M15.7: **overflow real ditemukan** — +1 baris Warp Tier di Minimal (60×24) dorong action
    /// hint keluar (sama kelas M15.2/4). Dites LANGSUNG: baris disembunyikan di bawah
    /// `DETAIL_MIN_HEIGHT`, action hint SELALU tampil.
    #[test]
    fn warp_tier_line_hidden_at_narrow_area_hint_always_visible() {
        let app = App::demo();
        let text = render_to_text(&app, 60, 24);

        assert!(
            !text.contains("Warp Tier:"),
            "Warp Tier line harus disembunyikan @60x24 (area terlalu sempit): {text}"
        );
        assert!(
            text.contains("[j/k]Pilih"),
            "action hint HARUS tetap tampil walau Warp Tier line disembunyikan: {text}"
        );
    }

    /// M15.8: **jembatan end-to-end** — `app.rs`'s `research_keys` (baru) genuinely gerakkan
    /// `app.sel`, tp apakah `render()` DI SINI genuinely BACA `app.sel` yg sama utk nentuin
    /// DETAIL terpilih? Dites LANGSUNG lewat `handle_key` (bukan set `app.sel` manual). Urutan
    /// `all_techs_ordered()` (branch EXTRACTION dulu): indeks 0=`ext_deep_core`, 1=`ext_
    /// efficiency` — `j` sekali dari sel=0 harus pindah DETAIL ke indeks 1.
    #[test]
    fn navigation_via_handle_key_changes_detail_panel_selection() {
        let mut app = App::demo();
        app.view = "research".into();

        let before = render_to_text(&app, 120, 40);
        assert!(
            before.contains("DETAIL: ext_deep_core"),
            "sel=0 default harus tunjuk tech PERTAMA urutan (ext_deep_core): {before}"
        );

        app.handle_key(crossterm::event::KeyCode::Char('j'));

        let after = render_to_text(&app, 120, 40);
        assert!(
            after.contains("DETAIL: ext_efficiency"),
            "stlh 'j' via handle_key, DETAIL harus pindah ke tech KEDUA (ext_efficiency): {after}"
        );
        assert!(
            !after.contains("DETAIL: ext_deep_core"),
            "DETAIL LAMA (ext_deep_core) tak boleh tampil lagi stlh navigasi: {after}"
        );
    }

    /// M15.9: "+feedback" — dites LANGSUNG lewat `handle_key` (Enter, bukan panggil `research::
    /// start_research` manual): ACTIVE section+status marker tech itu genuinely UPDATE di
    /// render berikutnya (state read langsung tiap render, pola sama M14.15's upgrade feedback
    /// — bukan sistem toast baru).
    #[test]
    fn enter_via_handle_key_starts_research_with_immediate_visible_feedback() {
        let mut app = App::demo();
        app.view = "research".into();
        app.state.research.active = None;

        let before = render_to_text(&app, 120, 40);
        assert!(
            before.contains("ACTIVE: (none)"),
            "sblm Enter, ACTIVE harus tampil kosong (blm ada riset aktif): {before}"
        );

        app.handle_key(crossterm::event::KeyCode::Char('j')); // sel=1 -> ext_efficiency.
        app.handle_key(crossterm::event::KeyCode::Enter);

        let after = render_to_text(&app, 120, 40);
        assert!(
            after.contains("ACTIVE: ext_efficiency"),
            "stlh Enter via handle_key, ACTIVE harus genuinely update ke tech dimulai: {after}"
        );
        assert!(
            after.contains("[>>] ext_efficiency"),
            "status marker tech itu di tree jg harus genuinely jd Active [>>]: {after}"
        );
    }

    /// M15.10: `raster.rs` (dipakai `screenshot.sh`) DICEK TAK handle `Modifier::REVERSED` sama
    /// sekali (grep kosong, sama gap M13.6/M14.8) — screenshot PNG TAK BISA buktikan highlight
    /// "jelas" ini genuinely nempel. Dites langsung baca `Cell.modifier` dari Buffer: REVERSED
    /// HARUS ada TEPAT di baris tech terpilih (bukan di baris lain manapun).
    #[test]
    fn selected_tech_row_is_reversed_and_only_that_row() {
        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        let app = App::demo();
        term.draw(|f| crate::ui::draw_view(f, &app, "research"))
            .unwrap();
        let buf = term.backend().buffer().clone();

        let lines: Vec<String> = (0..buf.area().height)
            .map(|y| {
                (0..buf.area().width)
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect();
        let reversed_rows: Vec<usize> = (0..buf.area().height as usize)
            .filter(|&y| {
                (0..buf.area().width).any(|x| {
                    buf[(x, y as u16)]
                        .modifier
                        .contains(ratatui::style::Modifier::REVERSED)
                })
            })
            .collect();

        let sel_row = lines
            .iter()
            .position(|l| l.contains("ext_deep_core"))
            .expect("baris ext_deep_core (tech terpilih default sel=0) harus ada");
        assert_eq!(
            reversed_rows,
            vec![sel_row],
            "REVERSED harus TEPAT di baris terpilih (ext_deep_core), tak di baris lain: {lines:?}"
        );
    }

    /// M15.11: **gap real ditemukan** — dulu TAK ADA scroll sama sekali, `Paragraph` yg
    /// kontennya lebih panjang drpd area cuma TERPOTONG diam (baris kelebihan hilang senyap).
    /// Demo (7 tech) TAK PERNAH genuinely overflow di breakpoint STANDAR (120×40/80×30/60×24,
    /// dikonfirmasi screenshot M15.1-10) — dites di area SENGAJA dipersempit (120×14, di bawah
    /// `DETAIL_MIN_HEIGHT` jg — branch header ikut disembunyikan M15.13, tp msh genuinely
    /// overflow tanpa itu) biar overflow genuinely terjadi, konfirmasi scroll ikut `sel`: sel=0
    /// (atas) tak perlu scroll (tech pertama SUDAH terlihat), sel=tech TERAKHIR (astro_far_warp,
    /// jauh di bawah) HARUS scroll turun bawa baris itu ke dlm pandangan.
    #[test]
    fn scroll_keeps_selected_tech_visible_when_tree_overflows_area() {
        let mut app = App::demo();
        app.view = "research".into();

        let top_text = render_to_text(&app, 120, 14);
        assert!(
            top_text.contains("ext_deep_core"),
            "sel=0 (atas): tech PERTAMA harus genuinely terlihat tanpa scroll: {top_text}"
        );
        assert!(
            !top_text.contains("astro_far_warp"),
            "sel=0: area sempit harus genuinely overflow (astro_far_warp belum \
             terlihat) — kalau tampil, test ini tak membuktikan apa2: {top_text}"
        );

        for _ in 0..6 {
            app.handle_key(crossterm::event::KeyCode::Char('j'));
        }
        assert_eq!(app.sel, 6); // astro_far_warp, indeks terakhir.

        let bottom_text = render_to_text(&app, 120, 14);
        assert!(
            bottom_text.contains("astro_far_warp"),
            "sel=tech TERAKHIR: scroll HARUS bawa astro_far_warp ke dlm pandangan: {bottom_text}"
        );
    }

    /// M15.12: "Compact: tree ringkas" — dicek SUDAH terpenuhi by construction, EFEK SAMPING
    /// M15.6/11's width/height gating (bukan fitur baru terpisah): Compact (80×30, MAIN 64w)
    /// SUDAH < `DEPS_SUFFIX_MIN_WIDTH`(70) — connector dependency genuinely disembunyikan,
    /// tree jadi genuinely lbh ringkas drpd Full LEBAR tanpa kode tambahan. Dites LANGSUNG di
    /// UKURAN Compact PERSIS (80×30, bukan cuma nilai lebar generik) biar bukti terikat ke
    /// breakpoint NYATA yg dimaksud checklist, bukan cuma kebetulan lolos test lebar M15.6.
    #[test]
    fn compact_tree_is_condensed_without_dependency_connector_clutter() {
        let app = App::demo();
        let text = render_to_text(&app, 80, 30);

        assert!(
            !text.contains("\u{2514}manu_steel"),
            "Compact (80x30) harus genuinely ringkas — connector dependency TAK tampil: {text}"
        );
        // Struktur tree (branch grouping+status) TETAP utuh — "ringkas" bukan berarti
        // "hilang", beda dari Minimal's "list tech (no grafis tree)" (scope M15.13 terpisah).
        for label in [
            "EXTRACTION",
            "MANUFACTURING",
            "AEROSPACE",
            "ASTROCARTOGRAPHY",
        ] {
            assert!(
                text.contains(label),
                "struktur tree per branch HARUS tetap utuh di Compact (ringkas != hilang): {text}"
            );
        }
    }

    /// M15.13: **gap real ditemukan** — dulu branch header ("EXTRACTION (2)" dst) SELALU
    /// tampil di SEMUA breakpoint termasuk Minimal (60×24), padahal checklist minta Minimal
    /// genuinely TANPA struktur tree (flat list). Dites LANGSUNG: header branch TAK tampil
    /// @60×24, tp SEMUA 7 tech (termasuk yg LOCKED) tetap tampil (flat list, bukan "hilang").
    #[test]
    fn minimal_shows_flat_tech_list_without_branch_tree_headers() {
        let app = App::demo();
        let text = render_to_text(&app, 60, 24);

        for label in [
            "EXTRACTION",
            "MANUFACTURING",
            "AEROSPACE",
            "ASTROCARTOGRAPHY",
        ] {
            assert!(
                !text.contains(label),
                "branch header HARUS disembunyikan @60x24 (Minimal: flat list, no tree): {text}"
            );
        }
        for id in [
            "ext_deep_core",
            "ext_efficiency",
            "manu_steel",
            "manu_titanium",
            "aero_warp_mk2",
            "aero_warp_mk3",
            "astro_far_warp",
        ] {
            assert!(
                text.contains(id),
                "SEMUA 7 tech (termasuk locked) harus tetap tampil flat @60x24, id={id}: {text}"
            );
        }
    }

    /// M15.13: dikonfirmasi Compact (80×30) TIDAK terpengaruh — masih genuinely tampilkan
    /// branch header (bukan cuma Minimal yg disembunyikan tanpa sengaja jg matikan Compact).
    #[test]
    fn compact_still_shows_branch_tree_headers() {
        let app = App::demo();
        let text = render_to_text(&app, 80, 30);
        assert!(
            text.contains("EXTRACTION") && text.contains("MANUFACTURING"),
            "Compact (80x30) HARUS tetap tampilkan branch header (bukan ikut Minimal): {text}"
        );
    }

    /// M15.15: dibandingkan ref Riftborne `_rTKGm.png` — tabel BUILD/UPGRADE py kolom SEJAJAR
    /// (nama di-pad, angka rata). **Gap ditemukan**: nama tech beda panjang bikin "cost:"/
    /// "time:" TAK sejajar antar baris (pola sama M14.21's "angka rata-kanan" planet_view).
    /// Dites LANGSUNG: nama di-pad ke max lebar GLOBAL — `ext_efficiency`/`astro_far_warp`
    /// (14 char, TERPANJANG demo) vs `manu_steel` (10 char, dipad +4 spasi biar "cost:"
    /// genuinely mulai di KOLOM SAMA persis, dikonfirmasi hitung `wc -c` tiap id langsung,
    /// bukan tebak angka).
    #[test]
    fn tech_names_padded_to_align_cost_column() {
        let app = App::demo();
        let text = render_to_text(&app, 120, 40);

        assert!(
            text.contains("ext_deep_core   cost:"),
            "ext_deep_core (13 char, dipad +1 ke max 14) + 2 spasi tetap = 3 spasi total \
             sblm 'cost:': {text}"
        );
        assert!(
            text.contains("manu_steel      cost:"),
            "manu_steel (10 char, dipad +4 ke max 14) + 2 spasi tetap = 6 spasi total, \
             genuinely sejajar kolom SAMA dgn baris nama lebih panjang: {text}"
        );
    }
}
