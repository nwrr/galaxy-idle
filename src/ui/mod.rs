//! UI ratatui: `draw_view` dispatch per view, panel, theme, layout.
//!
//! `06-ui.md`. UI hanya **membaca** snapshot `App`/`GameState`, tak memutasi.
//! `buffer_to_text` dipakai snapshot headless (`agent/test/README.md`).
#![allow(dead_code)]

pub mod galaxy_anim;
pub mod layout;
pub mod panels;
pub mod particles;
pub mod portrait;
pub mod sprite;
pub mod theme;

use crate::app::App;
use crate::ui::layout::{LayoutMode, layout_mode};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
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
        _ => None,
    }
}

/// Render satu view ke frame, responsif per breakpoint (`06` §Responsive).
/// `view`: main_menu | planet_view | galaxy_map | research | warp.
pub fn draw_view(f: &mut Frame, app: &App, view: &str) {
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

fn outer_block(app: &App) -> Block<'static> {
    let th = theme::theme(app.state.settings.theme);
    Block::bordered().title(Span::styled(
        " STELLAR IDLE ",
        Style::default().fg(th.header).bold(),
    ))
}

/// Full (≥100×30): strip RESOURCES (opsional) + sidebar (menu+shortcuts) + panel detail.
fn shell_full(f: &mut Frame, app: &App, main: MainFn, show_res: bool) {
    let outer = outer_block(app);
    let inner = outer.inner(f.area());
    f.render_widget(outer, f.area());

    let body = if show_res {
        let rows = Layout::vertical([Constraint::Length(5), Constraint::Min(0)]).split(inner);
        panels::resources::render(f, app, rows[0]);
        rows[1]
    } else {
        inner
    };
    let cols = Layout::horizontal([Constraint::Length(15), Constraint::Min(0)]).split(body);
    let side = Layout::vertical([Constraint::Min(0), Constraint::Length(7)]).split(cols[0]);
    panels::menu::render(f, app, side[0]);
    panels::shortcuts::render(f, app, side[1]);
    main(f, app, cols[1]);
}

/// Compact (≥70w): RESOURCES satu baris + sidebar tipis (menu saja) + panel detail.
fn shell_compact(f: &mut Frame, app: &App, main: MainFn, show_res: bool) {
    let outer = outer_block(app);
    let inner = outer.inner(f.area());
    f.render_widget(outer, f.area());

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
}

/// Minimal (<70w): tab bar atas + panel detail lebar penuh + footer shortcut.
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
    let txt = "g:Galaxy p:Planet r:Research m:Merchant ?:Help q:Quit";
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

fn draw_placeholder(f: &mut Frame, view: &str) {
    let area = f.area();
    let b = Block::bordered().title(Span::styled(
        format!(" STELLAR IDLE — {view} (TODO) "),
        Style::default().bold(),
    ));
    f.render_widget(b, area);
}
