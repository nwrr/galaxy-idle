//! Panel RESOURCES: agregat stockpile galaksi aktif per tier (BASIC/ADVANCED/RARE). `06`.
#![allow(dead_code)]

use crate::app::App;
use crate::game::defs::{ResourceId, ResourceTier};
use crate::ui::{format_num, theme};
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

fn tier_line(app: &App, agg: &[(ResourceId, f64)], want: ResourceTier) -> String {
    agg.iter()
        .filter(|(r, v)| *v > 0.0 && app.content.resources.get(r.0).tier == want)
        .map(|(r, v)| {
            format!(
                "[{}: {}]",
                app.content.resources.get(r.0).name,
                format_num(*v)
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

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let agg = aggregate(app);
    let mut rare = tier_line(app, &agg, ResourceTier::Rare);
    let special = tier_line(app, &agg, ResourceTier::Special);
    if !special.is_empty() {
        if !rare.is_empty() {
            rare.push(' ');
        }
        rare.push_str(&special);
    }
    let lines = vec![
        Line::styled("RESOURCES", Style::default().fg(th.header).bold()),
        Line::from(vec![
            Span::styled("BASIC:    ", Style::default().fg(th.basic).bold()),
            Span::styled(
                tier_line(app, &agg, ResourceTier::Basic),
                Style::default().fg(th.basic),
            ),
        ]),
        Line::from(vec![
            Span::styled("ADVANCED: ", Style::default().fg(th.advanced).bold()),
            Span::styled(
                tier_line(app, &agg, ResourceTier::Advanced),
                Style::default().fg(th.advanced),
            ),
        ]),
        Line::from(vec![
            Span::styled("RARE:     ", Style::default().fg(th.rare).bold()),
            Span::styled(rare, Style::default().fg(th.rare)),
        ]),
    ];
    f.render_widget(Paragraph::new(lines), area);
}
