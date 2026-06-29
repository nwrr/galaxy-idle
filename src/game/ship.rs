//! Ship: gatekeeper Warp Tier + buff pasif part (engine/cargo/scanner). `03-progression.md` §3–4.
//!
//! Ship **bukan unit combat**. Dua peran: (a) gatekeeper akses planet/galaksi via `UnlockReq`,
//! (b) buff pasif — engine ↓ travel time, cargo ↑ stockpile cap, scanner ↑ peluang event.
//! Biaya upgrade geometrik (`02` §5) dibayar Credits; konstanta di `balance.rs`.
//!
//! NOTE(scaffold): `allow(dead_code)` — dipakai sim travel (M6 item 2) & UI fleet (M9).
#![allow(dead_code)]

use crate::balance::{
    BASE_CAP, BASE_EVENT_CHANCE, BASE_TRAVEL_SECS, CARGO_PER_LVL, COST_SHIP_CARGO,
    COST_SHIP_ENGINE, COST_SHIP_SCANNER, DIST_YIELD_K, ENGINE_FACTOR, EVENT_DIST_FACTOR,
    EVENT_SCANNER_FACTOR,
};
use crate::game::defs::{Content, ResourceId, ShipPart};
use crate::game::economy::upgrade_cost;
use crate::game::state::{GameState, Ship, ShipStatus, UnlockReq};

#[derive(Debug, PartialEq, Eq)]
pub enum ShipError {
    /// Part tanpa kurva biaya di `08` (shield/cloaking) — belum bisa di-upgrade.
    NotUpgradable,
    Insufficient,
}

// ── Gatekeeper (Warp Tier / UnlockReq) ──────────────────────────────────────
/// Apakah ship + ketersediaan resource memenuhi `req` (akses planet/galaksi). `03` §3a.
/// `have(res)` = jumlah resource yang dimiliki (untuk `UnlockReq::Resource`).
pub fn meets_unlock_req<F: Fn(ResourceId) -> f64>(req: &UnlockReq, ship: &Ship, have: &F) -> bool {
    match req {
        UnlockReq::None => true,
        UnlockReq::WarpTier(t) => ship.warp_tier >= *t,
        UnlockReq::Resource(r, amt) => have(*r) >= *amt,
        UnlockReq::All(list) => list.iter().all(|r| meets_unlock_req(r, ship, have)),
    }
}

// ── Buff pasif (Part) ───────────────────────────────────────────────────────
/// Travel time (detik) = `BASE_TRAVEL * distance * ENGINE_FACTOR^engine`. `03` §4.
pub fn travel_secs(distance: f64, engine: u32) -> f64 {
    BASE_TRAVEL_SECS * distance * ENGINE_FACTOR.powi(engine as i32)
}

/// Stockpile cap planet luar = `BASE_CAP * (1 + CARGO_PER_LVL * cargo)`. `03` §3b.
pub fn stockpile_cap(cargo: u32) -> f64 {
    BASE_CAP * (1.0 + CARGO_PER_LVL * cargo as f64)
}

/// Yield multiplier resource makin jauh = `1 + DIST_YIELD_K * distance`. `03` §4.
pub fn yield_mult(distance: f64) -> f64 {
    1.0 + DIST_YIELD_K * distance
}

/// Peluang event per roll saat travel (naik thd jarak & scanner). `05`/`08`.
pub fn event_chance(distance: f64, scanner: u32) -> f64 {
    BASE_EVENT_CHANCE
        * (1.0 + EVENT_DIST_FACTOR * distance)
        * (1.0 + EVENT_SCANNER_FACTOR * scanner as f64)
}

// ── Upgrade part ────────────────────────────────────────────────────────────
pub fn part_level(ship: &Ship, part: ShipPart) -> u32 {
    match part {
        ShipPart::Engine => ship.engine,
        ShipPart::Cargo => ship.cargo,
        ShipPart::Scanner => ship.scanner,
        ShipPart::Shield => ship.shield,
        ShipPart::Cloaking => ship.cloaking,
    }
}

/// Base cost upgrade part dari `08`; `None` = belum ada kurva (shield/cloaking).
fn part_base_cost(part: ShipPart) -> Option<f64> {
    match part {
        ShipPart::Engine => Some(COST_SHIP_ENGINE),
        ShipPart::Cargo => Some(COST_SHIP_CARGO),
        ShipPart::Scanner => Some(COST_SHIP_SCANNER),
        ShipPart::Shield | ShipPart::Cloaking => None,
    }
}

