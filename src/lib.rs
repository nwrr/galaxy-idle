//! galaxy-idle — lib crate. Modul dipakai bersama oleh bin `galaxy-idle` (`main.rs`)
//! dan `snapshot` (`bin/snapshot.rs`). Layout modul = `07-architecture.md`.

pub mod app;
pub mod balance;
pub mod content;
pub mod game;
pub mod rng;
pub mod save;
pub mod sim;
pub mod ui;

/// State+content contoh deterministik untuk snapshot/headless render (`agent/test/README.md`).
pub fn demo_app() -> app::App {
    app::App::demo()
}
