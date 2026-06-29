//! Aksi player: build / upgrade factory & node. Aturan slot & tier (`02` §5, `13`).
//!
//! Biaya geometrik `base * GROWTH^level` dibayar dari `GameState.credits` (entri "credits")
//! + stockpile planet (resource lain). Operasi pada galaksi aktif.
//!
//! NOTE(scaffold): `allow(dead_code)` — dipanggil dari UI aksi (M4 keybinding).
#![allow(dead_code)]

use crate::balance::{COST_NODE_LEVEL, GROWTH};
use crate::game::defs::{BuildingId, BuildingKind, Content, RecipeId, ResourceId};
use crate::game::economy::upgrade_cost;
use crate::game::state::{Factory, FactoryId, FactoryKind, GameState, NodeId};

#[derive(Debug, PartialEq, Eq)]
pub enum ActionError {
    NoActiveGalaxy,
    PlanetOutOfRange,
    SlotOutOfRange,
    NodeOutOfRange,
    SlotOccupied,
    SlotEmpty,
    NoFreeSlots,
    TierTooLow,
    NotBuildable,
    MissingBinding,
    Insufficient,
}

fn active_gi(state: &GameState) -> Result<usize, ActionError> {
    state
        .galaxies
        .iter()
        .position(|g| g.id == state.active_galaxy)
        .ok_or(ActionError::NoActiveGalaxy)
}

fn credits_id(content: &Content) -> Option<ResourceId> {
    content.resources.id("credits").map(ResourceId)
}

/// Resolusi (string,jumlah) → (ResourceId,jumlah), di-skala `scale`.
fn resolve_cost(content: &Content, pairs: &[(String, f64)], scale: f64) -> Vec<(ResourceId, f64)> {
    pairs
        .iter()
        .filter_map(|(n, q)| content.resources.id(n).map(|h| (ResourceId(h), q * scale)))
        .collect()
}

/// Cek mampu lalu bayar dari credits + stockpile planet. Tak memotong bila kurang.
fn try_pay(
    state: &mut GameState,
    gi: usize,
    pi: usize,
    cost: &[(ResourceId, f64)],
    credits: Option<ResourceId>,
) -> Result<(), ActionError> {
    let planet = &state.galaxies[gi].planets[pi];
    for (r, amt) in cost {
        let have = if Some(*r) == credits {
            state.credits
        } else {
            planet.stockpile.get(r).copied().unwrap_or(0.0)
        };
        if have < *amt {
            return Err(ActionError::Insufficient);
        }
    }
    let planet = &mut state.galaxies[gi].planets[pi];
    for (r, amt) in cost {
        if Some(*r) != credits {
            *planet.stockpile.entry(*r).or_insert(0.0) -= amt;
        }
    }
    for (r, amt) in cost {
        if Some(*r) == credits {
            state.credits -= amt;
        }
    }
    Ok(())
}

/// Bangun factory di slot kosong. Hormati tier planet & jumlah slot bebas (`slot_cost`).
pub fn build_factory(
    state: &mut GameState,
    content: &Content,
    pi: usize,
    slot: usize,
    building: BuildingId,
    node: Option<NodeId>,
    recipe: Option<RecipeId>,
) -> Result<(), ActionError> {
    let gi = active_gi(state)?;
    let bdef = content.buildings.get(building.0);
    let planet = &state.galaxies[gi].planets[pi];
    if slot >= planet.factory_slots.len() {
        return Err(ActionError::SlotOutOfRange);
    }
    if planet.tier < bdef.min_planet_tier {
        return Err(ActionError::TierTooLow);
    }
    if planet.factory_slots[slot].is_some() {
        return Err(ActionError::SlotOccupied);
    }
    let free = planet.factory_slots.iter().filter(|s| s.is_none()).count();
    if bdef.slot_cost as usize > free {
        return Err(ActionError::NoFreeSlots);
    }
    let kind = match bdef.kind {
        BuildingKind::Extractor => FactoryKind::Extractor {
            node: node.ok_or(ActionError::MissingBinding)?,
        },
        BuildingKind::Refinery => FactoryKind::Refinery {
            recipe: recipe.ok_or(ActionError::MissingBinding)?,
        },
        BuildingKind::ResearchLab => FactoryKind::ResearchLab,
        BuildingKind::Storage | BuildingKind::Special => return Err(ActionError::NotBuildable),
    };
    let cost = resolve_cost(content, &bdef.base_cost, 1.0);
    try_pay(state, gi, pi, &cost, credits_id(content))?;
    let fid = FactoryId(next_factory_id(state, gi, pi));
    state.galaxies[gi].planets[pi].factory_slots[slot] = Some(Factory {
        id: fid,
        building,
        kind,
        level: 1,
        enabled: true,
    });
    Ok(())
}

fn next_factory_id(state: &GameState, gi: usize, pi: usize) -> u32 {
    state.galaxies[gi].planets[pi]
        .factory_slots
        .iter()
        .flatten()
        .map(|f| f.id.0)
        .max()
        .map_or(0, |m| m + 1)
}

