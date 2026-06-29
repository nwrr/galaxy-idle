//! View PLANET: header planet + ringkasan nodes/factories/ship/research + aksi. `06` §Planet View.
#![allow(dead_code)]

use crate::app::App;
use crate::balance::BASE_DATA_RATE;
use crate::game::state::{FactoryKind, Planet, ShipStatus};
use crate::ui::{format_num, theme};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

fn active_planet(app: &App) -> Option<&Planet> {
    app.state
        .galaxies
        .iter()
        .find(|g| g.id == app.state.active_galaxy)
        .and_then(|g| g.planets.iter().find(|p| p.unlocked))
}

fn nodes_line(app: &App, p: &Planet) -> String {
    let parts: Vec<String> = p
        .nodes
        .iter()
        .map(|n| {
            format!(
                "{} L{}",
                app.content.resources.get(n.resource.0).name,
                n.level
            )
        })
        .collect();
    format!("RESOURCE NODES ({}): {}", p.nodes.len(), parts.join("  "))
}

fn factories_line(app: &App, p: &Planet) -> String {
    let parts: Vec<String> = p
        .factory_slots
        .iter()
        .map(|slot| match slot {
            Some(fac) => {
                format!(
                    "[{} L{}]",
                    app.content.buildings.get(fac.building.0).name,
                    fac.level
                )
            }
            None => "[Empty]".to_string(),
        })
        .collect();
    format!("FACTORIES ({}): {}", p.factory_slots.len(), parts.join(" "))
}

fn data_rate(p: &Planet) -> f64 {
    p.factory_slots
        .iter()
        .flatten()
        .filter(|f| f.enabled && matches!(f.kind, FactoryKind::ResearchLab))
        .map(|f| f.level as f64 * BASE_DATA_RATE)
        .sum()
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let th = theme::theme(app.state.settings.theme);
    let Some(p) = active_planet(app) else {
        f.render_widget(
            Paragraph::new("Tidak ada planet aktif.").block(Block::bordered()),
            area,
        );
        return;
    };
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
    let lines = vec![
        Line::styled(
            format!("{name_up} - Sector 001 (Lvl {})", p.tier),
            Style::default().fg(th.focus).bold(),
        ),
        Line::raw(""),
        Line::styled(nodes_line(app, p), Style::default().fg(th.basic)),
        Line::styled(factories_line(app, p), Style::default().fg(th.advanced)),
        Line::styled(
            format!(
                "SHIP: WarpTier {} | Engine L{} Cargo L{} Scanner L{}",
                ship.warp_tier, ship.engine, ship.cargo, ship.scanner
            ),
            Style::default().fg(th.good),
        ),
        Line::styled(
            format!("RESEARCH: +{} Data/s", format_num(data_rate(p))),
            Style::default().fg(th.rare),
        ),
        Line::styled(status, Style::default().fg(th.neutral)),
        Line::raw(""),
        Line::styled(
            "[1]Node [2]Upgrade [3]Factory [4]Lab [5]Ship",
            Style::default().fg(th.dim),
        ),
    ];
    f.render_widget(Paragraph::new(lines).block(Block::bordered()), area);
}
