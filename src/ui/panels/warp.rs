//! View WARP NAVIGATION (prestige): galaksi target + requirement, estimasi Warp Core, peringatan
//! reset + catatan anchor, opsi initiate. `06-ui.md` §Warp, `04-prestige.md` §5.
#![allow(dead_code)]

use crate::app::App;
use crate::game::defs::ResourceId;
use crate::game::prestige::{total_value, warp_cores_gain, warp_threshold};
use crate::game::state::GameState;
use crate::ui::{compact_num, theme};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

/// Total resource `res` di seluruh planet galaksi aktif (untuk cek requirement).
fn have(state: &GameState, res: ResourceId) -> f64 {
    state
        .galaxies
        .iter()
        .find(|g| g.id == state.active_galaxy)
        .map(|g| {
            g.planets
                .iter()
                .map(|p| p.stockpile.get(&res).copied().unwrap_or(0.0))
                .sum()
        })
        .unwrap_or(0.0)
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let content = &app.content;
    let cur = app
        .state
        .galaxies
        .iter()
        .find(|g| g.id == app.state.active_galaxy);
    let cur_name = cur.map(|g| g.name.as_str()).unwrap_or("—");
    let cur_level = cur.map(|g| g.level).unwrap_or(0);
    let target = app.state.prestige.galaxy_level_reached + 1;

    let mut lines: Vec<Line> = vec![
        Line::styled("WARP NAVIGATION", Style::default().fg(th.focus).bold()),
        Line::styled(
            format!("Current: {cur_name} (Lvl {cur_level})"),
            Style::default().fg(th.neutral),
        ),
        Line::styled(
            format!("Target:  Outer Galaxy (Lvl {target})"),
            Style::default().fg(th.good).bold(),
        ),
        Line::raw(""),
        Line::styled("REQUIREMENT", Style::default().fg(th.header).bold()),
    ];

    let reqs = warp_threshold(content, target);
    let mut all_ok = !reqs.is_empty();
    for (r, amt) in &reqs {
        let got = have(&app.state, *r);
        let ok = got >= *amt;
        all_ok &= ok;
        lines.push(Line::styled(
            format!(
                "  {}: {} / {} {}",
                content.resources.get(r.0).name,
                compact_num(got),
                compact_num(*amt),
                if ok { "[OK]" } else { "[--]" }
            ),
            Style::default().fg(if ok { th.good } else { th.alert }),
        ));
    }

    let cores = warp_cores_gain(total_value(&app.state, content));
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        format!("Estimated Warp Cores gained: {}", compact_num(cores as f64)),
        Style::default().fg(th.rare),
    ));
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "Warning: Factories in current sector will be reset.",
        Style::default().fg(th.alert).bold(),
    ));
    lines.push(Line::styled(
        "Milky Way (Lvl 0) remains active as Anchor.",
        Style::default().fg(th.dim),
    ));
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        format!(
            "[1] INITIATE WARP JUMP {}   [2] Back",
            if all_ok { "[READY]" } else { "[LOCKED]" }
        ),
        Style::default().fg(if all_ok { th.good } else { th.dim }),
    ));

    let block = Block::bordered().border_style(Style::default().fg(th.dim));
    f.render_widget(Paragraph::new(lines).block(block), area);
}
