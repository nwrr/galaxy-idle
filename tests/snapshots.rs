//! Snapshot golden test: render view ke TestBackend → bandingkan `agent/test/snapshots/`.
//! `UPDATE_SNAPSHOTS=1 cargo test` menulis ulang golden (keputusan sadar).

use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn render_to_string(view: &str, w: u16, h: u16) -> String {
    let backend = TestBackend::new(w, h);
    let mut term = Terminal::new(backend).unwrap();
    let app = galaxy_idle::demo_app();
    term.draw(|f| galaxy_idle::ui::draw_view(f, &app, view))
        .unwrap();
    let lines = galaxy_idle::ui::buffer_to_text(term.backend().buffer());
    lines.join("\n")
}

fn check(view: &str, w: u16, h: u16, golden: &str) {
    let got = render_to_string(view, w, h);
    let path = format!("agent/test/snapshots/{golden}");
    if std::env::var("UPDATE_SNAPSHOTS").is_ok() {
        std::fs::write(&path, format!("{got}\n")).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        got.trim_end(),
        want.trim_end(),
        "snapshot {golden} mismatch"
    );
}

#[test]
fn main_menu() {
    check("main_menu", 120, 40, "main_menu_120x40.txt");
}

#[test]
fn planet_view() {
    check("planet_view", 80, 30, "planet_view_80x30.txt");
}

#[test]
fn research() {
    check("research", 120, 40, "research_120x40.txt");
}

#[test]
fn warp() {
    check("warp", 120, 40, "warp_120x40.txt");
}

#[test]
fn galaxy_map() {
    check("galaxy_map", 120, 40, "galaxy_map_120x40.txt");
}

// -- Responsiveness: main_menu di tiap breakpoint (06 §Responsive) --

#[test]
fn main_menu_compact() {
    check("main_menu", 80, 30, "main_menu_80x30.txt");
}

#[test]
fn main_menu_minimal() {
    check("main_menu", 60, 24, "main_menu_60x24.txt");
}
