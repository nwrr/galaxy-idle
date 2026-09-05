//! Render satu view TUI ke PNG berwarna (headless, Buffer→PNG). `agent/test/README.md`.
//! Usage: cargo run --bin screenshot -- <view> <w> <h> [theme] <out.png>
//! `[theme]`: `default` | `high_contrast` | `mono` (opsional, default `default`).

use galaxy_idle::game::state::ThemeChoice;
use galaxy_idle::ui::{self, KNOWN_VIEWS, raster, theme};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use std::process::ExitCode;

const THEME_NAMES: &[&str] = &["default", "high_contrast", "mono"];

fn parse_theme(s: &str) -> Option<ThemeChoice> {
    match s {
        "default" => Some(ThemeChoice::Default),
        "high_contrast" => Some(ThemeChoice::HighContrast),
        "mono" => Some(ThemeChoice::Mono),
        _ => None,
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        eprintln!(
            "usage: screenshot <view> <w> <h> [theme] <out.png>\nview: {}",
            KNOWN_VIEWS.join(", ")
        );
        return ExitCode::FAILURE;
    }

    let view = args[1].as_str();
    if !KNOWN_VIEWS.contains(&view) {
        eprintln!(
            "view tak dikenal: '{view}'. Pilihan: {}",
            KNOWN_VIEWS.join(", ")
        );
        return ExitCode::FAILURE;
    }

    let w: u16 = match args[2].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("w harus angka positif, dapat: '{}'", args[2]);
            return ExitCode::FAILURE;
        }
    };
    let h: u16 = match args[3].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("h harus angka positif, dapat: '{}'", args[3]);
            return ExitCode::FAILURE;
        }
    };
    let out_path = args.last().unwrap();

    // `[theme]` opsional: hadir hanya bila total argumen 6 (view,w,h,theme,out.png + argv[0]).
    let theme_choice = if args.len() >= 6 {
        match parse_theme(&args[4]) {
            Some(t) => t,
            None => {
                eprintln!(
                    "theme tak dikenal: '{}'. Pilihan: {}",
                    args[4],
                    THEME_NAMES.join(", ")
                );
                return ExitCode::FAILURE;
            }
        }
    } else {
        ThemeChoice::default()
    };

    let backend = TestBackend::new(w, h);
    let mut term = Terminal::new(backend).unwrap();
    let mut app = galaxy_idle::demo_app();
    app.state.settings.theme = theme_choice;
    term.draw(|f| ui::draw_view(f, &app, view)).unwrap();

    let t = theme::theme(app.state.settings.theme);
    let img = raster::buffer_to_rgba(term.backend().buffer(), &t);
    if let Err(e) = img.save(out_path) {
        eprintln!("gagal tulis PNG '{out_path}': {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
