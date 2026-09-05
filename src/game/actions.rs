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
    /// Building/recipe masih terkunci research (butuh tech `UnlockBuilding`/`Recipe`). `03` §5.
    TechLocked,
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

/// Resolusi (string,jumlah) → (ResourceId,jumlah), di-skala `scale`. `pub(crate)` M14.9: dipakai
/// jg UI cost-preview (`panels::planet`), bukan cuma internal `build_factory`/`upgrade_factory`.
pub(crate) fn resolve_cost(
    content: &Content,
    pairs: &[(String, f64)],
    scale: f64,
) -> Vec<(ResourceId, f64)> {
    pairs
        .iter()
        .filter_map(|(n, q)| content.resources.id(n).map(|h| (ResourceId(h), q * scale)))
        .collect()
}

/// M14.9: dipisah dari `try_pay` (bukan duplikasi cek) — dipakai NYATA jalur mutasi (`try_pay`
/// di bawah) DAN preview afford read-only di UI (`panels::planet`'s cost+afford indicator).
/// Satu sumber logic "cukup/tidak", bukan 2 tempat yg bisa divergen.
pub(crate) fn can_afford(
    state: &GameState,
    gi: usize,
    pi: usize,
    cost: &[(ResourceId, f64)],
    credits: Option<ResourceId>,
) -> bool {
    let planet = &state.galaxies[gi].planets[pi];
    cost.iter().all(|(r, amt)| {
        let have = if Some(*r) == credits {
            state.credits
        } else {
            planet.stockpile.get(r).copied().unwrap_or(0.0)
        };
        have >= *amt
    })
}

