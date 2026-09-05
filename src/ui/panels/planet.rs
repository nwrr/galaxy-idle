//! View PLANET: header planet + ringkasan nodes/factories/ship/research + aksi. `06` §Planet View.
#![allow(dead_code)]

use crate::app::App;
use crate::balance::{BASE_DATA_RATE, COST_NODE_LEVEL, GROWTH};
use crate::game::actions::{buildable_options, can_afford, resolve_cost};
use crate::game::defs::{BuildingKind, ResourceId};
use crate::game::economy::{extractor_output, upgrade_cost};
use crate::game::research;
use crate::game::state::{Factory, FactoryKind, Planet, ShipStatus};
use crate::ui::{biome_label, compact_num, format_num, progress_bar, theme, tier_color};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use std::collections::HashMap;

/// M14.18: ambang `info_area.height` di bawah mana SHIP+RESEARCH (ringkasan sekunder)
/// disembunyikan — Compact SEMPIT+PENDEK (mis. 70×24) kehabisan ruang, ditemukan via hitung
/// baris nyata (bukan tebakan). Nilai dikalibrasi via `render_to_text` empiris (lihat unit test
/// `no_content_lost_at_compact_minimum_boundary`), BUKAN cuma diasumsikan angka bulat "kelihatan
/// masuk akal".
const SHIP_RESEARCH_MIN_HEIGHT: u16 = 26;

/// M14.3: node dulu 1 baris gabung tanpa `richness` (`ResourceNode.richness` ADA di data, tak
/// pernah ditampilkan) + tanpa selektor (`app.sel` SUDAH dipakai nyata `planet_keys` utk pilih
/// node upgrade via tombol `1`, tp tak ada indikator VISUAL node mana yg sedang tertunjuk).
/// Skrng 1 baris/node: `► `/`  ` (selektor, sel diklem sama persis `planet_keys`'s
/// `sel.min(nodes-1)`) + nama + richness (`x{:.1}`, rentang balance 0.5-2.0×lvl-galaksi) + level.
/// M14.8: marker `►` polos ditemukan KURANG "jelas" (checklist eksplisit minta highlight
/// jelas) — dibandingkan `panels::menu`'s selector (M13.6, `Modifier::REVERSED`+bold) yg
/// jauh lbh menonjol. +`Modifier::REVERSED`+bold pd baris terpilih, konsisten pola OPTIONS.
/// M14.9: +biaya upgrade & afford indicator pd baris TERPILIH SAJA (cost cuma relevan utk
/// item yg akan di-upgrade bila tekan `1`/`2`, bukan tiap baris).
fn nodes_lines(
    app: &App,
    p: &Planet,
    th: &theme::Theme,
    gi: usize,
    pi: usize,
) -> Vec<Line<'static>> {
    let mut lines = vec![Line::styled(
        format!("RESOURCE NODES ({})", p.nodes.len()),
        Style::default().fg(th.basic).bold(),
    )];
    if p.nodes.is_empty() {
        return lines;
    }
    let sel = app.sel.min(p.nodes.len() - 1);
    let credits = app.content.resources.id("credits").map(ResourceId);
    // M14.21: "angka rata-kanan" (VISUAL_TARGETS.md, ref Riftborne `fmVy4y.png`/`9weFWl.png` —
    // tabel RESOURCES/Assets py kolom NUMERIK rata dgn nama di-pad) — nama node dulu panjang
    // BEDA-BEDA (Iron/Carbon/Water), richness+level "mengambang" tak sejajar antar baris. Nama
    // di-pad ke max lebar SEKSI INI (bukan lebar global tetap — biar tak buang ruang sia-sia
    // di planet lain yg py nama lebih pendek), richness+level jadi genuinely SEJAJAR 1 kolom.
    let name_w = p
        .nodes
        .iter()
        .map(|n| app.content.resources.get(n.resource.0).name.len())
        .max()
        .unwrap_or(0);
    for (i, n) in p.nodes.iter().enumerate() {
        let selected = i == sel;
        let marker = if selected { "\u{25ba} " } else { "  " };
        // M14.17: dulu HARDCODE `th.basic` utk SEMUA node terlepas tier resource asli — masked
        // krn 3 node demo Earth (Iron/Carbon/Water) SEMUA kebetulan Basic. Resource TIER LAIN
        // (Advanced/Rare, ADA di `data/resources.ron` — mis. Rare Crystals) akan salah warna
        // kalau planet lain py node begitu. Fix: `tier_color()` genuinely per-resource, sama
        // `resources.rs`'s RESOURCES table (konsisten SUMBER WARNA, bukan cuma "kelihatan mirip
        // hijau" kebetulan).
        let mut style =
            Style::default().fg(tier_color(th, app.content.resources.get(n.resource.0).tier));
        if selected {
            style = style.add_modifier(Modifier::REVERSED).bold();
        }
        let text = format!(
            "{marker}{:<name_w$} richness x{:.1} L{}",
            app.content.resources.get(n.resource.0).name,
            n.richness,
            n.level
        );
        let mut spans = vec![Span::styled(text, style)];
        if selected {
            let cost = vec![(
                credits.expect("resource 'credits' harus ada di data/resources.ron"),
                upgrade_cost(COST_NODE_LEVEL, n.level),
            )];
            let (label, afford) = cost_afford_label(app, &cost, gi, pi);
            push_cost_spans(&mut spans, label, afford, style, th);
        }
        lines.push(Line::from(spans));
    }
    lines
}

/// M14.9: checklist minta "afford indicator (**hijau/merah**)" — bukan cuma teks `[OK]`/`[--]`
/// SEWARNA baris induk (yg py warna tier resource/building, beda makna). Span TERPISAH biar
/// `[OK]`/`[--]` genuinely `th.good`/`th.alert`, sisanya (teks cost+label) ikut warna baris
/// induk — `REVERSED`/bold (M14.8) diwariskan ke SEMUA span biar highlight tetap utuh 1 baris.
fn push_cost_spans(
    spans: &mut Vec<Span<'static>>,
    label: String,
    afford: bool,
    base_style: Style,
    th: &theme::Theme,
) {
    spans.push(Span::styled(format!(" | Cost: {label} "), base_style));
    let status_style = base_style.fg(if afford { th.good } else { th.alert });
    spans.push(Span::styled(
        if afford { "[OK]" } else { "[--]" },
        status_style,
    ));
}

/// M14.9: ringkas label biaya "amt Nama amt Nama..." + status afford (`can_afford`, SAMA
/// PERSIS logic `try_pay`'s cek — bukan reka ambang baru, biar preview NEVER menyimpang dari
/// hasil tekan tombol beneran).
fn cost_afford_label(
    app: &App,
    cost: &[(ResourceId, f64)],
    gi: usize,
    pi: usize,
) -> (String, bool) {
    let parts: Vec<String> = cost
        .iter()
        .map(|(r, amt)| {
            format!(
                "{} {}",
                compact_num(*amt),
                app.content.resources.get(r.0).name
            )
        })
        .collect();
    let credits = app.content.resources.id("credits").map(ResourceId);
    let afford = can_afford(&app.state, gi, pi, cost, credits);
    (parts.join(" "), afford)
}

fn kind_label(k: FactoryKind) -> &'static str {
    match k {
        FactoryKind::Extractor { .. } => "Extractor",
        FactoryKind::Refinery { .. } => "Refinery",
        FactoryKind::ResearchLab => "Research Lab",
    }
}