/// Naikkan level factory (biaya = base_cost building × GROWTH^level).
pub fn upgrade_factory(
    state: &mut GameState,
    content: &Content,
    pi: usize,
    slot: usize,
) -> Result<(), ActionError> {
    let gi = active_gi(state)?;
    if pi >= state.galaxies[gi].planets.len() {
        return Err(ActionError::PlanetOutOfRange);
    }
    let planet = &state.galaxies[gi].planets[pi];
    if slot >= planet.factory_slots.len() {
        return Err(ActionError::SlotOutOfRange);
    }
    let Some(fac) = planet.factory_slots[slot] else {
        return Err(ActionError::SlotEmpty);
    };
    let bdef = content.buildings.get(fac.building.0);
    let cost = resolve_cost(content, &bdef.base_cost, GROWTH.powi(fac.level as i32));
    try_pay(state, gi, pi, &cost, credits_id(content))?;
    if let Some(f) = state.galaxies[gi].planets[pi].factory_slots[slot].as_mut() {
        f.level += 1;
    }
    Ok(())
}

/// Naikkan level node (biaya Credits = `COST_NODE_LEVEL × GROWTH^level`).
pub fn upgrade_node(state: &mut GameState, pi: usize, node: usize) -> Result<(), ActionError> {
    let gi = active_gi(state)?;
    if pi >= state.galaxies[gi].planets.len() {
        return Err(ActionError::PlanetOutOfRange);
    }
    let planet = &state.galaxies[gi].planets[pi];
    if node >= planet.nodes.len() {
        return Err(ActionError::NodeOutOfRange);
    }
    let cost = upgrade_cost(COST_NODE_LEVEL, planet.nodes[node].level);
    if state.credits < cost {
        return Err(ActionError::Insufficient);
    }
    state.credits -= cost;
    state.galaxies[gi].planets[pi].nodes[node].level += 1;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_content;
    use crate::game::state::{
        Biome, Galaxy, GalaxyId, GalaxyKind, GameState, Planet, PlanetId, ResourceMap, Ship,
        UnlockReq,
    };

    fn content() -> Content {
        load_content(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data")).unwrap()
    }

    fn state_t(tier: u8, slots: usize) -> GameState {
        let planet = Planet {
            id: PlanetId(0),
            name: "P".into(),
            tier,
            biome: Biome::Terran,
            unlocked: true,
            unlock_req: UnlockReq::None,
            nodes: vec![],
            factory_slots: vec![None; slots],
            stockpile: ResourceMap::new(),
            stockpile_cap: 10000.0,
        };
        GameState {
            version: 1,
            last_saved_unix: 0,
            tick: 0,
            credits: 1000.0,
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
    fn build_then_upgrade_drill() {
        let c = content();
        let mut st = state_t(1, 3);
        let drill = BuildingId(c.buildings.id("mining_drill").unwrap());
        build_factory(&mut st, &c, 0, 0, drill, Some(NodeId(0)), None).unwrap();
        // base_cost mining_drill = 50 credits → 1000-50 = 950.
        assert!((st.credits - 950.0).abs() < 1e-9);
        let f = st.galaxies[0].planets[0].factory_slots[0].unwrap();
        assert_eq!(f.level, 1);
        // upgrade: 50 * GROWTH^1 = 57.5 → 950-57.5 = 892.5.
        upgrade_factory(&mut st, &c, 0, 0).unwrap();
        assert!((st.credits - 892.5).abs() < 1e-9);
        assert_eq!(st.galaxies[0].planets[0].factory_slots[0].unwrap().level, 2);
    }

    #[test]
    fn tier_gate_and_funds() {
        let c = content();
        // arc_furnace butuh tier 2.
        let furnace = BuildingId(c.buildings.id("arc_furnace").unwrap());
        let mut low = state_t(1, 3);
        assert_eq!(
            build_factory(&mut low, &c, 0, 0, furnace, None, None),
            Err(ActionError::TierTooLow)
        );
        // dana kurang.
        let mut broke = state_t(1, 3);
        broke.credits = 10.0;
        let drill = BuildingId(c.buildings.id("mining_drill").unwrap());
        assert_eq!(
            build_factory(&mut broke, &c, 0, 0, drill, Some(NodeId(0)), None),
            Err(ActionError::Insufficient)
        );
    }

    #[test]
    fn occupied_slot_rejected() {
        let c = content();
        let mut st = state_t(1, 1);
        let drill = BuildingId(c.buildings.id("mining_drill").unwrap());
        build_factory(&mut st, &c, 0, 0, drill, Some(NodeId(0)), None).unwrap();
        assert_eq!(
            build_factory(&mut st, &c, 0, 0, drill, Some(NodeId(0)), None),
            Err(ActionError::SlotOccupied)
        );
    }
}
