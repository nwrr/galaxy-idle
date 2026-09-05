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

// M15.14: golden dikelompokkan per view jadi sub-folder (`main_menu/`, `planet_view/`,
// `research/`) — CONVENTIONS.md §1 batasi maks 10 file/folder, nambah 2 breakpoint `research`
// bikin folder flat lama (11 file) lewat batas. `warp` msh 1 file (blm py breakpoint lain)
// jadi tetap di root. M20.6: `galaxy_map` skrng py 3 breakpoint -> dipindah `galaxy_map/`
// sub-folder (pola SAMA `planet_view`/`research`), `120x40.txt` lama di-`git mv`.

#[test]
fn main_menu() {
    check("main_menu", 120, 40, "main_menu/120x40.txt");
}

#[test]
fn planet_view() {
    check("planet_view", 80, 30, "planet_view/80x30.txt");
}

// -- M14.20: planet_view di 3 breakpoint (main_menu sudah py ini, planet_view baru 1/3) --

#[test]
fn planet_view_full() {
    check("planet_view", 120, 40, "planet_view/120x40.txt");
}

#[test]
fn planet_view_minimal() {
    check("planet_view", 60, 24, "planet_view/60x24.txt");
}

#[test]
fn research() {
    check("research", 120, 40, "research/120x40.txt");
}

// -- M15.14: research di 3 breakpoint (main_menu/planet_view sudah py ini, research baru 1/3) --

#[test]
fn research_compact() {
    check("research", 80, 30, "research/80x30.txt");
}

#[test]
fn research_minimal() {
    check("research", 60, 24, "research/60x24.txt");
}

#[test]
fn warp() {
    check("warp", 120, 40, "warp_120x40.txt");
}

#[test]
fn galaxy_map() {
    check("galaxy_map", 120, 40, "galaxy_map/120x40.txt");
}

// -- M20.6: galaxy_map di 3 breakpoint (main_menu/planet_view/research sudah py ini) --

#[test]
fn galaxy_map_compact() {
    check("galaxy_map", 80, 30, "galaxy_map/80x30.txt");
}

#[test]
fn galaxy_map_minimal() {
    check("galaxy_map", 60, 24, "galaxy_map/60x24.txt");
}

// -- Responsiveness: main_menu di tiap breakpoint (06 §Responsive) --

#[test]
fn main_menu_compact() {
    check("main_menu", 80, 30, "main_menu/80x30.txt");
}

#[test]
fn main_menu_minimal() {
    check("main_menu", 60, 24, "main_menu/60x24.txt");
}