/// Upgrade part: bayar Credits geometrik `base * GROWTH^level`, lalu naikkan level part.
pub fn upgrade_part(state: &mut GameState, part: ShipPart) -> Result<(), ShipError> {
    let base = part_base_cost(part).ok_or(ShipError::NotUpgradable)?;
    let cost = upgrade_cost(base, part_level(&state.ship, part));
    if state.credits < cost {
        return Err(ShipError::Insufficient);
    }
    state.credits -= cost;
    match part {
        ShipPart::Engine => state.ship.engine += 1,
        ShipPart::Cargo => state.ship.cargo += 1,
        ShipPart::Scanner => state.ship.scanner += 1,
        ShipPart::Shield | ShipPart::Cloaking => {}
    }
    Ok(())
}

// ── Travel ──────────────────────────────────────────────────────────────────
#[derive(Debug, PartialEq, Eq)]
pub enum TravelError {
    NoActiveGalaxy,
    PlanetOutOfRange,
    /// Ship sedang `Traveling` — tak bisa mulai travel baru.
    Busy,
    AlreadyUnlocked,
    /// Syarat `UnlockReq` (Warp Tier / resource) belum terpenuhi.
    Locked,
}

/// Mulai travel ke planet `pi` (galaksi aktif). Gate via [`meets_unlock_req`]; durasi via
/// [`travel_secs`] (distance × engine). Kedatangan dibuka di `sim::tick::advance_travel`.
pub fn travel_to(state: &mut GameState, _content: &Content, pi: usize) -> Result<(), TravelError> {
    let gi = state
        .galaxies
        .iter()
        .position(|g| g.id == state.active_galaxy)
        .ok_or(TravelError::NoActiveGalaxy)?;
    if !matches!(state.ship.status, ShipStatus::Idle) {
        return Err(TravelError::Busy);
    }
    if pi >= state.galaxies[gi].planets.len() {
        return Err(TravelError::PlanetOutOfRange);
    }
    let (req, dist, to, unlocked) = {
        let p = &state.galaxies[gi].planets[pi];
        (p.unlock_req.clone(), p.distance, p.id, p.unlocked)
    };
    if unlocked {
        return Err(TravelError::AlreadyUnlocked);
    }
    // Resource dimiliki = total stockpile resource di planet unlocked galaksi aktif.
    let have = |res: ResourceId| -> f64 {
        state.galaxies[gi]
            .planets
            .iter()
            .filter(|p| p.unlocked)
            .map(|p| p.stockpile.get(&res).copied().unwrap_or(0.0))
            .sum()
    };
    if !meets_unlock_req(&req, &state.ship, &have) {
        return Err(TravelError::Locked);
    }
    state.ship.status = ShipStatus::Traveling {
        to,
        total_secs: travel_secs(dist, state.ship.engine),
        elapsed_secs: 0.0,
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balance::GROWTH;
    use crate::content::load_content;
    use crate::game::state::{GalaxyId, ShipStatus};

    fn ship_tier(t: u8) -> Ship {
        Ship {
            warp_tier: t,
            ..Ship::default()
        }
    }

    #[test]
    fn warp_tier_gates_access() {
        let s = ship_tier(2);
        let no_res = |_: ResourceId| 0.0;
        assert!(meets_unlock_req(&UnlockReq::None, &s, &no_res));
        assert!(meets_unlock_req(&UnlockReq::WarpTier(2), &s, &no_res));
        assert!(meets_unlock_req(&UnlockReq::WarpTier(1), &s, &no_res));
        assert!(!meets_unlock_req(&UnlockReq::WarpTier(3), &s, &no_res));
    }

    #[test]
    fn resource_and_all_req() {
        let s = ship_tier(5);
        let have = |r: ResourceId| if r == ResourceId(7) { 100.0 } else { 0.0 };
        assert!(meets_unlock_req(
            &UnlockReq::Resource(ResourceId(7), 50.0),
            &s,
            &have
        ));
        assert!(!meets_unlock_req(
            &UnlockReq::Resource(ResourceId(7), 200.0),
            &s,
            &have
        ));
        // All: WarpTier ok + Resource ok → true; satu gagal → false.
        let ok = UnlockReq::All(vec![
            UnlockReq::WarpTier(3),
            UnlockReq::Resource(ResourceId(7), 50.0),
        ]);
        assert!(meets_unlock_req(&ok, &s, &have));
        let bad = UnlockReq::All(vec![
            UnlockReq::WarpTier(3),
            UnlockReq::Resource(ResourceId(7), 200.0),
        ]);
        assert!(!meets_unlock_req(&bad, &s, &have));
    }

    #[test]
    fn engine_reduces_travel_cargo_raises_cap() {
        // engine 0 → full travel; tiap level ×ENGINE_FACTOR(<1) → lebih cepat.
        let t0 = travel_secs(2.0, 0);
        let t1 = travel_secs(2.0, 1);
        assert!(t1 < t0);
        assert!((t1 - t0 * ENGINE_FACTOR).abs() < 1e-9);
        // cargo 0 → BASE_CAP; naik linear.
        assert!((stockpile_cap(0) - BASE_CAP).abs() < 1e-9);
        assert!((stockpile_cap(4) - BASE_CAP * (1.0 + CARGO_PER_LVL * 4.0)).abs() < 1e-9);
        // scanner naik → peluang event naik.
        assert!(event_chance(1.0, 1) > event_chance(1.0, 0));
    }

    #[test]
    fn upgrade_part_pays_geometric() {
        let mut st = state_credits(1000.0);
        // engine lvl 0 → cost COST_SHIP_ENGINE.
        upgrade_part(&mut st, ShipPart::Engine).unwrap();
        assert_eq!(st.ship.engine, 1);
        assert!((st.credits - (1000.0 - COST_SHIP_ENGINE)).abs() < 1e-9);
        // lvl 1 → cost ×GROWTH.
        let before = st.credits;
        upgrade_part(&mut st, ShipPart::Engine).unwrap();
        assert_eq!(st.ship.engine, 2);
        assert!((before - st.credits - COST_SHIP_ENGINE * GROWTH).abs() < 1e-6);
        // shield tak punya kurva → NotUpgradable.
        assert_eq!(
            upgrade_part(&mut st, ShipPart::Shield),
            Err(ShipError::NotUpgradable)
        );
        // dana habis → Insufficient.
        st.credits = 0.0;
        assert_eq!(
            upgrade_part(&mut st, ShipPart::Cargo),
            Err(ShipError::Insufficient)
        );
    }

    fn state_credits(credits: f64) -> GameState {
        let mut st = sol_state();
        st.credits = credits;
        st.galaxies.clear();
        st
    }

    fn data_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
    }

    /// State dengan galaksi Milky Way ter-load (Earth unlocked, sisa terkunci).
    fn sol_state() -> GameState {
        let content = load_content(&data_dir()).unwrap();
        let g = crate::game::world::load_milky_way(&data_dir(), &content).unwrap();
        GameState {
            version: 1,
            last_saved_unix: 0,
            tick: 0,
            credits: 0.0,
            inventory: std::collections::HashMap::new(),
            galaxies: vec![g],
            active_galaxy: GalaxyId(0),
            anchor_galaxy: GalaxyId(0),
            ship: Ship::default(),
            research: Default::default(),
            prestige: Default::default(),
            merchant: Default::default(),
            events: Default::default(),
            settings: Default::default(),
        }
    }

    fn luna_index(st: &GameState) -> usize {
        st.galaxies[0]
            .planets
            .iter()
            .position(|p| p.name.starts_with("Luna"))
            .unwrap()
    }

    #[test]
    fn travel_starts_and_is_busy() {
        let c = load_content(&data_dir()).unwrap();
        let mut st = sol_state();
        let li = luna_index(&st);
        // Ship tier 1 → Luna WarpTier(1) ok → mulai travel.
        super::travel_to(&mut st, &c, li).unwrap();
        assert!(matches!(st.ship.status, ShipStatus::Traveling { .. }));
        // Sedang travel → Busy.
        assert_eq!(super::travel_to(&mut st, &c, li), Err(TravelError::Busy));
    }

    #[test]
    fn travel_gate_and_already_unlocked() {
        let c = load_content(&data_dir()).unwrap();
        let mut st = sol_state();
        let li = luna_index(&st);
        // Warp tier 0 → Luna terkunci.
        st.ship.warp_tier = 0;
        assert_eq!(super::travel_to(&mut st, &c, li), Err(TravelError::Locked));
        // Earth (unlock None) sudah unlocked → AlreadyUnlocked.
        let ei = st.galaxies[0]
            .planets
            .iter()
            .position(|p| p.name == "Earth")
            .unwrap();
        assert_eq!(
            super::travel_to(&mut st, &c, ei),
            Err(TravelError::AlreadyUnlocked)
        );
    }
}
