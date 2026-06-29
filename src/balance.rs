//! Konstanta balancing global dari `abstraction/design/08-balancing.md`.
//!
//! Hanya konstanta **lintas-entri** (rumus, growth, rate). Angka per-entri (harga
//! resource, input/output recipe, biaya building) ada di katalog `data/*.ron`,
//! bukan di sini. Nilai = baseline awal; di-tune saat playtest.
//!
//! NOTE(scaffold): `allow(dead_code)` sementara — const dikonsumsi mulai M2 (sim),
//! M5–M9. Hapus allow saat seluruhnya terpakai dari jalur `main`.
#![allow(dead_code)]

// ── Tick & Loop ────────────────────────────────────────────────────────────
/// Satu langkah simulasi = 1 detik game-time.
pub const TICK_DURATION_SECS: f64 = 1.0;
/// Interval render (~13 FPS untuk animasi).
pub const RENDER_INTERVAL_MS: u64 = 75;
/// Autosave berkala (+ saat quit & sebelum warp).
pub const AUTOSAVE_INTERVAL_SECS: f64 = 30.0;
/// Batas catch-up tick per frame; sisanya jadi offline progress.
pub const MAX_TICKS_PER_FRAME: u32 = 16;

// ── Offline Progress ───────────────────────────────────────────────────────
/// Maksimum waktu offline yang dihitung (8 jam).
pub const OFFLINE_CAP_SECS: u64 = 28_800;
/// Efisiensi dasar offline (<1.0).
pub const OFFLINE_BASE_EFF: f64 = 0.75;
/// Bonus efisiensi offline per level upgrade `offline_eff` (cap total 1.0).
pub const OFFLINE_EFF_PER_LVL: f64 = 0.05;

// ── Economy — Scaling ──────────────────────────────────────────────────────
/// `cost(n) = base * GROWTH^n` untuk semua upgrade.
pub const GROWTH: f64 = 1.15;
/// Laju extractor dasar /s (× level × richness × tech_mult).
pub const BASE_EXTRACTOR_RATE: f64 = 1.0;
/// Laju Data dasar /s per level ResearchLab.
pub const BASE_DATA_RATE: f64 = 5.0;

// Base cost upgrade (Credits / resource terkait) — lihat `08` §Base cost.
pub const COST_FACTORY_LEVEL: f64 = 50.0;
pub const COST_NODE_LEVEL: f64 = 100.0;
pub const COST_SHIP_ENGINE: f64 = 200.0;
pub const COST_SHIP_CARGO: f64 = 200.0;
pub const COST_SHIP_SCANNER: f64 = 250.0;

// ── Ship / Travel ──────────────────────────────────────────────────────────
/// Basis travel time (×distance).
pub const BASE_TRAVEL_SECS: f64 = 600.0;
/// `travel *= ENGINE_FACTOR^engine_level`.
pub const ENGINE_FACTOR: f64 = 0.90;
/// Stockpile cap dasar planet.
pub const BASE_CAP: f64 = 1_000.0;
/// `cap = BASE_CAP * (1 + CARGO_PER_LVL * cargo_level)`.
pub const CARGO_PER_LVL: f64 = 0.25;
/// `yield_mult = 1 + DIST_YIELD_K * distance`.
pub const DIST_YIELD_K: f64 = 0.15;

// Warp Tier → akses (lihat `08` §Warp Tier).
pub const WARP_TIER_OUTER1: u32 = 6;
pub const WARP_TIER_OUTER2: u32 = 10;

// ── Prestige / Warp ────────────────────────────────────────────────────────
/// `cores = floor(WARP_K * sqrt(total_value / WARP_THRESHOLD_REF))`.
pub const WARP_K: f64 = 10.0;
/// Referensi normalisasi total_value.
pub const WARP_THRESHOLD_REF: f64 = 1_000_000.0;
/// `prod_mult = 1 + PRESTIGE_MULT_PER_LEVEL * level_reached`.
pub const PRESTIGE_MULT_PER_LEVEL: f64 = 0.10;
/// Feed dasar Milky Way → galaksi aktif (/s).
pub const ANCHOR_BASE: f64 = 5.0;
/// `feed = ANCHOR_BASE * (1 + ANCHOR_PER_LVL * anchor_lvl)`.
pub const ANCHOR_PER_LVL: f64 = 0.50;

