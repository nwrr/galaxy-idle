//! UI ratatui: `draw_view` dispatch per view, panel, theme, layout.
//!
//! `06-ui.md`. UI hanya **membaca** snapshot `App`/`GameState`, tak memutasi.
//! `buffer_to_text` dipakai snapshot headless (`agent/test/README.md`).
#![allow(dead_code)]

pub mod layout;
pub mod panels;
pub mod theme;

use crate::app::App;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::Span;
use ratatui::widgets::Block;

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

/// Render satu view ke frame. `view`: main_menu | planet_view | galaxy_map | research | warp.
pub fn draw_view(f: &mut Frame, app: &App, view: &str) {
    match view {
        "main_menu" => draw_main_menu(f, app),
        "planet_view" => draw_planet_view(f, app),
        other => draw_placeholder(f, other),
    }
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

fn draw_main_menu(f: &mut Frame, app: &App) {
    let th = theme::theme(app.state.settings.theme);
    let area = f.area();
    let outer = Block::bordered().title(Span::styled(
        " STELLAR IDLE ",
        Style::default().fg(th.header).bold(),
    ));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let rows = Layout::vertical([Constraint::Length(5), Constraint::Min(0)]).split(inner);
    panels::resources::render(f, app, rows[0]);

    let cols = Layout::horizontal([Constraint::Length(15), Constraint::Min(0)]).split(rows[1]);
    let side = Layout::vertical([Constraint::Min(0), Constraint::Length(7)]).split(cols[0]);
    panels::menu::render(f, app, side[0]);
    panels::shortcuts::render(f, app, side[1]);
    panels::main_view::render(f, app, cols[1]);
}

fn draw_planet_view(f: &mut Frame, app: &App) {
    let th = theme::theme(app.state.settings.theme);
    let area = f.area();
    let outer = Block::bordered().title(Span::styled(
        " STELLAR IDLE ",
        Style::default().fg(th.header).bold(),
    ));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let cols = Layout::horizontal([Constraint::Length(15), Constraint::Min(0)]).split(inner);
    let side = Layout::vertical([Constraint::Min(0), Constraint::Length(7)]).split(cols[0]);
    panels::menu::render(f, app, side[0]);
    panels::shortcuts::render(f, app, side[1]);
    panels::planet::render(f, app, cols[1]);
}

fn draw_placeholder(f: &mut Frame, view: &str) {
    let area = f.area();
    let b = Block::bordered().title(Span::styled(
        format!(" STELLAR IDLE — {view} (TODO) "),
        Style::default().bold(),
    ));
    f.render_widget(b, area);
}