/// M14.12: label kind utk `BuildingDef` TEMPLATE (beda dari `kind_label` di atas yg utk
/// `FactoryKind` runtime instance — `buildable_options` cuma pernah hasilkan 3 kind ini,
/// Storage/Special disaring duluan, jadi `unreachable!` aman bukan default menyesatkan).
fn building_kind_label(k: BuildingKind) -> &'static str {
    match k {
        BuildingKind::Extractor => "Extractor",
        BuildingKind::Refinery => "Refinery",
        BuildingKind::ResearchLab => "Research Lab",
        BuildingKind::Storage | BuildingKind::Special => {
            unreachable!("buildable_options sudah saring Storage/Special")
        }
    }
}

/// M14.12: render list kandidat building (`actions::buildable_options`) GANTI info_area penuh
/// selagi `app.build_picker` aktif — selektor `►`+REVERSED (pola sama M14.8), biaya+afford per
/// kandidat (pola sama M14.9, `resolve_cost` scale 1.0 krn level 1 baru/`GROWTH^0=1`).
fn render_build_picker(f: &mut Frame, app: &App, area: Rect, pi: usize, th: &theme::Theme) {
    let Some(picker) = app.build_picker else {
        return;
    };
    let options = buildable_options(&app.state, &app.content, pi);
    let mut lines = vec![
        Line::styled(
            "BUILD — pilih building",
            Style::default().fg(th.header).bold(),
        ),
        Line::raw(""),
    ];
    if options.is_empty() {
        lines.push(Line::styled(
            "(tak ada building yg bisa dibangun sekarang)",
            Style::default().fg(th.dim),
        ));
    }
    let (gi, _) = app
        .active_pi()
        .expect("render_build_picker cuma dipanggil saat planet aktif ada");
    for (i, &building) in options.iter().enumerate() {
        let selected = i == picker.sel.min(options.len().saturating_sub(1));
        let marker = if selected { "\u{25ba} " } else { "  " };
        let bdef = app.content.buildings.get(building.0);
        let mut style = Style::default().fg(th.advanced);
        if selected {
            style = style.add_modifier(Modifier::REVERSED).bold();
        }
        let text = format!("{marker}{} ({})", bdef.name, building_kind_label(bdef.kind));
        let mut spans = vec![Span::styled(text, style)];
        if selected {
            let cost = resolve_cost(&app.content, &bdef.base_cost, 1.0);
            let (label, afford) = cost_afford_label(app, &cost, gi, pi);
            push_cost_spans(&mut spans, label, afford, style, th);
        }
        lines.push(Line::from(spans));
    }
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "[j/k]Pilih  [Enter]Bangun  [Esc]Batal",
        Style::default().fg(th.dim),
    ));
    let block = Block::bordered().border_style(Style::default().fg(th.dim));
    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

/// M14.4: slot dulu 1 baris gabung `[Nama L{level}]`/`[Empty]` — tanpa `kind` (Extractor/
/// Refinery/Research Lab, `Factory.kind` ADA di data, tak pernah ditampilkan) + tanpa `enabled`
/// (`Factory.enabled` jg ADA, dipakai NYATA `economy::run_extractors`/`data_rate` filter, msh
/// senyap di UI) + tanpa selektor visual. Skrng 1 baris/slot — selektor `app.sel` LANGSUNG
/// (BUKAN `.min(len-1)` spt nodes_lines: `planet_keys`'s j/k SUDAH clamp `sel` ke `slots.len()`
/// langsung sblm dipakai upgrade factory, beda dari node yg re-clamp terpisah via tombol `1`).
/// M14.9: +biaya+afford pd slot TERISI terpilih (slot `[Empty]` TAK py upgrade cost — `upgrade_
/// factory` return `Err(SlotEmpty)` bila slot kosong, jadi jujur tak ditampilkan cost apa pun).
fn factories_lines(
    app: &App,
    p: &Planet,
    th: &theme::Theme,
    gi: usize,
    pi: usize,
) -> Vec<Line<'static>> {
    let mut lines = vec![Line::styled(
        format!("FACTORIES ({})", p.factory_slots.len()),
        Style::default().fg(th.advanced).bold(),
    )];
    // M14.21: "angka rata-kanan" (sama alasan `nodes_lines`) — nama building panjang BEDA-BEDA
    // (mis. "Steel Mill" vs "Research Lab"), level+kind "mengambang". Nama TERISI di-pad ke max
    // lebar SEKSI ini (slot `[+ Build]` tak ikut dihitung — format beda total, bukan nama).
    let name_w = p
        .factory_slots
        .iter()
        .flatten()
        .map(|fac| app.content.buildings.get(fac.building.0).name.len())
        .max()
        .unwrap_or(0);
    for (i, slot) in p.factory_slots.iter().enumerate() {
        let selected = i == app.sel;
        let marker = if selected { "\u{25ba} " } else { "  " };
        let text = match slot {
            Some(fac) => format!(
                "{marker}{:<name_w$} L{} {} {}",
                app.content.buildings.get(fac.building.0).name,
                fac.level,
                kind_label(fac.kind),
                if fac.enabled { "[ON]" } else { "[OFF]" }
            ),
            None => format!("{marker}[+ Build]"),
        };
        let mut style = Style::default().fg(th.advanced);
        if selected {
            style = style.add_modifier(Modifier::REVERSED).bold();
        }
        let mut spans = vec![Span::styled(text, style)];
        if selected && let Some(fac) = slot {
            if let Some(detail) = factory_detail(app, p, fac) {
                spans.push(Span::styled(format!(" \u{2192} {detail}"), style));
            }
            let bdef = app.content.buildings.get(fac.building.0);
            let cost = resolve_cost(&app.content, &bdef.base_cost, GROWTH.powi(fac.level as i32));
            let (label, afford) = cost_afford_label(app, &cost, gi, pi);
            push_cost_spans(&mut spans, label, afford, style, th);
        }
        lines.push(Line::from(spans));
    }
    lines
}

/// M14.16: "detail item terpilih" — `kind_label` cuma bilang "Refinery"/"Extractor" GENERIK,
/// TAK PERNAH tunjukkan resource/recipe SPESIFIK yg genuinely dikonfigurasi (`Factory.kind`
/// simpan `node`/`recipe` id, ADA di data, senyap di UI — pola sama "data hilang" M14.2-4/6/7).
/// Ditampilkan CUMA pd baris TERPILIH (bukan tiap slot — hemat baris, sama disiplin M14.9/10),
/// DIGABUNG ke span existing (bukan Line baru) biar TAK nambah baris vertikal sama sekali —
/// belajar LANGSUNG dari M14.11/M14.13's insiden overflow, hati-hati skrng SEBELUM nambah teks.
fn factory_detail(app: &App, p: &Planet, fac: &Factory) -> Option<String> {
    match fac.kind {
        FactoryKind::Extractor { node } => p
            .nodes
            .iter()
            .find(|n| n.id == node)
            .map(|n| app.content.resources.get(n.resource.0).name.clone()),
        FactoryKind::Refinery { recipe } => Some(app.content.recipes.get(recipe.0).id.clone()),
        FactoryKind::ResearchLab => None,
    }
}

fn data_rate(p: &Planet) -> f64 {
    p.factory_slots
        .iter()
        .flatten()
        .filter(|f| f.enabled && matches!(f.kind, FactoryKind::ResearchLab))
        .map(|f| f.level as f64 * BASE_DATA_RATE)
        .sum()
}

