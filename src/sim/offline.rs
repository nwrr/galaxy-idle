//! Offline progress: saat load, simulasikan waktu yang terlewat secara batch. `07` §Offline.
//!
//! `delta = now - last_saved_unix`; `offline_secs = min(delta, OFFLINE_CAP)`;
//! `eff = OFFLINE_BASE_EFF * (1 + OFFLINE_EFF_PER_LVL * level)` (cap 1.0);
//! `sim_secs = floor(offline_secs * eff)` → jalankan `tick::step` sebanyak itu (tanpa render).
//!
//! NOTE(scaffold): `allow(dead_code)` — dipanggil `app::run` setelah load save (wiring M7 lanjut).
#![allow(dead_code)]

use crate::balance::{OFFLINE_BASE_EFF, OFFLINE_CAP_SECS, OFFLINE_EFF_PER_LVL};
use crate::game::defs::Content;
use crate::game::state::GameState;

/// Ringkasan progres offline (untuk panel "Selama kamu pergi: …").
#[derive(Clone, Debug, Default)]
pub struct OfflineReport {
    /// Waktu offline nyata setelah cap (detik).
    pub real_secs: u64,
    /// Detik yang benar-benar disimulasikan setelah efisiensi.
    pub sim_secs: u64,
    pub credits_gained: f64,
    pub data_gained: f64,
}

/// Efisiensi offline = `OFFLINE_BASE_EFF * (1 + OFFLINE_EFF_PER_LVL * level)`, dibatasi 1.0.
pub fn offline_efficiency(level: u32) -> f64 {
    (OFFLINE_BASE_EFF * (1.0 + OFFLINE_EFF_PER_LVL * level as f64)).min(1.0)
}

/// Hitung & terapkan progres offline pada `state` (batch `tick::step`). Update `last_saved_unix`.
/// Level upgrade efisiensi diambil dari `prestige.permanent_upgrades["offline_eff"]` (default 0).
pub fn apply_offline(state: &mut GameState, content: &Content, now_unix: u64) -> OfflineReport {
    let delta = now_unix.saturating_sub(state.last_saved_unix);
    let real_secs = delta.min(OFFLINE_CAP_SECS);
    let level = state
        .prestige
        .permanent_upgrades
        .get("offline_eff")
        .copied()
        .unwrap_or(0);
    let sim_secs = (real_secs as f64 * offline_efficiency(level)).floor() as u64;

    let credits_before = state.credits;
    let data_before = state.research.data;
    for _ in 0..sim_secs {
        crate::sim::tick::step(state, content);
        state.tick += 1;
    }
    state.last_saved_unix = now_unix;

    OfflineReport {
        real_secs,
        sim_secs,
        credits_gained: state.credits - credits_before,
        data_gained: state.research.data - data_before,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_content;
    use crate::game::defs::{BuildingId, ResourceId};
    use crate::game::state::{
        AutoSellRule, Biome, Factory, FactoryId, FactoryKind, Galaxy, GalaxyId, GalaxyKind, NodeId,
        Planet, PlanetId, ResourceMap, ResourceNode, Ship, UnlockReq,
    };

    fn content() -> Content {
        load_content(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data")).unwrap()
    }

    fn producing_state(c: &Content) -> GameState {
        let iron = ResourceId(c.resources.id("iron").unwrap());
        let energy = ResourceId(c.resources.id("energy").unwrap());
        let mut stockpile = ResourceMap::new();
        stockpile.insert(energy, 1_000_000.0);
        let planet = Planet {
            id: PlanetId(0),
            name: "Earth".into(),
            tier: 1,
            biome: Biome::Terran,
            distance: 1.0,
            unlocked: true,
            unlock_req: UnlockReq::None,
            nodes: vec![ResourceNode {
                id: NodeId(0),
                resource: iron,
                richness: 1.0,
                level: 1,
            }],
            factory_slots: vec![Some(Factory {
                id: FactoryId(0),
                building: BuildingId(c.buildings.id("mining_drill").unwrap()),
                kind: FactoryKind::Extractor { node: NodeId(0) },
                level: 1,
                enabled: true,
            })],
            stockpile,
            stockpile_cap: 10_000_000.0,
        };
        let mut st = GameState {
            version: 1,
            last_saved_unix: 0,
            tick: 0,
            credits: 0.0,
            inventory: std::collections::HashMap::new(),
            galaxies: vec![Galaxy {
                id: GalaxyId(0),
                name: "MW".into(),
                level: 0,
                kind: GalaxyKind::Fixed,
                planets: vec![planet],
            }],
            active_galaxy: GalaxyId(0),
            anchor_galaxy: GalaxyId(0),
            ship: Ship::default(),
            research: Default::default(),
            prestige: Default::default(),
            merchant: Default::default(),
            events: Default::default(),
            settings: Default::default(),
        };
        st.settings.auto_sell = vec![AutoSellRule {
            resource: iron,
            keep_above: 0.0,
            enabled: true,
        }];
        st
    }

    #[test]
    fn efficiency_base_and_capped() {
        assert!((offline_efficiency(0) - OFFLINE_BASE_EFF).abs() < 1e-9);
        // Level cukup tinggi → di-cap 1.0.
        assert!((offline_efficiency(100) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn applies_batched_income_with_efficiency() {
        let c = content();
        let mut st = producing_state(&c);
        // Offline 100 detik → sim = floor(100 * 0.75) = 75 tick.
        let report = apply_offline(&mut st, &c, 100);
        assert_eq!(report.real_secs, 100);
        assert_eq!(report.sim_secs, 75);
        // 1 iron/tick dijual @1 credit → 75 credits.
        assert!((report.credits_gained - 75.0).abs() < 1e-6);
        assert!((st.credits - 75.0).abs() < 1e-6);
        assert_eq!(st.tick, 75);
        assert_eq!(st.last_saved_unix, 100);
    }

    #[test]
    fn caps_offline_window() {
        let c = content();
        let mut st = producing_state(&c);
        // Delta jauh melebihi cap → real_secs = OFFLINE_CAP_SECS.
        let report = apply_offline(&mut st, &c, OFFLINE_CAP_SECS + 10_000);
        assert_eq!(report.real_secs, OFFLINE_CAP_SECS);
        assert_eq!(
            report.sim_secs,
            (OFFLINE_CAP_SECS as f64 * OFFLINE_BASE_EFF).floor() as u64
        );
    }
}
