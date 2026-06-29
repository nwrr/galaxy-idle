//! View RESEARCH: terminal riset — Data/sec + total, riset aktif (progress bar), available techs.
//! `06-ui.md` §Research, `03-progression.md` §5.
#![allow(dead_code)]

use crate::app::App;
use crate::game::defs::TechDef;
use crate::game::research;
use crate::ui::{format_num, theme};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

/// Data/sec dari ResearchLab di galaksi aktif.
fn data_rate(app: &App) -> f64 {
    app.state
        .galaxies
        .iter()
        .find(|g| g.id == app.state.active_galaxy)
        .map(research::data_rate)
        .unwrap_or(0.0)
}

/// Bar `[####......]` lebar 10 dari fraksi 0..1. `06` §Progress Bars.
fn progress_bar(frac: f64) -> String {
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

/// Baris riset aktif (header + progress bar), atau placeholder bila tak ada.
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
    // Completion butuh Data DAN waktu → progress = fraksi pembatas (yang terkecil).
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
            format!("{} {pct}% ({}s left)", progress_bar(p), format_num(left)),
            Style::default().fg(th.good),
        ),
    ]
}

fn frac(cur: f64, max: f64) -> f64 {
    if max > 0.0 { cur / max } else { 1.0 }
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let content = &app.content;
    let mut lines: Vec<Line> = vec![
        Line::styled("RESEARCH TERMINAL", Style::default().fg(th.focus).bold()),
        Line::styled(
            format!(
                "Data/sec: +{}   Data total: {}",
                format_num(data_rate(app)),
                format_num(app.state.research.data)
            ),
            Style::default().fg(th.rare),
        ),
        Line::raw(""),
    ];
    lines.extend(active_lines(app, &th));
    lines.push(Line::raw(""));

    let active_id = app
        .state
        .research
        .active
        .as_ref()
        .map(|a| a.tech_id.as_str());
    let avail: Vec<&TechDef> = content
        .techs
        .iter()
        .filter(|t| research::is_available(&app.state, content, &t.id))
        .filter(|t| Some(t.id.as_str()) != active_id)
        .collect();
    lines.push(Line::styled(
        format!("AVAILABLE TECHS ({})", avail.len()),
        Style::default().fg(th.header).bold(),
    ));
    for t in &avail {
        lines.push(Line::styled(
            format!(
                "  {}  [{:?}]  cost: {} Data  time: {}s",
                t.id,
                t.branch,
                format_num(t.data_cost),
                format_num(t.time_secs)
            ),
            Style::default().fg(th.advanced),
        ));
    }
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "[j/k]Pilih  [Enter]Mulai riset  [p]Planet  [g]Galaxy",
        Style::default().fg(th.dim),
    ));

    f.render_widget(Paragraph::new(lines).block(Block::bordered()), area);
}
