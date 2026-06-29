//! `step()` — satu langkah simulasi 1 detik game-time, 9 fase berurutan (`07` §urutan tick).
//!
//! Idempoten thd urutan: produksi → konsumsi/craft → upkeep → auto-sell → research →
//! travel/event → anchor → merchant → buff. Caller (`app`) menaikkan `state.tick`, bukan di sini.
//!
//! NOTE(scaffold): `allow(dead_code)` — dipanggil `app::run` (M3). Fase 6–9 sebagian stub
//! (travel dasar ada; event/anchor/merchant/buff diisi M6/M8/M9).
#![allow(dead_code)]

use crate::balance::{BASE_DATA_RATE, TICK_DURATION_SECS};
use crate::game::defs::{Content, ResourceId};
use crate::game::economy::{auto_sell_planet, run_extractors, run_refineries};
use crate::game::state::{FactoryKind, GameState, ShipStatus};

/// Ringkasan hasil satu tick (untuk dashboard net income & feedback deficit).
#[derive(Clone, Debug, Default)]
pub struct TickReport {
    pub deficits: Vec<ResourceId>,
    pub energy_deficit: bool,
    pub credits_gained: f64,
    pub data_gained: f64,
}

/// Jalankan satu tick simulasi pada galaksi aktif + state global.
pub fn step(state: &mut GameState, content: &Content) -> TickReport {
    let energy = content.resources.id("energy").map(ResourceId);
    let mut report = TickReport::default();

    let Some(gi) = state
        .galaxies
        .iter()
        .position(|g| g.id == state.active_galaxy)
    else {
        return report;
    };
    // Clone agar tidak meminjam `state` saat memutasi galaksi aktif.
    let rules = state.settings.auto_sell.clone();
    let blueprints = state.prestige.blueprints.clone();
    let mut data_rate = 0.0;

    for planet in state.galaxies[gi].planets.iter_mut().filter(|p| p.unlocked) {
        // 1. Extractor: node → stockpile (tech_mult 1.0; research M5).
        run_extractors(planet, |_| 1.0);
        // 2. Refinery: raw → crafted (atau catat deficit).
        report
            .deficits
            .extend(run_refineries(planet, content, &blueprints));
        // 3. Upkeep: kurangi Energy; defisit → flag (penalti efisiensi M5+).
        if let Some(e) = energy {
            let mut cost = 0.0;
            for f in planet.factory_slots.iter().flatten() {
                if !f.enabled || f.level == 0 {
                    continue;
                }
                let b = content.buildings.get(f.building.0);
                for (res, amt) in &b.upkeep {
                    if content.resources.id(res) == Some(e.0) {
                        cost += amt; // upkeep tetap per-building (lihat asumsi tick).
                    }
                }
            }
            if cost > 0.0 {
                let have = planet.stockpile.get(&e).copied().unwrap_or(0.0);
                if have >= cost {
                    planet.stockpile.insert(e, have - cost);
                } else {
                    planet.stockpile.insert(e, 0.0);
                    report.energy_deficit = true;
                }
            }
        }
        // 4. Auto-sell: surplus → credits.
        report.credits_gained += auto_sell_planet(planet, content, &rules);
        // 5a. Research: kumpulkan Data/sec dari ResearchLab enabled.
        for f in planet.factory_slots.iter().flatten() {
            if f.enabled && f.level > 0 && matches!(f.kind, FactoryKind::ResearchLab) {
                data_rate += f.level as f64 * BASE_DATA_RATE;
            }
        }
    }

    state.credits += report.credits_gained;
    // 5b. Research: tambah Data; majukan active research (completion → M5).
    report.data_gained = data_rate * TICK_DURATION_SECS;
    state.research.data += report.data_gained;
    if let Some(active) = state.research.active.as_mut() {
        active.elapsed_secs += TICK_DURATION_SECS;
    }
    // 6. Ship travel: majukan elapsed; resolusi tiba. (Roll event → M6/M9.)
    advance_travel(state);
    // 7. Anchor feed → M8. 8. Merchant expire/restock → M9. 9. Buff expiry → M9.

    report
}

