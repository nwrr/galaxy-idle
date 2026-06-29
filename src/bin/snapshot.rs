//! Render satu view TUI ke teks (headless, deterministik) → stdout. `agent/test/README.md`.
//! Usage: cargo run --bin snapshot -- <view> <w> <h>

use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let view = args.get(1).map(String::as_str).unwrap_or("main_menu");
    let w: u16 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(120);
    let h: u16 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(40);

    let backend = TestBackend::new(w, h);
    let mut term = Terminal::new(backend).unwrap();
    let app = galaxy_idle::demo_app();
    term.draw(|f| galaxy_idle::ui::draw_view(f, &app, view))
        .unwrap();

    for line in galaxy_idle::ui::buffer_to_text(term.backend().buffer()) {
        println!("{line}");
    }
}