/// Cek mampu lalu bayar dari credits + stockpile planet. Tak memotong bila kurang.
fn try_pay(
    state: &mut GameState,
    gi: usize,
    pi: usize,
    cost: &[(ResourceId, f64)],
    credits: Option<ResourceId>,
) -> Result<(), ActionError> {
    if !can_afford(state, gi, pi, cost, credits) {
        return Err(ActionError::Insufficient);
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
    if !crate::game::research::building_available(state, content, &bdef.id) {
        return Err(ActionError::TechLocked);
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
        BuildingKind::Refinery => {
            let rid = recipe.ok_or(ActionError::MissingBinding)?;
            let recipe_name = &content.recipes.get(rid.0).id;
            if !crate::game::research::recipe_available(state, content, recipe_name) {
                return Err(ActionError::TechLocked);
            }
            FactoryKind::Refinery { recipe: rid }
        }
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

/// M14.12: daftar building yg BISA dibangun di planet `pi` SEKARANG — filter tier planet,
/// tech unlocked (`research::building_available`), slot cukup, DAN kind genuinely buildable
/// (`Storage`/`Special` selalu `Err(NotBuildable)` di `build_factory`, jadi disaring di sini
/// jg biar list tak tampilkan opsi yg pasti gagal bila dipilih).
pub fn buildable_options(state: &GameState, content: &Content, pi: usize) -> Vec<BuildingId> {
    let Ok(gi) = active_gi(state) else {
        return Vec::new();
    };
    let planet = &state.galaxies[gi].planets[pi];
    let free = planet.factory_slots.iter().filter(|s| s.is_none()).count();
    content
        .buildings
        .iter()
        .enumerate()
        .filter(|(_, b)| {
            matches!(
                b.kind,
                BuildingKind::Extractor | BuildingKind::Refinery | BuildingKind::ResearchLab
            )
        })
        .filter(|(_, b)| planet.tier >= b.min_planet_tier)
        .filter(|(_, b)| b.slot_cost as usize <= free)
        .filter(|(_, b)| crate::game::research::building_available(state, content, &b.id))
        .map(|(h, _)| BuildingId(h as u32))
        .collect()
}

/// M14.12: bangun `building` di `slot`, auto-pilih node/recipe (bukan minta user pilih 2 kali
/// — dicek `BuildingDef` TAK py field pembatas resource-per-node, jadi Extractor MANA PUN bisa
/// bind ke node MANA PUN; default deterministik: node PERTAMA yg blm dipakai extractor lain).
/// Refinery: recipe PERTAMA di `bdef.recipes` yg genuinely `recipe_available` (bukan cuma
/// recipe pertama SEADANYA — kalau recipe pertama msh terkunci tech tp recipe lain di building
/// yg sama SUDAH ada, pilih itu, biar tak gagal `TechLocked` sia-sia padahal ada alternatif).
pub fn build_picked(
    state: &mut GameState,
    content: &Content,
    pi: usize,
    slot: usize,
    building: BuildingId,
) -> Result<(), ActionError> {
    let gi = active_gi(state)?;
    let bdef = content.buildings.get(building.0);
    let node = match bdef.kind {
        BuildingKind::Extractor => {
            let used: std::collections::HashSet<NodeId> = state.galaxies[gi].planets[pi]
                .factory_slots
                .iter()
                .flatten()
                .filter_map(|f| match f.kind {
                    FactoryKind::Extractor { node } => Some(node),
                    _ => None,
                })
                .collect();
            state.galaxies[gi].planets[pi]
                .nodes
                .iter()
                .find(|n| !used.contains(&n.id))
                .map(|n| n.id)
        }
        _ => None,
    };
    let recipe = match bdef.kind {
        BuildingKind::Refinery => bdef
            .recipes
            .iter()
            .filter_map(|name| content.recipes.id(name).map(|h| (name, RecipeId(h))))
            .find(|(name, _)| crate::game::research::recipe_available(state, content, name))
            .map(|(_, rid)| rid),
        _ => None,
    };
    build_factory(state, content, pi, slot, building, node, recipe)
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
            distance: 1.0,
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
            tutorial_step: None,
            quests: Default::default(),
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
    fn tech_locked_building_then_unlocked() {
        let c = content();
        // deep_core_miner butuh tech ext_deep_core (UnlockBuilding) + tier 2.
        let dcm = BuildingId(c.buildings.id("deep_core_miner").unwrap());
        let mut st = state_t(2, 3);
        assert_eq!(
            build_factory(&mut st, &c, 0, 0, dcm, Some(NodeId(0)), None),
            Err(ActionError::TechLocked)
        );
        // Research selesai → gate tech terbuka (lalu gagal karena dana/steel, bukan TechLocked).
        st.research.completed.insert("ext_deep_core".into());
        assert_eq!(
            build_factory(&mut st, &c, 0, 0, dcm, Some(NodeId(0)), None),
            Err(ActionError::Insufficient) // butuh 500 credits + 50 steel.
        );
    }

    #[test]
    fn tech_locked_recipe_then_unlocked() {
        let c = content();
        // steel_mill recipe terkunci tech manu_steel.
        let mill = BuildingId(c.buildings.id("steel_mill_bld").unwrap());
        let steel = RecipeId(c.recipes.id("steel_mill").unwrap());
        let mut st = state_t(1, 3);
        assert_eq!(
            build_factory(&mut st, &c, 0, 0, mill, None, Some(steel)),
            Err(ActionError::TechLocked)
        );
        // Research selesai → recipe boleh dipilih → build sukses (base_cost 200 credits).
        st.research.completed.insert("manu_steel".into());
        build_factory(&mut st, &c, 0, 0, mill, None, Some(steel)).unwrap();
        assert!((st.credits - 800.0).abs() < 1e-9);
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

    /// M14.12: `buildable_options` HARUS saring tier (arc_furnace tier 2 tak boleh muncul di
    /// planet tier 1) DAN HARUS saring Storage/Special (`storage_depot`/`anchor_booster` —
    /// keduanya SELALU `Err(NotBuildable)` di `build_factory`, list tak boleh tawarkan opsi
    /// yg pasti gagal).
    #[test]
    fn buildable_options_filters_tier_and_kind() {
        let c = content();
        let st = state_t(1, 3);
        let opts = buildable_options(&st, &c, 0);
        let names: Vec<&str> = opts
            .iter()
            .map(|&b| c.buildings.get(b.0).id.as_str())
            .collect();
        assert!(names.contains(&"mining_drill"), "tier 1 harus muncul");
        assert!(
            !names.contains(&"arc_furnace"),
            "tier 2 TAK boleh muncul di planet tier 1: {names:?}"
        );
        assert!(
            !names.contains(&"storage_depot") && !names.contains(&"anchor_booster"),
            "Storage/Special TAK bisa dibangun via build_factory, tak boleh ditawarkan: {names:?}"
        );
    }

    /// M14.12: `build_picked` utk Extractor HARUS auto-bind ke node PERTAMA (dicek langsung
    /// hasil `FactoryKind::Extractor{node}`, bukan cuma "build sukses tanpa cek node mana").
    #[test]
    fn build_picked_extractor_auto_binds_node() {
        let c = content();
        let mut st = state_t(1, 3);
        st.galaxies[0].planets[0]
            .nodes
            .push(crate::game::state::ResourceNode {
                id: NodeId(7),
                resource: ResourceId(c.resources.id("iron").unwrap()),
                richness: 1.0,
                level: 1,
            });
        let drill = BuildingId(c.buildings.id("mining_drill").unwrap());
        build_picked(&mut st, &c, 0, 0, drill).unwrap();
        let f = st.galaxies[0].planets[0].factory_slots[0].unwrap();
        assert!(
            matches!(f.kind, FactoryKind::Extractor { node } if node == NodeId(7)),
            "harus auto-bind ke node yg ada (id 7), dpt {:?}",
            f.kind
        );
    }

    /// M14.12: `build_picked` utk Refinery HARUS auto-pilih recipe PERTAMA yg genuinely
    /// available (smelter: smelt_iron TAK tech-locked, beda dari steel_mill_bld yg recipe
    /// pertamanya "steel_mill" TERKUNCI manu_steel — dicek recipe yg kepilih PERSIS).
    #[test]
    fn build_picked_refinery_auto_picks_available_recipe() {
        let c = content();
        let mut st = state_t(1, 3);
        let smelter = BuildingId(c.buildings.id("smelter").unwrap());
        build_picked(&mut st, &c, 0, 0, smelter).unwrap();
        let f = st.galaxies[0].planets[0].factory_slots[0].unwrap();
        let expected = RecipeId(c.recipes.id("smelt_iron").unwrap());
        assert!(
            matches!(f.kind, FactoryKind::Refinery { recipe } if recipe.0 == expected.0),
            "harus pilih smelt_iron (recipe pertama & tak terkunci), dpt {:?}",
            f.kind
        );
    }
}
