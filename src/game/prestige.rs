//! Prestige (Warp Jump): reset galaksi aktif, gain Warp Core, naikkan level, pertahankan anchor.
//! `04-prestige.md` §1–2, konstanta `08-balancing.md` §Prestige.
//!
//! Yang di-reset: resource/factory/node galaksi aktif (kecuali anchor), `credits`,
//! `research.data` (+active). Yang dipertahankan: `PrestigeState`, `AnchorState`, `Ship`, tech
//! `completed`. Warp Core: `floor(WARP_K * sqrt(total_value / WARP_THRESHOLD_REF))`.
//!
//! NOTE(scaffold): `allow(dead_code)` — pemindahan ke galaksi ProcGen Lvl+1 (set active baru)
//! menyusul di M9; iter ini fokus efek persisten + reset + Warp Core.
#![allow(dead_code)]

use crate::balance::{
    ANCHOR_BASE, ANCHOR_PER_LVL, PRESTIGE_MULT_PER_LEVEL, TICK_DURATION_SECS, WARP_K,
    WARP_THRESHOLD_REF,
};
use crate::game::defs::{Content, ResourceId};
use crate::game::economy::{add_capped, price};
use crate::game::state::GameState;

#[derive(Debug, PartialEq, Eq)]
pub enum PrestigeError {
    NoActiveGalaxy,
    /// Resource belum memenuhi Warp Threshold untuk level target.
    ThresholdNotMet,
    /// Warp Core tak cukup untuk membeli upgrade.
    InsufficientCores,
    /// Id permanent upgrade tak dikenal.
    UnknownUpgrade,
}

#[derive(Clone, Debug, Default)]
pub struct WarpReport {
    pub cores_gained: u64,
    pub new_level: u8,
    pub production_mult: f64,
}

/// Production multiplier dari level prestige tertinggi yang dicapai. `04` §1.
pub fn production_mult(galaxy_level_reached: u8) -> f64 {
    1.0 + PRESTIGE_MULT_PER_LEVEL * galaxy_level_reached as f64
}

/// Warp Core yang didapat dari `total_value` (sqrt → diminishing returns). `04` §1.
pub fn warp_cores_gain(total_value: f64) -> u64 {
    (WARP_K * (total_value / WARP_THRESHOLD_REF).max(0.0).sqrt()).floor() as u64
}

/// Nilai agregat saat warp = `credits` + Σ(stockpile resource × base_price) di galaksi aktif.
pub fn total_value(state: &GameState, content: &Content) -> f64 {
    let Some(g) = state.galaxies.iter().find(|g| g.id == state.active_galaxy) else {
        return state.credits;
    };
    let mut v = state.credits;
    for p in &g.planets {
        for (res, amt) in &p.stockpile {
            v += amt * price(content, *res);
        }
    }
    v
}

/// Warp Threshold per level target (`08` §Warp Threshold): L1 Energy+Titanium, L2 +Antimatter,
/// ≥3 ×5 Energy & ×4 Titanium per level. Resource yang tak ada di katalog dilewati.
pub fn warp_threshold(content: &Content, target_level: u8) -> Vec<(ResourceId, f64)> {
    let n = target_level.max(1) as i32;
    let energy = 1_000_000.0 * 5f64.powi(n - 1);
    let titanium = 500.0 * 4f64.powi(n - 1);
    let mut out = Vec::new();
    let mut push = |name: &str, amt: f64| {
        if let Some(h) = content.resources.id(name) {
            out.push((ResourceId(h), amt));
        }
    };
    push("energy", energy);
    push("titanium", titanium);
    if target_level >= 2 {
        // Antimatter mulai L2; skala ×5/level (default; `08` tandai rare-resource lanjut TBD).
        push("antimatter", 100.0 * 5f64.powi(n - 2));
    }
    out
}

/// Total resource `res` di seluruh planet galaksi aktif.
fn have(state: &GameState, res: ResourceId) -> f64 {
    state
        .galaxies
        .iter()
        .find(|g| g.id == state.active_galaxy)
        .map(|g| {
            g.planets
                .iter()
                .map(|p| p.stockpile.get(&res).copied().unwrap_or(0.0))
                .sum()
        })
        .unwrap_or(0.0)
}

/// Apakah resource galaksi aktif memenuhi threshold untuk `target_level`.
pub fn meets_threshold(state: &GameState, content: &Content, target_level: u8) -> bool {
    warp_threshold(content, target_level)
        .iter()
        .all(|(r, amt)| have(state, *r) >= *amt)
}