fn advance_travel(state: &mut GameState) {
    if let ShipStatus::Traveling {
        to,
        total_secs,
        elapsed_secs,
    } = state.ship.status
    {
        let elapsed = elapsed_secs + TICK_DURATION_SECS;
        if elapsed >= total_secs {
            for g in state.galaxies.iter_mut() {
                for p in g.planets.iter_mut() {
                    if p.id == to {
                        p.unlocked = true;
                    }
                }
            }
            state.ship.status = ShipStatus::Idle;
        } else {
            state.ship.status = ShipStatus::Traveling {
                to,
                total_secs,
                elapsed_secs: elapsed,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_content;
    use crate::game::defs::BuildingId;
    use crate::game::state::{
        AutoSellRule, Biome, Factory, FactoryId, Galaxy, GalaxyId, GalaxyKind, NodeId, Planet,
        PlanetId, ResourceMap, ResourceNode, Ship, ShipStatus, UnlockReq,
    };

    fn content() -> Content {
        load_content(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data")).unwrap()
    }

    fn state_with(planet: Planet) -> GameState {
        GameState {
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
        }
    }

    #[test]
    fn one_tick_net_income() {
        let c = content();
        let iron = ResourceId(c.resources.id("iron").unwrap());
        let energy = ResourceId(c.resources.id("energy").unwrap());
        let mut p = Planet {
            id: PlanetId(0),
            name: "Earth".into(),
            tier: 1,
            biome: Biome::Terran,
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
            stockpile: ResourceMap::new(),
            stockpile_cap: 1000.0,
        };
        p.stockpile.insert(energy, 10.0);
        let mut st = state_with(p);
        st.settings.auto_sell = vec![AutoSellRule {
            resource: iron,
            keep_above: 0.0,
            enabled: true,
        }];

        let r = step(&mut st, &c);
        // 1 iron ditambang → dijual semua → +1 credit (price iron 1).
        assert!((r.credits_gained - 1.0).abs() < 1e-9);
        assert!((st.credits - 1.0).abs() < 1e-9);
        // Upkeep mining_drill = 1 energy → 10 - 1 = 9, tidak defisit.
        assert!(!r.energy_deficit);
        assert!((st.galaxies[0].planets[0].stockpile[&energy] - 9.0).abs() < 1e-9);
    }

    #[test]
    fn energy_deficit_flagged() {
        let c = content();
        let mut p = Planet {
            id: PlanetId(0),
            name: "Earth".into(),
            tier: 1,
            biome: Biome::Terran,
            unlocked: true,
            unlock_req: UnlockReq::None,
            nodes: vec![],
            factory_slots: vec![Some(Factory {
                id: FactoryId(0),
                building: BuildingId(c.buildings.id("smelter").unwrap()),
                kind: FactoryKind::ResearchLab,
                level: 1,
                enabled: true,
            })],
            stockpile: ResourceMap::new(),
            stockpile_cap: 1000.0,
        };
        let _ = &mut p;
        let mut st = state_with(p);
        let r = step(&mut st, &c);
        assert!(r.energy_deficit); // tak ada energy → upkeep smelter tak terbayar.
    }

    #[test]
    fn travel_arrival_unlocks() {
        let c = content();
        let p = Planet {
            id: PlanetId(7),
            name: "Luna".into(),
            tier: 1,
            biome: Biome::DeadWorld,
            unlocked: false,
            unlock_req: UnlockReq::WarpTier(1),
            nodes: vec![],
            factory_slots: vec![],
            stockpile: ResourceMap::new(),
            stockpile_cap: 1000.0,
        };
        let mut st = state_with(p);
        st.ship.status = ShipStatus::Traveling {
            to: PlanetId(7),
            total_secs: 1.0,
            elapsed_secs: 0.0,
        };
        let _ = step(&mut st, &c);
        assert!(matches!(st.ship.status, ShipStatus::Idle));
        assert!(st.galaxies[0].planets[0].unlocked);
    }
}
