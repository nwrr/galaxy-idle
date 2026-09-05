//! Panel RESOURCES: agregat stockpile galaksi aktif per tier (BASIC/ADVANCED/RARE). `06`.
#![allow(dead_code)]

use crate::app::App;
use crate::game::defs::{ResourceId, ResourceTier};
use crate::game::economy::extractor_output;
use crate::game::research;
use crate::game::state::{FactoryKind, Planet};
use crate::ui::{compact_num, fmt_hms, theme, tier_color};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

/// Jumlahkan stockpile semua planet galaksi aktif per ResourceId (urutan stabil).
fn aggregate(app: &App) -> Vec<(ResourceId, f64)> {
    use std::collections::HashMap;
    let mut sum: HashMap<u32, f64> = HashMap::new();
    if let Some(g) = app
        .state
        .galaxies
        .iter()
        .find(|g| g.id == app.state.active_galaxy)
    {
        for p in &g.planets {
            for (res, amt) in &p.stockpile {
                *sum.entry(res.0).or_insert(0.0) += amt;
            }
        }
    }
    let mut v: Vec<(ResourceId, f64)> = sum.into_iter().map(|(h, a)| (ResourceId(h), a)).collect();
    v.sort_by_key(|(r, _)| r.0);
    v
}

/// M13.7: `compact_num` (k/M) — konsisten dgn tabel full-mode `render()` di file ini (kuantitas
/// resource SAMA, dulu beda format: compact strip pakai `format_num` koma-ribuan, full pakai
/// k/M — inkonsistensi nyata utk data identik).
fn tier_line(app: &App, agg: &[(ResourceId, f64)], want: ResourceTier) -> String {
    agg.iter()
        .filter(|(r, v)| *v > 0.0 && app.content.resources.get(r.0).tier == want)
        .map(|(r, v)| {
            format!(
                "[{}: {}]",
                app.content.resources.get(r.0).name,
                compact_num(*v)
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Versi Compact: ringkas seluruh tier jadi **satu baris** (`06` §Responsive).
pub fn render_compact(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let agg = aggregate(app);
    let mut spans = vec![Span::styled("RES ", Style::default().fg(th.header).bold())];
    for (label, tier, col) in [
        ("B:", ResourceTier::Basic, th.basic),
        ("A:", ResourceTier::Advanced, th.advanced),
        ("R:", ResourceTier::Rare, th.rare),
    ] {
        spans.push(Span::styled(label, Style::default().fg(col).bold()));
        spans.push(Span::styled(
            format!("{} ", tier_line(app, &agg, tier)),
            Style::default().fg(col),
        ));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Planet aktif (unlocked pertama galaksi aktif) — sama pola `panels::planet::active_planet`,
/// diduplikasi (M13.1: `render()` skrng per-planet bukan agregat multi-planet, krn `Cap`
/// (`stockpile_cap`) mekanik PER-PLANET seragam — agregat lintas planet bikin "Cap" tabel
/// ambigu/tak bermakna; `render_compact` msh agregat multi-planet, blm disentuh, itu M13.8).
fn active_planet(app: &App) -> Option<&Planet> {
    app.state
        .galaxies
        .iter()
        .find(|g| g.id == app.state.active_galaxy)
        .and_then(|g| g.planets.iter().find(|p| p.unlocked))
}

/// Laju produksi KOTOR /detik utk `res` di planet `p` — jumlah `extractor_output` tiap
/// Extractor enabled yg node-nya match `res`, pakai `research::tech_mult` NYATA (formula SAMA
/// PERSIS `economy::run_extractors`/`sim::tick`, dry-run tanpa mutasi state — bukan model baru).
/// **Simplifikasi disengaja dicatat**: TAK netting konsumsi input refinery (perlu dry-run
/// recipe evaluation lbh kompleks, di luar scope M13.1 — item ini soal FORMAT tabel, bukan
/// model ekonomi baru). Resource yg jg dikonsumsi refinery bisa tampak rate lbh tinggi drpd
/// net riil; batasan jujur, bukan disembunyikan.
fn extractor_rate_per_sec(app: &App, p: &Planet, res: ResourceId) -> f64 {
    let completed = &app.state.research.completed;
    p.factory_slots
        .iter()
        .flatten()
        .filter(|f| f.enabled && f.level > 0)
        .filter_map(|f| match f.kind {
            FactoryKind::Extractor { node } => p
                .nodes
                .iter()
                .find(|n| n.id == node)
                .filter(|n| n.resource == res)
                .map(|n| {
                    let mult = research::tech_mult(completed, &app.content, res);
                    extractor_output(f.level, n.richness, mult)
                }),
            _ => None,
        })
        .sum()
}

/// RESOURCES tabel (M13.1, ref `riftborne/42UN3F.png` §RESOURCES): Resource/Now/Cap/▲per-jam/
/// ETA per baris, rata-kanan pd baris statistik, warna per tier. Planet AKTIF saja (bukan
/// agregat galaksi) — lihat `active_planet()`.
///
/// M13.10: jumlah resource ditampilkan DIBATASI sesuai `area.height` NYATA (bukan asumsi tetap
/// muat) — ditemukan REAL di boundary Full-mode minimum (100×30, body cuma ~25 baris utk
/// SELURUH kolom kiri): dgn 7 resource demo × 3 baris = 22 baris FIX, overflow diam² (Rare
/// Crystals/Dark Matter hilang tanpa indikasi, sama kelas bug M13.1/M13.3). Fix: hitung max
/// resource yg MUAT dari `area.height` (1 header + N×3), sisanya "+N lain" (pola sama Assets
/// M13.3) — proporsional ke tinggi SESUNGGUHNYA, bukan hardcode 22 yg cuma pas di 120×40.
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let Some(p) = active_planet(app) else {
        f.render_widget(
            Paragraph::new(Line::styled(
                "RESOURCES",
                Style::default().fg(th.header).bold(),
            )),
            area,
        );
        return;
    };
    let cap = p.stockpile_cap;
    let mut stock: Vec<(ResourceId, f64)> = p.stockpile.iter().map(|(r, v)| (*r, *v)).collect();
    stock.sort_by_key(|(r, _)| r.0);

    let width = area.width as usize;
    let max_shown = ((area.height as usize).saturating_sub(1) / 3).max(1);
    let truncated = stock.len().saturating_sub(max_shown);
    let mut lines = vec![Line::styled(
        "RESOURCES",
        Style::default().fg(th.header).bold(),
    )];
    for (res, now) in stock.into_iter().take(max_shown) {
        let def = app.content.resources.get(res.0);
        let color = tier_color(&th, def.tier);
        let rate_h = extractor_rate_per_sec(app, p, res) * 3600.0;
        let eta = if rate_h > 0.0 && now < cap {
            fmt_hms(((cap - now) / rate_h * 3600.0).round() as u64)
        } else {
            "--:--:--".to_string()
        };
        // 3 baris TETAP per resource (nama/now-cap/rate-eta) — disengaja, BUKAN 1 baris
        // wrap-bila-perlu: lebar terpanjang mungkin ("999.9k/999.9k +999.9k/h 99:99:99") tetap
        // bisa lampaui kolom kiri (~20 usable) walau sudah `compact_num`, & wrap OTOMATIS
        // ratatui memakan baris EKSTRA TAK TERDUGA (`Paragraph` tak scroll — kelebihan baris
        // dari estimasi terpotong DIAM² dari `area`, ditemukan nyata: Rare Crystals/Dark
        // Matter hilang total dari layar). Format tetap 3-baris = jumlah baris PASTI diprediksi.
        lines.push(Line::styled(
            def.name.clone(),
            Style::default().fg(color).bold(),
        ));
        let now_cap = format!("{}/{}", compact_num(now), compact_num(cap));
        lines.push(Line::styled(
            format!("{now_cap:>width$}"),
            Style::default().fg(color),
        ));
        let rate_eta = format!(
            "{}{}/h {}",
            if rate_h >= 0.0 { "+" } else { "" },
            compact_num(rate_h),
            eta
        );
        lines.push(Line::styled(
            format!("{rate_eta:>width$}"),
            Style::default().fg(color),
        ));
    }
    if truncated > 0 {
        lines.push(Line::styled(
            format!("+{truncated} resource lain"),
            Style::default().fg(th.dim),
        ));
    }
    f.render_widget(Paragraph::new(lines), area);
}