/// M14.10: net income/s per resource — TAK ADA sebelumnya di planet_view SAMA SEKALI (beda dari
/// M14.6's "data hilang", ini genuinely fitur baru diminta checklist). `resources.rs`'s
/// `extractor_rate_per_sec` (M13.1) cuma GROSS (dicatat jujur "tak netting refinery" — scope
/// item ITU soal format tabel doang). Di sini genuinely DIHITUNG net: extraksi (+) dari
/// Extractor enabled (formula SAMA `economy::run_extractors`/`extractor_output`, dry-run) DIKURANGI
/// konsumsi input Refinery enabled + DITAMBAH output-nya (SAMA formula `economy::run_refineries`,
/// recipe instan `craft_time_secs==0` doang — recipe berwaktu ditunda sama spt run_refineries).
/// **Simplifikasi jujur dicatat** (bukan disembunyikan): TAK simulasikan stock-gating (recipe di
/// `run_refineries` NYATA berhenti kalau input planet habis di tengah tick) — preview ini
/// asumsikan SEMUA refinery jalan penuh tiap tick, angka "kalau lancar", bukan garansi 100% match
/// simulasi nyata saat stockpile input kritis. Di luar scope 1 item UI utk simulasikan penuh.
fn net_income_per_sec(app: &App, p: &Planet) -> Vec<(ResourceId, f64)> {
    let completed = &app.state.research.completed;
    let blueprints = &app.state.prestige.blueprints;
    let mut net: HashMap<ResourceId, f64> = HashMap::new();
    for slot in p.factory_slots.iter().flatten() {
        if !slot.enabled || slot.level == 0 {
            continue;
        }
        match slot.kind {
            FactoryKind::Extractor { node } => {
                if let Some(n) = p.nodes.iter().find(|n| n.id == node) {
                    let mult = research::tech_mult(completed, &app.content, n.resource);
                    let out = extractor_output(slot.level, n.richness, mult);
                    *net.entry(n.resource).or_insert(0.0) += out;
                }
            }
            FactoryKind::Refinery { recipe } => {
                let rec = app.content.recipes.get(recipe.0);
                if rec.craft_time_secs != 0.0
                    || (rec.requires_blueprint && !blueprints.contains(&recipe))
                {
                    continue;
                }
                for (name, qty) in &rec.inputs {
                    if let Some(rid) = app.content.resources.id(name) {
                        *net.entry(ResourceId(rid)).or_insert(0.0) -= qty * slot.level as f64;
                    }
                }
                for (name, qty) in &rec.outputs {
                    if let Some(rid) = app.content.resources.id(name) {
                        *net.entry(ResourceId(rid)).or_insert(0.0) += qty * slot.level as f64;
                    }
                }
            }
            FactoryKind::ResearchLab => {}
        }
    }
    let mut v: Vec<(ResourceId, f64)> = net.into_iter().filter(|(_, r)| r.abs() > 1e-9).collect();
    v.sort_by_key(|(r, _)| r.0);
    v
}

fn net_income_line(app: &App, p: &Planet, th: &theme::Theme) -> Line<'static> {
    let rates = net_income_per_sec(app, p);
    if rates.is_empty() {
        return Line::styled(
            "NET INCOME/S: (tak ada produksi aktif)",
            Style::default().fg(th.dim),
        );
    }
    let mut spans = vec![Span::styled(
        "NET INCOME/S: ",
        Style::default().fg(th.header).bold(),
    )];
    for (i, (r, rate)) in rates.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        let name = &app.content.resources.get(r.0).name;
        let color = if *rate >= 0.0 { th.good } else { th.alert };
        spans.push(Span::styled(
            format!(
                "{name} {}{}",
                if *rate >= 0.0 { "+" } else { "" },
                compact_num(*rate)
            ),
            Style::default().fg(color),
        ));
    }
    Line::from(spans)
}

/// M14.11: bar kapasitas stockpile — TAK ADA sebelumnya di planet_view (fitur baru, beda kelas
/// dari M14.10's netting). Scope: resource yg PLANET INI produksi (via `p.nodes`, dedup) —
/// bukan SEMUA entry `p.stockpile` (bisa byk resource tak relevan planet ini, mis. dari trade/
/// event) — biar bar yg ditampilkan genuinely relevan konteks NODES yg sudah ditunjukkan di
/// atasnya, bukan daftar tak berhubungan. `progress_bar()` (M14.11: dipindah `research.rs`→
/// `ui/mod.rs`, `pub(crate)`, pola sama `compact_num`/`biome_label`/M13.7-M14.2) dipakai reuse.
/// **Bug real ditemukan+diperbaiki (2 tahap)**: versi pertama 1 BARIS/resource (4 baris
/// tambahan: header+3 resource) — dites di Compact (80×30, info_area LEBIH PENDEK drpd Full),
/// footer aksi "[j/k]Pilih..." KEPOTONG HABIS dari layar (Paragraph tanpa scroll, kelebihan
/// baris hilang diam² — bug KELAS SAMA M13.1/M13.3's overflow, kali ini MULTI-BARIS bukan
/// lebar). Fix tahap 1: SEMUA resource digabung 1 BARIS (pola sama `net_income_line`). MASIH
/// kepotong (dihitung PERSIS: 26 baris logis dibutuhkan, cuma 25 baris tersedia @80×30 — kurang
/// PAS 1 baris). Fix tahap 2: 1 `Line::raw("")` (jarak kosmetik sblm action hint, `render()`)
/// DIHAPUS — bukan info, cuma spasi, aman dikorbankan drpd action hint (JAUH lbh krusial)
/// hilang. Diverifikasi ULANG screenshot+snapshot teks, action hint terbukti TAMPIL @80×30.
fn stockpile_line(app: &App, p: &Planet, th: &theme::Theme) -> Line<'static> {
    let mut seen = Vec::new();
    let mut spans = vec![Span::styled(
        "STOCKPILE: ",
        Style::default().fg(th.header).bold(),
    )];
    let mut first = true;
    for n in &p.nodes {
        if seen.contains(&n.resource) {
            continue;
        }
        seen.push(n.resource);
        if !first {
            spans.push(Span::raw("  "));
        }
        first = false;
        let now = p.stockpile.get(&n.resource).copied().unwrap_or(0.0);
        let frac = if p.stockpile_cap > 0.0 {
            now / p.stockpile_cap
        } else {
            0.0
        };
        let pct = (frac.clamp(0.0, 1.0) * 100.0).round() as u32;
        spans.push(Span::styled(
            format!(
                "{} {} {pct}%",
                app.content.resources.get(n.resource.0).name,
                progress_bar(frac)
            ),
            Style::default().fg(th.basic),
        ));
    }
    Line::from(spans)
}

/// M14.6: RESEARCH line dulu cuma "Data/s" — tech AKTIF (`state.research.active`, dipakai
/// nyata `research.rs`'s view detail) tak ditunjukkan sama sekali di ringkasan planet_view,
/// padahal checklist minta "Data/s, aktif". Ringkas (BUKAN duplikasi progress bar detail
/// `research.rs` — itu tujuan view terpisah), cuma id tech, atau "(none)" jujur bila tak ada.
fn active_tech_label(app: &App) -> &str {
    app.state
        .research
        .active
        .as_ref()
        .map(|a| a.tech_id.as_str())
        .unwrap_or("(none)")
}

