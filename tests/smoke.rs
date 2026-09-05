//! Smoke G1: app stabil (tidak panic) menjalankan sim + render lintas semua view, termasuk
//! ship Traveling (memicu roll event). Loop interaktif `cargo run` butuh TTY → tak bisa di-drive
//! headless; test ini mengover jalur sim/render/quit yang sama dipakai `app::event_loop`.

use galaxy_idle::app::App;
use galaxy_idle::game::state::{PlanetId, ShipStatus};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

const VIEWS: [&str; 5] = ["main_menu", "planet_view", "research", "galaxy_map", "warp"];

#[test]
fn app_runs_sim_and_renders_all_views_without_panic() {
    let mut app = App::demo();
    // Set ship Traveling → fase tick 6 me-roll event (jalur baru M9) tiap interval.
    app.state.ship.status = ShipStatus::Traveling {
        to: PlanetId(0),
        total_secs: 10_000.0,
        elapsed_secs: 0.0,
    };
    let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();

    // ~120 tick game-time + render tiap view bergantian; tak boleh panic.
    for i in 0..120u64 {
        galaxy_idle::sim::tick::step(&mut app.state, &app.content);
        app.state.tick += 1;
        let view = VIEWS[(i as usize) % VIEWS.len()];
        term.draw(|f| galaxy_idle::ui::draw_view(f, &app, view))
            .unwrap();
    }
}

#[test]
fn app_renders_every_breakpoint_without_panic() {
    let app = App::demo();
    for (w, h) in [(120u16, 40u16), (80, 30), (60, 24), (40, 12)] {
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        for view in VIEWS {
            term.draw(|f| galaxy_idle::ui::draw_view(f, &app, view))
                .unwrap();
        }
    }
}

/// M12.9: guard lebar minimum — ukuran EKSTREM (di bawah breakpoint Minimal terkecil, sampai
/// 1×1) tak boleh panic. `shell_full` (M12) pakai kolom `Length` tetap (22/18) yg SECARA TEORI
/// bisa jadi masalah bila area lebih sempit dari totalnya, tp `layout_mode` menggerbang
/// `shell_full` hanya di lebar≥100 jadi tak pernah kejadian nyata — test ini tetap MEMBUKTIKAN
/// ratatui `Layout` gagal-aman (clip/kompres, bukan panic) di ekstrem, bukan cuma diasumsikan.
#[test]
fn app_renders_extreme_tiny_sizes_without_panic() {
    let app = App::demo();
    let sizes: [(u16, u16); 8] = [
        (1, 1),
        (2, 2),
        (5, 5),
        (10, 5),
        (20, 8),
        (39, 11),
        (40, 12),
        (99, 29),
    ];
    for (w, h) in sizes {
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        for view in VIEWS {
            term.draw(|f| galaxy_idle::ui::draw_view(f, &app, view))
                .unwrap();
        }
    }
}
