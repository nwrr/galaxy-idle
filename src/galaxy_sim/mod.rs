//! Galaxy simulation engine: density-wave orbital model + GPU/CPU render, diadaptasi dari
//! proyek referensi `andromeda-simulation-tui` (M20.8 follow-up plan, lihat
//! `/home/nwrr/.claude/plans/playful-wondering-castle.md`). Modul ini murni simulasi/matematika
//! (bukan game state) — tampilan/wiring UI ada di `crate::ui::galaxy_sim_view` (Phase 5+).
//!
//! Phase 0: `spec.rs`. Phase 1: `model.rs`+`orbit.rs`. Phase 2: `generate.rs`+`palette.rs`.
//! Phase 3: `render/` (CPU/rayon). Phase 4: `render::gpu`. Phase 5: UI wiring (`ui::
//! galaxy_sim_view`). Phase 6: `camera.rs` (pan/zoom). Phase 7 (SEKARANG): `poi.rs` (katalog
//! POI + fokus/follow, wiring di `ui::galaxy_sim_view`).

pub mod camera;
pub mod generate;
pub mod model;
pub mod orbit;
pub mod palette;
pub mod poi;
pub mod render;
pub mod spec;