/// M14.7: **bug real ditemukan** — hint dulu STATIS "[1]Node [2]Upgrade [3]Factory [4]Lab
/// [5]Ship" tp dicek `app.rs`'s `planet_keys`: HANYA `1` (guard `nodes>0`) & `2`/Enter yg
/// genuinely tersambung ke aksi (`upgrade_node`/`upgrade_factory`) — `3`/`4`/`5` jatuh ke
/// `_ => {}` (no-op TOTAL, tak ada handler). Hint lama MENYESATKAN (janji aksi yg tak ada,
/// user bisa tekan 3/4/5 tanpa efek apa pun tanpa tahu kenapa). "jangan ubah mekanik" — TAK
/// menambah handler baru utk 3/4/5 (itu keputusan gameplay besar di luar scope UI), FIX via
/// hint jd KONTEKSTUAL: cuma tampilkan aksi yg BENAR2 match key handler nyata + guard `nodes>0`
/// sama persis `planet_keys`. `[j/k]` (navigasi sel) ditambah, konsisten pola
/// research.rs/galaxy_map.rs yg jg cantumkan hint navigasi bareng aksi.
/// M14.12: +`[3]Build` KONTEKSTUAL — cuma muncul bila slot factory TERPILIH kosong (guard
/// SAMA PERSIS `planet_keys`'s `KeyCode::Char('3') if slot_empty`), sama disiplin "hint match
/// realita kode" M14.7 — bukan ditampilkan permanen spt hint palsu lama.
fn action_hint(p: &Planet, sel: usize) -> String {
    let mut s = if p.nodes.is_empty() {
        "[j/k]Pilih [2]Upgrade".to_string()
    } else {
        "[j/k]Pilih [1]Node [2]Upgrade".to_string()
    };
    if p.factory_slots.get(sel).is_some_and(Option::is_none) {
        s.push_str(" [3]Build");
    }
    s
}