/// Eksekusi Warp Jump: gain Warp Core, naik level, reset galaksi aktif (kecuali anchor) + credits +
/// Data; pertahankan prestige/anchor/ship/tech `completed`. Gagal bila threshold belum terpenuhi.
pub fn warp_jump(state: &mut GameState, content: &Content) -> Result<WarpReport, PrestigeError> {
    if !state.galaxies.iter().any(|g| g.id == state.active_galaxy) {
        return Err(PrestigeError::NoActiveGalaxy);
    }
    let target = state.prestige.galaxy_level_reached + 1;
    if !meets_threshold(state, content, target) {
        return Err(PrestigeError::ThresholdNotMet);
    }
    let cores = warp_cores_gain(total_value(state, content));
    state.prestige.warp_cores += cores;
    state.prestige.galaxy_level_reached = target;

    // Reset "cash": credits + Data terkumpul (+ riset aktif). Tech `completed` dipertahankan.
    state.credits = 0.0;
    state.research.data = 0.0;
    state.research.active = None;

    // Reset ekonomi galaksi aktif — kecuali bila aktif = anchor (Milky Way tak pernah reset).
    if state.active_galaxy != state.anchor_galaxy
        && let Some(gi) = state
            .galaxies
            .iter()
            .position(|g| g.id == state.active_galaxy)
    {
        for p in &mut state.galaxies[gi].planets {
            p.stockpile.clear();
            for slot in &mut p.factory_slots {
                *slot = None;
            }
            for node in &mut p.nodes {
                node.level = 0;
            }
        }
    }

    Ok(WarpReport {
        cores_gained: cores,
        new_level: target,
        production_mult: production_mult(target),
    })
}

// ── Anchor feed (Milky Way → galaksi aktif) ─────────────────────────────────
/// Feed pasif Milky Way ke galaksi aktif /detik = `ANCHOR_BASE * (1 + ANCHOR_PER_LVL * level)`.
/// `04` §2.
pub fn anchor_feed(passive_upgrade_level: u32) -> f64 {
    ANCHOR_BASE * (1.0 + ANCHOR_PER_LVL * passive_upgrade_level as f64)
}

/// Terapkan anchor feed satu tick: tambah resource dasar ke planet pertama galaksi aktif. Hanya
/// berlaku saat bermain di galaksi non-anchor (Lvl≥1); di Milky Way sendiri tak ada feed.
/// Asumsi: resource yang di-feed = `iron` (representatif basic; `04` §2 "resource dasar").
pub fn apply_anchor_feed(state: &mut GameState, content: &Content) {
    if state.active_galaxy == state.anchor_galaxy {
        return;
    }
    let Some(iron) = content.resources.id("iron").map(ResourceId) else {
        return;
    };
    let amount = anchor_feed(state.prestige.anchor.passive_upgrade_level) * TICK_DURATION_SECS;
    if let Some(g) = state
        .galaxies
        .iter_mut()
        .find(|g| g.id == state.active_galaxy)
        && let Some(p) = g.planets.iter_mut().find(|p| p.unlocked)
    {
        let cap = p.stockpile_cap;
        add_capped(&mut p.stockpile, iron, amount, cap);
    }
}

// ── Permanent upgrades & anchor upgrade (sink Warp Core) ─────────────────────
/// `(base_cost, growth)` biaya Warp Core per upgrade id (`08`). `None` = id tak dikenal.
pub fn upgrade_base_cost(id: &str) -> Option<(f64, f64)> {
    match id {
        "prod_speed" => Some((5.0, 1.5)),
        "extra_slot" => Some((10.0, 2.0)),
        "auto_collect" => Some((8.0, 1.8)),
        "offline_eff" => Some((6.0, 1.6)),
        _ => None,
    }
}

/// Biaya Warp Core upgrade `id` pada `level` sekarang = `ceil(base * growth^level)`.
pub fn permanent_upgrade_cost(id: &str, level: u32) -> Option<u64> {
    upgrade_base_cost(id).map(|(base, g)| (base * g.powi(level as i32)).ceil() as u64)
}

/// Beli 1 level permanent upgrade `id` dengan Warp Core. Kembalikan biaya yang dibayar.
pub fn buy_permanent_upgrade(state: &mut GameState, id: &str) -> Result<u64, PrestigeError> {
    let level = state
        .prestige
        .permanent_upgrades
        .get(id)
        .copied()
        .unwrap_or(0);
    let cost = permanent_upgrade_cost(id, level).ok_or(PrestigeError::UnknownUpgrade)?;
    if state.prestige.warp_cores < cost {
        return Err(PrestigeError::InsufficientCores);
    }
    state.prestige.warp_cores -= cost;
    *state
        .prestige
        .permanent_upgrades
        .entry(id.to_string())
        .or_insert(0) += 1;
    Ok(cost)
}