// ── Events ─────────────────────────────────────────────────────────────────
/// Peluang event per roll.
pub const BASE_EVENT_CHANCE: f64 = 0.05;
/// × travel_distance.
pub const EVENT_DIST_FACTOR: f64 = 0.10;
/// × scanner level.
pub const EVENT_SCANNER_FACTOR: f64 = 0.20;
/// Roll event tiap N ticks travel.
pub const EVENT_ROLL_INTERVAL_TICKS: u32 = 30;
/// Penalti travel saat nebula.
pub const NEBULA_TRAVEL_PENALTY: f64 = 0.20;
/// Buff outpost: +50% prod selama durasi di bawah.
pub const OUTPOST_BUFF_MULT: f64 = 0.50;
pub const OUTPOST_BUFF_SECS: f64 = 3_600.0;
/// Salvage wreck mem-pause travel selama ini.
pub const WRECK_SALVAGE_SECS: f64 = 600.0;

// ── Merchant ───────────────────────────────────────────────────────────────
/// Window aktif merchant setelah muncul.
pub const MERCHANT_WINDOW_SECS: f64 = 300.0;
/// Interval restock (rotasi stock, 2 jam).
pub const MERCHANT_RESTOCK_SECS: f64 = 7_200.0;

// ── ProcGen (lihat `16-procgen-logic.md`) ──────────────────────────────────
pub const PLANET_MIN: u32 = 5;
pub const PLANET_MAX: u32 = 12;
pub const NODE_MIN: u32 = 2;
pub const NODE_MAX: u32 = 5;
/// Ambang jarak → tier planet.
pub const DIST_D1: f64 = 3.5;
pub const DIST_D2: f64 = 7.0;
/// Peluang planet punya anomali.
pub const ANOMALY_CHANCE: f64 = 0.15;
pub const RICH_MIN: f64 = 0.5;
pub const RICH_MAX: f64 = 2.0;
pub const DIST_MIN: f64 = 1.0;
pub const DIST_MAX: f64 = 10.0;

// ── Galaxy Animation (lihat `14-galaxy-animation.md`) ──────────────────────
pub const STAR_COUNT: u32 = 600;
pub const ARM_COUNT: u32 = 2;
pub const SPIRAL_B: f64 = 0.25;
pub const R_MAX: f64 = 1.0;
pub const R_CORE: f64 = 0.2;
pub const OMEGA0: f64 = 0.05;
pub const OMEGA_R0: f64 = 0.3;
pub const SCATTER: f64 = 0.3;
/// Koreksi rasio sel terminal (juga dipakai particle).
pub const ASPECT: f64 = 2.0;
pub const TWINKLE_FREQ: f64 = 1.0;
pub const TWINKLE_AMP: f64 = 0.3;

// ── Particle Effects (lihat `15-particle-effects.md`) ──────────────────────
/// Hard cap total partikel.
pub const PARTICLE_BUDGET: u32 = 500;
/// Emitter aktif bersamaan.
pub const EMITTER_MAX: u32 = 16;

// Sanity invariant baseline — dicek saat kompilasi (gagal build bila dilanggar).
const _: () = assert!(GROWTH > 1.0);
const _: () = assert!(OFFLINE_BASE_EFF > 0.0 && OFFLINE_BASE_EFF < 1.0);
const _: () = assert!(PLANET_MIN < PLANET_MAX);
const _: () = assert!(NODE_MIN < NODE_MAX);
const _: () = assert!(RICH_MIN < RICH_MAX);
const _: () = assert!(DIST_MIN < DIST_MAX);
const _: () = assert!(DIST_D1 < DIST_D2);
const _: () = assert!(WARP_TIER_OUTER1 < WARP_TIER_OUTER2);
const _: () = assert!(ENGINE_FACTOR < 1.0);