/// M14.13: sprite width HITUNG dari breakpoint `SpriteSize` NYATA (`sprite::SpriteSize::cells`:
/// lg=64, md=40, sm=20), bukan `Percentage(50)` buta — di lebar sempit, kasih sprite persentase
/// LEBIH drpd `Sm` butuhkan cuma SIA-SIA (tetap render "sm" sama persis), sementara info
/// (Min(30) floor) kehabisan ruang. Threshold "total_w >= X" dihitung: butuh `X` sprite + 30
/// info floor: lg perlu total>=94 (64+30), md perlu total>=70 (40+30), di bawah itu sprite cuma
/// dpt `sm`+padding (22) — SISA lebar (bisa jauh lbh besar drpd 30 floor) balik ke info.
/// M14.19: **bug real ditemukan** — Minimal SEMPIT (mis. 45×20, MAIN 43w): sprite TETAP dipaksa
/// 22w (sm) walau `total_w`(43) < 22+30(floor info)=52 — `Layout`'s `Min(30)` TERPAKSA dilanggar
/// (ratatui shrink proporsional saat total tak cukup), FACTORIES/Status/hint kepotong (dites
/// screenshot: "Research Lab" hilang total). "List prioritas" (checklist) ditegakkan: sprite
/// DISEMBUNYIKAN (0w) bila `total_w` tak cukup utk sprite+info floor SEKALIGUS — list SELALU
/// menang ruang drpd sprite dekoratif.
fn sprite_width_for(total_w: u16) -> u16 {
    const INFO_MIN: u16 = 30;
    if total_w >= 94 {
        64
    } else if total_w >= 70 {
        40
    } else if total_w >= 22 + INFO_MIN {
        22
    } else {
        0
    }
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    // M14.9: butuh index (gi,pi), bukan cuma ref `&Planet` — `can_afford`/`resolve_cost` cek
    // stockpile PLANET SPESIFIK (bukan agregat). `active_pi()` (dipakai jg `app.rs`'s
    // `planet_keys`, SAMA semantik `Planet` lama di sini: galaksi aktif+planet unlocked
    // pertama) menggantikan `active_planet()` lama (dihapus, kini superseded) biar 1 sumber
    // index yg konsisten dgn logic aksi nyata.
    let Some((gi, pi)) = app.active_pi() else {
        f.render_widget(
            Paragraph::new("Tidak ada planet aktif.")
                .block(Block::bordered().border_style(Style::default().fg(th.dim))),
            area,
        );
        return;
    };
    let p = &app.state.galaxies[gi].planets[pi];
    // M14.1: sprite_area PROPORSIONAL (`Percentage(50)`) — REVISI dari `Length(24)` FIXED M04.6
    // (comment asli: "Layout final baru dirapikan di M14"). Ditemukan NYATA: `Length(24)` FIXED
    // artinya `SpriteSize::for_area` (lg butuh w>=64, md w>=40) TAK PERNAH dpt lg/md brp pun
    // lebar terminal — sprite SELALU sm (20×10) walau MAIN 80 lebar di 120×40, kotak "size per
    // area" janji checklist TAK terpenuhi nyata. Fix: `Percentage(50)` dari MAIN 80w Full →
    // sprite ~40w (pas ambang md), info dpt ~40 (>=Min(30) floor, aman).
    // M14.13: `Percentage(50)` DIREVISI LAGI — bug real ditemukan di boundary Full-mode MINIMUM
    // (100×30, MAIN cuma 60w): Percentage(50) kasih sprite 30w (cukup, tp SIA-SIA lebih dari
    // `SpriteSize::Sm`'s 20w kebutuhan asli — sprite TETAP render "sm" di 30w SAMA PERSIS spt
    // di 22w, breakpoint md butuh 40w yg 30w tak capai) SEDANGKAN info Min(30) floor PAS PENUH
    // 30, konten wrap parah, OVERFLOW SEVERE (dikonfirmasi: SHIP/RESEARCH/Status/action hint
    // SEMUA hilang total, bukan cuma 1 baris spt M14.11). Fix: `sprite_width_for()` HITUNG sprite
    // width dari breakpoint `SpriteSize` NYATA (bukan persentase buta) — di lebar sempit, sprite
    // cuma dpt SM+padding (22), SISA lebar (jauh lbh besar) balik ke info. Di 120×40 (MAIN 80w)
    // hasil SAMA PERSIS drpd Percentage(50) lama (40w) — backward-compatible, tak regresi.
    let sprite_w = sprite_width_for(area.width);
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(30), Constraint::Length(sprite_w)])
        .split(area);
    let (info_area, sprite_area) = (cols[0], cols[1]);

    // M14.12: build picker aktif — GANTI seluruh info_area dgn list kandidat (bukan
    // dioverlay/dicampur konten normal — lebih sederhana & jelas: 1 mode fokus per waktu,
    // sprite kanan tetap tampil tak terganggu).
    if app.build_picker.is_some() {
        render_build_picker(f, app, info_area, pi, &th);
        let base = App::sprite_for_biome(p.biome);
        app.sprites.render(f, sprite_area, base);
        return;
    }

    let name_up = p.name.to_uppercase();
    let ship = &app.state.ship;
    let status = match ship.status {
        ShipStatus::Idle => "Status: Idle".to_string(),
        ShipStatus::Traveling {
            total_secs,
            elapsed_secs,
            ..
        } => {
            let left = (total_secs - elapsed_secs).max(0.0);
            format!("Status: Traveling ({}s left)", format_num(left))
        }
    };
    // M14.2: header dulu "Sector 001" — literal HARDCODE, `Planet` tak py field sektor apa pun
    // (dicek `game/state.rs`), bukan reka data palsu — DIGANTI 2 field REAL yg justru ada tp
    // tak pernah ditampilkan: `biome` (`biome_label`, dipindah dari `galaxy_map.rs` M14.2, shared
    // di `ui::mod`) + `distance` (AU abstrak, format sama `galaxy_map.rs` "d=X.X" biar konsisten).
    let mut lines = vec![Line::styled(
        format!(
            "{name_up} - {} (Lvl {}) - d={:.1}",
            biome_label(p.biome),
            p.tier,
            p.distance
        ),
        Style::default().fg(th.focus).bold(),
    )];
    lines.extend(nodes_lines(app, p, &th, gi, pi));
    lines.extend(factories_lines(app, p, &th, gi, pi));
    // M14.18: Compact NARROW+PENDEK (mis. 70×24) — bug real ditemukan: SHIP+RESEARCH+Status+
    // action hint SEMUA hilang total, bahkan STLH SHIP+RESEARCH disembunyikan msh kurang (dites
    // empiris, bukan dihitung tangan sekali lalu percaya). "List tetap" (checklist) ditegakkan
    // KETAT: NODES+FACTORIES (list INTI, tempat user upgrade/build — aksi utama planet_view)
    // TAK PERNAH dikurangi. NET INCOME/STOCKPILE (analisis SEKUNDER/derivatif, bukan aksi
    // langsung) DISEMBUNYIKAN bareng SHIP/RESEARCH di ambang sama — action hint (JAUH lbh
    // krusial, satu2nya cara tau aksi tersedia) SELALU dijaga tampil, prioritas tertinggi.
    if info_area.height >= SHIP_RESEARCH_MIN_HEIGHT {
        lines.push(net_income_line(app, p, &th));
        lines.push(stockpile_line(app, p, &th));
        lines.push(Line::styled(
            format!(
                "SHIP: WarpTier {} | Engine L{} Cargo L{} Scanner L{}",
                ship.warp_tier, ship.engine, ship.cargo, ship.scanner
            ),
            Style::default().fg(th.good),
        ));
        lines.push(Line::styled(
            format!(
                "RESEARCH: +{} Data/s | Active: {}",
                compact_num(data_rate(p)),
                active_tech_label(app)
            ),
            Style::default().fg(th.rare),
        ));
    }
    lines.push(Line::styled(status, Style::default().fg(th.neutral)));
    lines.push(Line::styled(
        action_hint(p, app.sel),
        Style::default().fg(th.dim),
    ));
    // M13.10: `.wrap()` — ditemukan REAL clipping di boundary 100×30 (Full-mode minimum):
    // sprite_area FIXED `Length(24)` (M04.6) menyisakan info_area sempit saat MAIN column
    // menyusut, baris NODES/FACTORIES/SHIP/aksi kepotong tanpa wrap.
    let info_block = Block::bordered().border_style(Style::default().fg(th.dim));
    f.render_widget(
        Paragraph::new(lines)
            .block(info_block)
            .wrap(Wrap { trim: false }),
        info_area,
    );

    let base = App::sprite_for_biome(p.biome);
    app.sprites.render(f, sprite_area, base);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    /// M14.3: `app.sel` dipakai NYATA `planet_keys` (tombol `1`) tp `render()` static/`App::demo()`
    /// (sel selalu 0) — screenshot TAK bisa buktikan marker `►` genuinely IKUT `sel` (sama gap
    /// kelas M13.6). Dites langsung: geser `app.sel`, marker HARUS pindah baris (bukan nempel
    /// baris pertama terus).
    fn marker_row(app: &App) -> Option<usize> {
        let mut term = Terminal::new(TestBackend::new(80, 20)).unwrap();
        term.draw(|f| render(f, app, f.area())).unwrap();
        let buf = term.backend().buffer().clone();
        (0..buf.area().height)
            .find(|&y| buf[(1, y)].symbol() == "\u{25ba}")
            .map(|y| y as usize)
    }

    /// M14.4: sama gap kelas M14.3 tp utk FACTORIES — `marker_row` (di atas) cuma nangkep
    /// occurrence PERTAMA (selalu di NODES, krn render lebih dulu); dipakai `last` di sini
    /// krn FACTORIES dirender SETELAH nodes (occurrence terakhir = slot factory, valid krn
    /// `app.sel` node re-clamp `min(len-1)` STABIL di sel>=2 sedang factory slot TETAP geser
    /// bebas 0..3, jadi occurrence terakhir yg berubah murni krn factory, bukan node).
    fn last_marker_row(app: &App) -> Option<usize> {
        let mut term = Terminal::new(TestBackend::new(80, 20)).unwrap();
        term.draw(|f| render(f, app, f.area())).unwrap();
        let buf = term.backend().buffer().clone();
        (0..buf.area().height)
            .rev()
            .find(|&y| buf[(1, y)].symbol() == "\u{25ba}")
            .map(|y| y as usize)
    }

    /// M14.8: `raster.rs` (dipakai `screenshot.sh`) DICEK TAK handle `Modifier::REVERSED` sama
    /// sekali (grep kosong) — sama persis gap M13.6, screenshot PNG TAK BISA buktikan highlight
    /// "jelas" ini genuinely nempel. Dites langsung baca `Cell.modifier` dari Buffer: REVERSED
    /// HARUS ada TEPAT di baris marker (bukan di baris lain manapun).
    fn reversed_rows(app: &App) -> Vec<usize> {
        let mut term = Terminal::new(TestBackend::new(80, 20)).unwrap();
        term.draw(|f| render(f, app, f.area())).unwrap();
        let buf = term.backend().buffer().clone();
        (0..buf.area().height)
            .filter(|&y| {
                (0..buf.area().width).any(|x| buf[(x, y)].modifier.contains(Modifier::REVERSED))
            })
            .map(|y| y as usize)
            .collect()
    }

    #[test]
    fn selected_node_and_factory_rows_are_reversed() {
        let mut app = App::demo();
        app.sel = 0;
        let node_row = marker_row(&app).unwrap();
        let factory_row = last_marker_row(&app).unwrap();
        let reversed = reversed_rows(&app);

        assert!(
            reversed.contains(&node_row),
            "baris node terpilih harus REVERSED, ditemukan: {reversed:?}"
        );
        assert!(
            reversed.contains(&factory_row),
            "baris factory terpilih harus REVERSED, ditemukan: {reversed:?}"
        );
        // M14.9: baris terpilih skrng +teks biaya, bisa WRAP jadi >1 baris visual (`Wrap`
        // ikutkan style KE SEMUA baris hasil wrap dari 1 `Line` sama) — jd total reversed BISA
        // >2 (bukan bug, konsekuensi wrap). Cek row 0 (header "RESOURCE NODES", SELALU pendek
        // tak wrap) TAK reversed — bukti REVERSED tak "bocor" ke baris tak terkait.
        assert!(
            !reversed.contains(&0),
            "header RESOURCE NODES (row 0) TAK boleh reversed, ditemukan: {reversed:?}"
        );
    }

    #[test]
    fn node_selector_follows_app_sel() {
        let mut app = App::demo();
        app.sel = 0;
        let row0 = marker_row(&app).expect("sel=0 harus ada marker");

        app.sel = 2;
        let row2 = marker_row(&app).expect("sel=2 harus ada marker (3 node demo)");

        assert_ne!(
            row0, row2,
            "marker harus PINDAH ikut app.sel, bukan statis di node pertama"
        );
    }

    #[test]
    fn factory_selector_follows_app_sel() {
        let mut app = App::demo();
        app.sel = 0;
        let row0 = last_marker_row(&app).expect("sel=0 harus ada marker factory");

        app.sel = 3;
        let row3 = last_marker_row(&app).expect("sel=3 harus ada marker (4 slot demo)");

        assert_ne!(
            row0, row3,
            "marker factory harus PINDAH ikut app.sel, bukan statis di slot pertama"
        );
    }

    /// M14.6: `active_tech_label` fallback "(none)" — `App::demo()` py active research
    /// (`manu_steel`) selalu, jadi cabang kosong tak kesentuh screenshot. Dites eksplisit
    /// (bukan diasumsikan aman) via manipulasi state langsung.
    #[test]
    fn active_tech_label_falls_back_honestly_when_none() {
        let mut app = App::demo();
        assert_eq!(active_tech_label(&app), "manu_steel");
        app.state.research.active = None;
        assert_eq!(active_tech_label(&app), "(none)");
    }

    /// M14.7: `action_hint` HARUS drop "[1]Node" bila `nodes` kosong (sama guard persis
    /// `planet_keys`'s `KeyCode::Char('1') if nodes > 0`) — `App::demo()` selalu py nodes>0
    /// jadi cabang ini tak kesentuh screenshot; dites via Planet buatan tangan.
    #[test]
    fn action_hint_omits_node_when_no_nodes() {
        use crate::game::state::{Biome, PlanetId, ResourceMap, UnlockReq};

        let no_nodes = Planet {
            id: PlanetId(0),
            name: "Test".into(),
            tier: 1,
            biome: Biome::Terran,
            distance: 1.0,
            unlocked: true,
            unlock_req: UnlockReq::None,
            nodes: vec![],
            factory_slots: vec![],
            stockpile: ResourceMap::new(),
            stockpile_cap: 1000.0,
        };
        assert_eq!(action_hint(&no_nodes, 0), "[j/k]Pilih [2]Upgrade");
    }

    /// M14.9: `App::demo()` py `credits: 0.0` — cabang `[OK]` (afford) TAK PERNAH kesentuh
    /// screenshot (selalu `[--]`). Dites eksplisit KEDUA cabang via `cost_afford_label`
    /// langsung (pure function, bukan parse teks render), naikkan `state.credits` cukup besar
    /// utk konfirmasi `[OK]` genuinely bisa true, bukan cuma diasumsikan format-nya benar.
    #[test]
    fn cost_afford_label_reflects_real_credits() {
        let mut app = App::demo();
        let (gi, pi) = app.active_pi().expect("demo harus py planet aktif");
        let credits = app.content.resources.id("credits").map(ResourceId).unwrap();
        let cost = vec![(credits, 100.0)];

        app.state.credits = 0.0;
        let (_, afford_no) = cost_afford_label(&app, &cost, gi, pi);
        assert!(!afford_no, "credits 0 harus TAK afford");

        app.state.credits = 1_000_000.0;
        let (_, afford_yes) = cost_afford_label(&app, &cost, gi, pi);
        assert!(afford_yes, "credits 1jt harus AFFORD cost 100");
    }

    /// M14.10: dihitung TANGAN dari `data/`: Mining Drill L4 (Extractor node Iron richness 1.5)
    /// → `BASE_EXTRACTOR_RATE(1.0)×level(4)×richness(1.5)×tech_mult(1.0)=6.0` Iron/s. Steel Mill
    /// L2 (Refinery recipe `steel_mill`: inputs Iron 2.0+Carbon 1.0 → outputs Steel 1.0, instant)
    /// jalan `level=2`×/tick: konsumsi Iron 4.0+Carbon 2.0, produksi Steel 2.0. Net: Iron
    /// 6.0-4.0=+2.0, Carbon 0-2.0=-2.0, Steel +2.0 — dites match PERSIS angka ini (bukan cuma
    /// "ada nilai", memverifikasi FORMULA genuinely benar bukan kebetulan plausible).
    #[test]
    fn net_income_matches_hand_calculated_values() {
        let app = App::demo();
        let (gi, pi) = app.active_pi().expect("demo harus py planet aktif");
        let p = &app.state.galaxies[gi].planets[pi];
        let net = net_income_per_sec(&app, p);

        let get = |name: &str| -> f64 {
            let rid = app.content.resources.id(name).unwrap();
            net.iter()
                .find(|(r, _)| r.0 == rid)
                .map(|(_, v)| *v)
                .unwrap_or(0.0)
        };
        assert!(
            (get("iron") - 2.0).abs() < 1e-9,
            "Iron net harus +2.0, dpt {}",
            get("iron")
        );
        assert!(
            (get("carbon") - (-2.0)).abs() < 1e-9,
            "Carbon net harus -2.0, dpt {}",
            get("carbon")
        );
        assert!(
            (get("steel") - 2.0).abs() < 1e-9,
            "Steel net harus +2.0, dpt {}",
            get("steel")
        );
    }

    /// M14.11: `App::demo()` seed stockpile Iron=850 (dari `app.rs`'s seed array, cap=100_000)
    /// — dihitung tangan: 850/100_000=0.85% → dibulatkan 1%. Carbon/Water TAK di seed (0.0),
    /// harus jujur 0%. Dites teks gabungan span (bukan cuma "ada bar", angka PERSIS dikunci).
    #[test]
    fn stockpile_line_shows_correct_percentages() {
        let app = App::demo();
        let (gi, pi) = app.active_pi().expect("demo harus py planet aktif");
        let p = &app.state.galaxies[gi].planets[pi];
        let line = stockpile_line(&app, p, &theme::theme(app.state.settings.theme));
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();

        assert!(text.contains("Iron"), "harus sebut Iron: {text}");
        assert!(
            text.contains("1%"),
            "Iron 850/100_000 harus bulat 1%: {text}"
        );
        assert!(text.contains("Carbon"), "harus sebut Carbon: {text}");
        assert!(text.contains("Water"), "harus sebut Water: {text}");
        assert!(
            text.matches("0%").count() >= 2,
            "Carbon+Water tak di-seed, harus jujur 0%: {text}"
        );
    }

    /// M14.12: `App::demo()` `build_picker` selalu `None` — `screenshot.sh` (dipakai iterasi
    /// lain) TAK PUNYA cara inject state ini (sama gap kelas M13.6/M14.3/M14.8: state dinamis
    /// tak kesentuh CLI screenshot statis). Dites lgs render buffer teks: buka picker (slot 2
    /// kosong demo), konfirmasi header+kandidat (mining_drill dkk)+cost+hint genuinely tampil.
    /// M14.18: dulu panggil `render()` LANGSUNG (BYPASS `draw_view`'s shell dispatch, area
    /// LEBIH BESAR drpd real Full-mode MAIN column) — diperbaiki rute lewat `render_to_text`
    /// (pipeline NYATA), konsisten sama fix `no_content_lost_at_full_mode_minimum_boundary`.
    #[test]
    fn build_picker_renders_candidates_and_hint() {
        let mut app = App::demo();
        app.sel = 2; // slot Empty (demo: Mining Drill/Steel Mill/Empty/Research Lab).
        app.build_picker = Some(crate::app::BuildPicker { slot: 2, sel: 0 });

        let text = render_to_text(&app, 120, 30);

        assert!(text.contains("BUILD"), "harus tampil header BUILD: {text}");
        assert!(
            text.contains("Mining Drill"),
            "kandidat tier-1 harus tampil: {text}"
        );
        assert!(
            text.contains("Cost:"),
            "biaya kandidat terpilih harus tampil: {text}"
        );
        assert!(
            text.contains("[Enter]Bangun") && text.contains("[Esc]Batal"),
            "hint konfirmasi/batal harus tampil: {text}"
        );
    }

    /// M14.12: `action_hint` +`[3]Build` bikin baris hint LEBIH PANJANG (38 char, dulu ~29) —
    /// dicek RISIKO REGRESI overflow SAMA KELAS M14.11 (budget baris di Compact 80×30 PAS
    /// PERSIS 25/25 sblm ini). Dites LANGSUNG: pilih slot KOSONG (sel=2, trigger "[3]Build"
    /// muncul) @80×30, konfirmasi hint TETAP genuinely tampil (bukan diasumsikan aman krn
    /// "cuma nambah beberapa char"). M14.18: diperbaiki rute lewat `render_to_text` (pipeline
    /// NYATA `draw_view`) — versi lama panggil `render()` langsung, area lebih besar drpd
    /// real Compact MAIN column, FALSE CONFIDENCE (lihat catatan `render_to_text`).
    #[test]
    fn action_hint_with_build_fits_at_compact_width() {
        let mut app = App::demo();
        app.sel = 2; // slot Empty → action_hint tambah "[3]Build".

        let text = render_to_text(&app, 80, 30);

        assert!(
            text.contains("[3]Build"),
            "slot kosong terpilih harus tampilkan hint [3]Build: {text}"
        );
        assert!(
            text.contains("[j/k]Pilih") && text.contains("[2]Upgrade"),
            "hint LENGKAP (bukan cuma sebagian krn kepotong) harus tampil: {text}"
        );
    }

    /// **Bug real ditemukan (M14.18)**: versi SEBELUMNYA panggil `render(f, app, f.area())`
    /// LANGSUNG — BYPASS `crate::ui::draw_view`'s shell dispatch (RESOURCES sidebar 22w+OPTIONS
    /// sidebar 18w+status bar+footer TAK PERNAH disubtraksi). Ini artinya SEMUA test M14.13/15/
    /// 16/17 yg pakai helper ini menguji `render()` dgn AREA JAUH LEBIH BESAR drpd yg genuinely
    /// diterima di real usage (mis. "100×30" test = 100×30 PENUH utk info+sprite, padahal
    /// MAIN column REAL cuma ~60×25 stlh shell_full subtraksi sidebar) — FALSE CONFIDENCE, test
    /// lulus tp tak membuktikan real pipeline. Fix: rute lewat `crate::ui::draw_view` (pipeline
    /// NYATA dipakai `screenshot.sh`/gameplay), bukan panggil `render()` planet.rs langsung.
    fn render_to_text(app: &App, w: u16, h: u16) -> String {
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        term.draw(|f| crate::ui::draw_view(f, app, "planet_view"))
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

    /// M14.13: **bug real ditemukan** di boundary Full-mode MINIMUM (100×30, MAIN 60w): sprite
    /// `Percentage(50)` dulu kasih sprite 30w (SIA-SIA lebih dari `Sm`'s kebutuhan 20w — tetap
    /// render "sm" sama persis) sedang info Min(30) floor PAS PENUH tanpa slack — overflow
    /// SEVERE (dikonfirmasi manual: SHIP/RESEARCH/Status/action hint SEMUA hilang total). Fix
    /// `sprite_width_for()` (hitung dari breakpoint `SpriteSize` nyata, bukan persentase buta),
    /// ditambah hapus 1 blank-line kosmetik.
    /// **BUG TEST INFRASTRUKTUR ditemukan M14.18**: versi test INI sendiri dulu pakai
    /// `render_to_text` yg (SEBELUM diperbaiki) panggil `render()` LANGSUNG bypass `draw_view`
    /// — area JAUH lebih besar drpd real Full-mode MAIN column, test SELALU lulus tanpa
    /// genuinely membuktikan apa pun (FALSE CONFIDENCE 2 iterasi, M14.16/17 luput). Setelah
    /// `render_to_text` diperbaiki rute lewat `draw_view` NYATA, test ini GENUINELY GAGAL
    /// (M14.16's tambahan "→ Iron" bikin overflow lagi @100×30 height=25) — diperbaiki via
    /// `SHIP_RESEARCH_MIN_HEIGHT` dinaikkan 24→26 (M14.18's mekanisme sendiri), skrng SHIP/
    /// RESEARCH/NET INCOME/STOCKPILE disembunyikan JUGA di 100×30 (trade-off SADAR, bukan
    /// "tak ada yg hilang" lagi — NODES/FACTORIES/Status/hint = jaminan INTI yg dites di sini).
    #[test]
    fn no_content_lost_at_full_mode_minimum_boundary() {
        let app = App::demo();
        let text = render_to_text(&app, 100, 30);
        assert!(
            text.contains("RESOURCE NODES") && text.contains("FACTORIES"),
            "list INTI (NODES/FACTORIES) HARUS tampil @100×30 — 'list tetap' TAK PERNAH \
             dikorbankan: {text}"
        );
        assert!(
            text.contains("Status: Idle"),
            "Status HARUS tampil @100×30 (boundary Full-mode minimum): {text}"
        );
        assert!(
            text.contains("[j/k]Pilih") && text.contains("[1]Node") && text.contains("[2]Upgrade"),
            "action hint LENGKAP HARUS tampil @100×30, bukan hilang/kepotong: {text}"
        );
    }

    /// M14.19: **bug real ditemukan** — Minimal SEMPIT (45×20, MAIN 43w): sprite dulu TETAP
    /// dipaksa 22w (sm) walau `total_w`(43) < 22+30(info floor)=52, `Min(30)` TERLANGGAR
    /// (ratatui shrink proporsional), FACTORIES (mis. "Research Lab") + Status + action hint
    /// kepotong (dikonfirmasi screenshot). "List prioritas" (checklist) ditegakkan: sprite
    /// disembunyikan (0w) di bawah ambang, list SELALU menang ruang. Dites 2 sisi: `sprite_
    /// width_for` sendiri (pure logic, ambang PERSIS 52) + render 45×20 genuinely lengkap.
    #[test]
    fn sprite_width_hides_below_threshold_giving_list_full_width() {
        assert_eq!(
            sprite_width_for(51),
            0,
            "51 < 52 (22+30 floor) harus sembunyikan sprite"
        );
        assert_eq!(
            sprite_width_for(52),
            22,
            "52 == ambang PERSIS harus tampil sm(22)"
        );
        assert_eq!(sprite_width_for(69), 22, "69 (Compact tersempit -1) msh sm");

        let app = App::demo();
        let text = render_to_text(&app, 45, 20);
        assert!(
            text.contains("RESOURCE NODES") && text.contains("FACTORIES"),
            "list INTI HARUS tampil @45×20 (Minimal sempit): {text}"
        );
        assert!(
            text.contains("Research Lab L2"),
            "slot FACTORIES terakhir (Research Lab) TAK boleh kepotong @45×20: {text}"
        );
        assert!(
            text.contains("Status: Idle") && text.contains("[j/k]Pilih"),
            "Status+action hint HARUS tampil @45×20, bukan hilang krn sprite makan ruang: {text}"
        );
    }

    /// M14.15: "aksi upgrade feedback (level naik terlihat)" — dicek dulu apakah SUDAH
    /// terpenuhi by construction (`factories_lines`/`nodes_lines` baca `fac.level`/`n.level`
    /// LANGSUNG dari state tiap render, tak ada cache/delay) SEBELUM menambah sistem feedback
    /// baru (toast/flash) yg TAK ADA presedennya di codebase ini (`particles`/`galaxy_anim`
    /// cuma utk kosmetik, bukan feedback aksi). Dites LANGSUNG: render SEBELUM upgrade tampil
    /// level LAMA, upgrade via `actions::upgrade_factory` (real, bukan simulasi kosong), render
    /// SESUDAH tampil level BARU — DAN level lama TAK ADA lagi di teks (bukan cuma "level baru
    /// nongol", tp genuinely GANTI, bukti render bukan stale/cache).
    #[test]
    fn upgrade_feedback_visible_immediately_in_next_render() {
        let mut app = App::demo();
        let (_, pi) = app.active_pi().expect("demo harus py planet aktif");
        app.state.credits = 1_000_000.0;

        let before = render_to_text(&app, 120, 40);
        assert!(
            before.contains("Steel Mill L2"),
            "sblm upgrade harus tampil level LAMA (L2): {before}"
        );

        crate::game::actions::upgrade_factory(&mut app.state, &app.content, pi, 1)
            .expect("upgrade harus sukses (credits cukup)");

        let after = render_to_text(&app, 120, 40);
        assert!(
            after.contains("Steel Mill L3"),
            "sesudah upgrade harus tampil level BARU (L3) di render berikutnya: {after}"
        );
        assert!(
            !after.contains("Steel Mill L2"),
            "level LAMA (L2) TAK boleh tampil lagi stlh upgrade: {after}"
        );
    }

    /// M14.15: simetris test factory di atas, utk NODE (`actions::upgrade_node`, tombol `1`).
    #[test]
    fn node_upgrade_feedback_visible_immediately_in_next_render() {
        let mut app = App::demo();
        let (_, pi) = app.active_pi().expect("demo harus py planet aktif");
        app.state.credits = 1_000_000.0;

        // M14.21: "Iron" di-pad `name_w` (max nama node demo, "Carbon"=6) — bukan lagi 1 spasi
        // tunggal antara nama+"richness". Cek substring LONGGAR (nama+angka, bukan spasi persis)
        // biar tak rapuh thd perubahan padding di masa depan.
        let before = render_to_text(&app, 120, 40);
        assert!(
            before.contains("Iron") && before.contains("richness x1.5 L4"),
            "sblm upgrade harus tampil level LAMA (L4): {before}"
        );

        crate::game::actions::upgrade_node(&mut app.state, pi, 0)
            .expect("upgrade node harus sukses (credits cukup)");

        let after = render_to_text(&app, 120, 40);
        assert!(
            after.contains("richness x1.5 L5"),
            "sesudah upgrade harus tampil level BARU (L5): {after}"
        );
        assert!(
            !after.contains("richness x1.5 L4"),
            "level LAMA (L4) TAK boleh tampil lagi: {after}"
        );
    }

    /// M14.16: "detail item terpilih" — `kind_label` dulu cuma "Extractor"/"Refinery" GENERIK,
    /// resource/recipe SPESIFIK yg genuinely dikonfigurasi (`Factory.kind`'s `node`/`recipe`)
    /// tak pernah ditampilkan. Dites SEMUA 3 kind demo Earth: Mining Drill (Extractor→node Iron)
    /// harus tampil "Iron"; Steel Mill (Refinery→recipe steel_mill) harus tampil "steel_mill";
    /// Research Lab (ResearchLab, tak py binding) harus `None` (jujur, bukan reka detail kosong).
    /// M14.18: dites LANGSUNG panggil `factory_detail()` (pure logic), BUKAN via render teks
    /// penuh — `render_to_text` (skrng genuinely rute `draw_view`, fix M14.18) bikin teks BISA
    /// wrap (`Wrap` M13.10) di TITIK MANA PUN (mis. persis di antara panah "→" dan "Iron"),
    /// exact-contiguous-string check jadi rapuh thd wrap wajar (BUKAN bug UI) — test fungsi
    /// LOGIC langsung menghindari kerapuhan ini sepenuhnya, konsisten pola `stockpile_line_
    /// shows_correct_percentages`.
    #[test]
    fn factory_detail_shows_bound_resource_or_recipe_when_selected() {
        let app = App::demo();
        let (gi, pi) = app.active_pi().expect("demo harus py planet aktif");
        let p = &app.state.galaxies[gi].planets[pi];

        let drill = p.factory_slots[0].expect("slot 0 Mining Drill");
        assert_eq!(
            factory_detail(&app, p, &drill),
            Some("Iron".to_string()),
            "Extractor (Mining Drill) harus tampil resource terikat (Iron)"
        );

        let mill = p.factory_slots[1].expect("slot 1 Steel Mill");
        assert_eq!(
            factory_detail(&app, p, &mill),
            Some("steel_mill".to_string()),
            "Refinery (Steel Mill) harus tampil recipe terikat (steel_mill)"
        );

        let lab = p.factory_slots[3].expect("slot 3 Research Lab");
        assert_eq!(
            factory_detail(&app, p, &lab),
            None,
            "ResearchLab tak py binding, harus jujur None (bukan reka detail kosong)"
        );
    }

    /// M14.17: bug real ditemukan — `nodes_lines` dulu HARDCODE `th.basic` utk SEMUA node,
    /// masked krn 3 node demo Earth (Iron/Carbon/Water) SEMUA kebetulan tier `Basic` (screenshot
    /// tak bisa buktikan perbedaan tier krn tak ada node non-Basic di demo). Dites LANGSUNG:
    /// tambah node Rare Crystals (tier `Rare`, dicek `data/resources.ron`) ke planet, konfirmasi
    /// warna FG baris-nya genuinely `th.rare` (BUKAN `th.basic` yg dulu selalu dipaksa).
    #[test]
    fn node_color_matches_actual_resource_tier_not_hardcoded_basic() {
        use crate::game::state::{NodeId, ResourceNode};

        let mut app = App::demo();
        let (gi, pi) = app.active_pi().expect("demo harus py planet aktif");
        let rare_id = ResourceId(app.content.resources.id("rare_crystals").unwrap());
        app.state.galaxies[gi].planets[pi].nodes.push(ResourceNode {
            id: NodeId(99),
            resource: rare_id,
            richness: 1.0,
            level: 1,
        });
        // sel tetap 0 (Iron) — node Rare Crystals baru (indeks 3) TAK terpilih, biar cek fg
        // bersih tanpa campur `Modifier::REVERSED`.
        assert_eq!(app.sel, 0);

        let th = theme::theme(app.state.settings.theme);
        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term.draw(|f| render(f, &app, f.area())).unwrap();
        let buf = term.backend().buffer().clone();

        let lines: Vec<String> = (0..buf.area().height)
            .map(|y| {
                (0..buf.area().width)
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect();
        let row = lines
            .iter()
            .position(|l| l.contains("Rare Crystals"))
            .expect("baris 'Rare Crystals' harus ada di render") as u16;
        let col = lines[row as usize].find("Rare Crystals").unwrap() as u16;

        assert_eq!(
            buf[(col, row)].fg,
            th.rare,
            "node Rare Crystals harus warna th.rare, BUKAN th.basic yg dulu dipaksa"
        );
    }
}