/// Base cost upgrade Anchor (`AnchorState.passive_upgrade_level`). `08` belum tetapkan kurva khusus
/// → default geometrik base 10 growth 2.0 (TBD, sink Warp Core jangka panjang `04` §2).
const ANCHOR_UPGRADE_BASE: f64 = 10.0;
const ANCHOR_UPGRADE_GROWTH: f64 = 2.0;

/// Beli 1 level upgrade Anchor (boost feed pasif) dengan Warp Core. Kembalikan biaya.
pub fn buy_anchor_upgrade(state: &mut GameState) -> Result<u64, PrestigeError> {
    let level = state.prestige.anchor.passive_upgrade_level;
    let cost = (ANCHOR_UPGRADE_BASE * ANCHOR_UPGRADE_GROWTH.powi(level as i32)).ceil() as u64;
    if state.prestige.warp_cores < cost {
        return Err(PrestigeError::InsufficientCores);
    }
    state.prestige.warp_cores -= cost;
    state.prestige.anchor.passive_upgrade_level += 1;
    Ok(cost)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_content;
    use crate::game::state::{
        Biome, Factory, FactoryId, FactoryKind, Galaxy, GalaxyId, GalaxyKind, NodeId, Planet,
        PlanetId, ResourceMap, ResourceNode, Ship, UnlockReq,
    };

    fn content() -> Content {
        load_content(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data")).unwrap()
    }

    fn rid(c: &Content, n: &str) -> ResourceId {
        ResourceId(c.resources.id(n).unwrap())
    }

    /// State: anchor MW (GalaxyId 0) + galaksi aktif (GalaxyId 1) dengan stockpile threshold L1.
    fn two_galaxy_state(c: &Content) -> GameState {
        let energy = rid(c, "energy");
        let titanium = rid(c, "titanium");
        let mut stock = ResourceMap::new();
        stock.insert(energy, 1_000_000.0);
        stock.insert(titanium, 500.0);
        let active_planet = Planet {
            id: PlanetId(0),
            name: "New".into(),
            tier: 1,
            biome: Biome::Terran,
            distance: 1.0,
            unlocked: true,
            unlock_req: UnlockReq::None,
            nodes: vec![ResourceNode {
                id: NodeId(0),
                resource: energy,
                richness: 1.0,
                level: 3,
            }],
            factory_slots: vec![Some(Factory {
                id: FactoryId(0),
                building: crate::game::defs::BuildingId(c.buildings.id("mining_drill").unwrap()),
                kind: FactoryKind::Extractor { node: NodeId(0) },
                level: 2,
                enabled: true,
            })],
            stockpile: stock,
            stockpile_cap: 1e9,
        };
        let anchor = Galaxy {
            id: GalaxyId(0),
            name: "Milky Way".into(),
            level: 0,
            kind: GalaxyKind::Fixed,
            planets: vec![],
        };
        let active = Galaxy {
            id: GalaxyId(1),
            name: "Andromeda".into(),
            level: 1,
            kind: GalaxyKind::Fixed,
            planets: vec![active_planet],
        };
        GameState {
            version: 1,
            last_saved_unix: 0,
            tick: 0,
            credits: 5000.0,
            inventory: std::collections::HashMap::new(),
            galaxies: vec![anchor, active],
            active_galaxy: GalaxyId(1),
            anchor_galaxy: GalaxyId(0),
            ship: Ship::default(),
            research: Default::default(),
            prestige: Default::default(),
            merchant: Default::default(),
            events: Default::default(),
            settings: Default::default(),
        }
    }

    #[test]
    fn cores_and_mult_formulas() {
        assert!((production_mult(0) - 1.0).abs() < 1e-9);
        assert!((production_mult(3) - 1.3).abs() < 1e-9);
        // total_value = REF → WARP_K*sqrt(1) = 10.
        assert_eq!(warp_cores_gain(WARP_THRESHOLD_REF), WARP_K as u64);
        // ×4 REF → ×2.
        assert_eq!(warp_cores_gain(4.0 * WARP_THRESHOLD_REF), 2 * WARP_K as u64);
    }

    #[test]
    fn warp_jump_resets_active_keeps_persistent() {
        let c = content();
        let mut st = two_galaxy_state(&c);
        st.research.completed.insert("manu_steel".into());
        let report = warp_jump(&mut st, &c).expect("threshold L1 terpenuhi");
        assert!(report.cores_gained > 0);
        assert_eq!(report.new_level, 1);
        assert_eq!(st.prestige.galaxy_level_reached, 1);
        // Cash di-reset.
        assert_eq!(st.credits, 0.0);
        assert_eq!(st.research.data, 0.0);
        // Tech completed dipertahankan.
        assert!(st.research.completed.contains("manu_steel"));
        // Galaksi aktif (non-anchor) di-reset.
        let active = st.galaxies.iter().find(|g| g.id == GalaxyId(1)).unwrap();
        assert!(active.planets[0].stockpile.is_empty());
        assert!(active.planets[0].factory_slots[0].is_none());
    }

    #[test]
    fn warp_jump_threshold_gate() {
        let c = content();
        let mut st = two_galaxy_state(&c);
        // Kurangi energy di bawah threshold → gagal.
        let energy = rid(&c, "energy");
        st.galaxies[1].planets[0].stockpile.insert(energy, 1.0);
        assert!(matches!(
            warp_jump(&mut st, &c),
            Err(PrestigeError::ThresholdNotMet)
        ));
    }

    #[test]
    fn anchor_feed_formula_and_apply() {
        let c = content();
        assert!((anchor_feed(0) - ANCHOR_BASE).abs() < 1e-9);
        assert!((anchor_feed(2) - ANCHOR_BASE * (1.0 + ANCHOR_PER_LVL * 2.0)).abs() < 1e-9);
        let iron = rid(&c, "iron");
        // Active = galaksi non-anchor (GalaxyId 1) → feed masuk.
        let mut st = two_galaxy_state(&c);
        apply_anchor_feed(&mut st, &c);
        let g = st.galaxies.iter().find(|g| g.id == GalaxyId(1)).unwrap();
        let got = g.planets[0].stockpile.get(&iron).copied().unwrap_or(0.0);
        assert!((got - anchor_feed(0) * TICK_DURATION_SECS).abs() < 1e-9);
        // Active = anchor → tak ada feed.
        let mut st2 = two_galaxy_state(&c);
        st2.active_galaxy = GalaxyId(0);
        apply_anchor_feed(&mut st2, &c);
        // Galaksi anchor tak punya planet → tak ada perubahan (dan tak panik).
        assert!(st2.galaxies[0].planets.is_empty());
    }

    #[test]
    fn permanent_and_anchor_upgrades_cost_cores() {
        let c = content();
        let mut st = two_galaxy_state(&c);
        st.prestige.warp_cores = 100;
        // prod_speed base 5 → lvl0 cost 5.
        let paid = buy_permanent_upgrade(&mut st, "prod_speed").unwrap();
        assert_eq!(paid, 5);
        assert_eq!(st.prestige.warp_cores, 95);
        assert_eq!(
            st.prestige.permanent_upgrades.get("prod_speed").copied(),
            Some(1)
        );
        // lvl1 cost ceil(5*1.5)=8.
        assert_eq!(buy_permanent_upgrade(&mut st, "prod_speed").unwrap(), 8);
        // unknown id.
        assert_eq!(
            buy_permanent_upgrade(&mut st, "nope"),
            Err(PrestigeError::UnknownUpgrade)
        );
        // anchor upgrade base 10.
        assert_eq!(buy_anchor_upgrade(&mut st).unwrap(), 10);
        assert_eq!(st.prestige.anchor.passive_upgrade_level, 1);
        // dana habis → Insufficient.
        st.prestige.warp_cores = 0;
        assert_eq!(
            buy_anchor_upgrade(&mut st),
            Err(PrestigeError::InsufficientCores)
        );
    }

    #[test]
    fn anchor_not_reset_when_active() {
        let c = content();
        let mut st = two_galaxy_state(&c);
        // Jadikan anchor sebagai galaksi aktif + beri stockpile threshold.
        st.active_galaxy = GalaxyId(0);
        let energy = rid(&c, "energy");
        let titanium = rid(&c, "titanium");
        let p = Planet {
            id: PlanetId(9),
            name: "Earth".into(),
            tier: 1,
            biome: Biome::Terran,
            distance: 1.0,
            unlocked: true,
            unlock_req: UnlockReq::None,
            nodes: vec![],
            factory_slots: vec![],
            stockpile: {
                let mut m = ResourceMap::new();
                m.insert(energy, 1_000_000.0);
                m.insert(titanium, 500.0);
                m
            },
            stockpile_cap: 1e9,
        };
        st.galaxies[0].planets.push(p);
        warp_jump(&mut st, &c).unwrap();
        // Anchor (aktif) TIDAK di-reset: stockpile tetap ada.
        let anchor = st.galaxies.iter().find(|g| g.id == GalaxyId(0)).unwrap();
        assert!(!anchor.planets[0].stockpile.is_empty());
    }
}
