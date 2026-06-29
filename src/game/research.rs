//! Research: Data/sec dari ResearchLab, alokasi ke satu tech node aktif, completion + efek.
//!
//! Model (`03-progression.md` §5): ResearchLab menghasilkan Data/sec → terkumpul di pool
//! `ResearchState.data`. Player memilih **satu** tech `active`; tiap tick Data dialokasikan
//! dari pool ke `data_invested` (cap di `data_cost`) dan `elapsed_secs` maju. Tech selesai
//! bila `data_invested >= data_cost` DAN `elapsed_secs >= time_secs` → masuk `completed`,
//! efek permanen diterapkan. State hanya simpan `completed` + `active` (`01` §Research).
//!
//! NOTE(scaffold): `allow(dead_code)` — `start_research`/`available_techs` dipanggil UI M5 item 3.
#![allow(dead_code)]

use crate::balance::{BASE_DATA_RATE, TICK_DURATION_SECS};
use crate::game::defs::{Content, ResourceId, TechDef, TechUnlock};
use crate::game::state::{ActiveResearch, FactoryKind, Galaxy, GameState};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq)]
pub enum ResearchError {
    UnknownTech,
    AlreadyCompleted,
    AlreadyActive,
    PrereqNotMet,
}

/// Lookup `TechDef` dari id string (interned via `Content`).
pub fn tech_def<'a>(content: &'a Content, id: &str) -> Option<&'a TechDef> {
    content.techs.id(id).map(|h| content.techs.get(h))
}

/// Data/sec dari semua ResearchLab enabled di galaksi (planet unlocked). `03` §5.
pub fn data_rate(galaxy: &Galaxy) -> f64 {
    let mut rate = 0.0;
    for p in galaxy.planets.iter().filter(|p| p.unlocked) {
        for f in p.factory_slots.iter().flatten() {
            if f.enabled && f.level > 0 && matches!(f.kind, FactoryKind::ResearchLab) {
                rate += f.level as f64 * BASE_DATA_RATE;
            }
        }
    }
    rate
}

/// True bila tech bisa dimulai sekarang: ada, belum selesai, semua `depends_on` selesai.
pub fn is_available(state: &GameState, content: &Content, tech_id: &str) -> bool {
    let Some(tech) = tech_def(content, tech_id) else {
        return false;
    };
    !state.research.completed.contains(tech_id)
        && tech
            .depends_on
            .iter()
            .all(|d| state.research.completed.contains(d))
}

/// Daftar id tech yang available (untuk UI research terminal). Urut sesuai definisi katalog.
pub fn available_techs<'a>(state: &GameState, content: &'a Content) -> Vec<&'a str> {
    content
        .techs
        .iter()
        .filter(|t| is_available(state, content, &t.id))
        .map(|t| t.id.as_str())
        .collect()
}

/// Pilih tech aktif. Gagal bila tak dikenal, sudah selesai, sudah ada yang aktif, atau prasyarat
/// belum lengkap. Hanya **satu** research aktif (`03` §5).
pub fn start_research(
    state: &mut GameState,
    content: &Content,
    tech_id: &str,
) -> Result<(), ResearchError> {
    let tech = tech_def(content, tech_id).ok_or(ResearchError::UnknownTech)?;
    if state.research.completed.contains(tech_id) {
        return Err(ResearchError::AlreadyCompleted);
    }
    if state.research.active.is_some() {
        return Err(ResearchError::AlreadyActive);
    }
    if !tech
        .depends_on
        .iter()
        .all(|d| state.research.completed.contains(d))
    {
        return Err(ResearchError::PrereqNotMet);
    }
    state.research.active = Some(ActiveResearch {
        tech_id: tech_id.to_string(),
        data_invested: 0.0,
        elapsed_secs: 0.0,
    });
    Ok(())
}

/// Majukan research aktif satu tick: tambah `data_gained` ke pool, alokasikan ke tech aktif, dan
/// selesaikan bila Data + waktu terpenuhi. Kembalikan id tech yang selesai tick ini (feedback UI).
pub fn advance(state: &mut GameState, content: &Content, data_gained: f64) -> Option<String> {
    state.research.data += data_gained;
    let active = state.research.active.as_mut()?;
    let tech = tech_def(content, &active.tech_id)?.clone();
    // Alokasi: tarik Data dari pool ke investasi, dibatasi sisa data_cost.
    let remaining = (tech.data_cost - active.data_invested).max(0.0);
    let take = remaining.min(state.research.data);
    active.data_invested += take;
    state.research.data -= take;
    active.elapsed_secs += TICK_DURATION_SECS;

    if active.data_invested >= tech.data_cost && active.elapsed_secs >= tech.time_secs {
        state.research.active = None;
        state.research.completed.insert(tech.id.clone());
        apply_unlock(state, content, &tech.unlock);
        Some(tech.id)
    } else {
        None
    }
}

/// Terapkan efek permanen tech yang selesai. WarpTier langsung ke ship; TechMult diturunkan
/// on-the-fly oleh [`tech_mult`]. Recipe/UnlockBuilding/AccessOuterTier cukup tercatat di
/// `completed` (dikonsumsi gate building/recipe/galaksi pada modul terkait).
fn apply_unlock(state: &mut GameState, _content: &Content, unlock: &TechUnlock) {
    if let TechUnlock::WarpTier(n) = unlock
        && *n > state.ship.warp_tier
    {
        state.ship.warp_tier = *n;
    }
}

/// Multiplier produksi resource dari semua tech `TechMult` yang selesai. Default 1.0.
/// `completed` di-pass terpisah agar bisa dipakai di dalam loop mutasi galaksi (`sim::tick`).
pub fn tech_mult(completed: &HashSet<String>, content: &Content, res: ResourceId) -> f64 {
    let mut mult = 1.0;
    for id in completed {
        if let Some(t) = tech_def(content, id)
            && let TechUnlock::TechMult { resources, add } = &t.unlock
            && resources
                .iter()
                .any(|r| content.resources.id(r) == Some(res.0))
        {
            mult += add;
        }
    }
    mult
}

/// Tech id yang membuka `recipe_id` via `TechUnlock::Recipe`, bila ada (recipe tech-locked).
pub fn recipe_locked_by<'a>(content: &'a Content, recipe_id: &str) -> Option<&'a str> {
    content.techs.iter().find_map(|t| match &t.unlock {
        TechUnlock::Recipe(r) if r == recipe_id => Some(t.id.as_str()),
        _ => None,
    })
}

/// Tech id yang membuka `building_id` via `TechUnlock::UnlockBuilding`, bila ada.
pub fn building_locked_by<'a>(content: &'a Content, building_id: &str) -> Option<&'a str> {
    content.techs.iter().find_map(|t| match &t.unlock {
        TechUnlock::UnlockBuilding(b) if b == building_id => Some(t.id.as_str()),
        _ => None,
    })
}

/// Recipe boleh dipilih saat build refinery: tidak tech-locked, atau tech pembukanya selesai.
/// (Refinery yang sudah berjalan tak dicek ulang — `02` §3 hanya gate blueprint.)
pub fn recipe_available(state: &GameState, content: &Content, recipe_id: &str) -> bool {
    match recipe_locked_by(content, recipe_id) {
        Some(tech) => state.research.completed.contains(tech),
        None => true,
    }
}

/// Building boleh dibangun: tidak tech-locked, atau tech `UnlockBuilding` pembukanya selesai.
pub fn building_available(state: &GameState, content: &Content, building_id: &str) -> bool {
    match building_locked_by(content, building_id) {
        Some(tech) => state.research.completed.contains(tech),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_content;
    use crate::game::defs::BuildingId;
    use crate::game::state::{
        Biome, Factory, FactoryId, Galaxy, GalaxyId, GalaxyKind, Planet, PlanetId, ResourceMap,
        Ship, UnlockReq,
    };

    fn content() -> Content {
        load_content(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data")).unwrap()
    }

    fn lab_planet(c: &Content, level: u32) -> Planet {
        Planet {
            id: PlanetId(0),
            name: "Earth".into(),
            tier: 1,
            biome: Biome::Terran,
            distance: 1.0,
            unlocked: true,
            unlock_req: UnlockReq::None,
            nodes: vec![],
            factory_slots: vec![Some(Factory {
                id: FactoryId(0),
                building: BuildingId(c.buildings.id("research_lab").unwrap_or(0)),
                kind: FactoryKind::ResearchLab,
                level,
                enabled: true,
            })],
            stockpile: ResourceMap::new(),
            stockpile_cap: 1000.0,
        }
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
    fn data_rate_sums_labs() {
        let c = content();
        let g = &state_with(lab_planet(&c, 3)).galaxies[0];
        // 3 level × BASE_DATA_RATE(5) = 15 /s.
        assert!((data_rate(g) - 15.0).abs() < 1e-9);
    }

    #[test]
    fn start_respects_prereq_and_single_active() {
        let c = content();
        let mut st = state_with(lab_planet(&c, 1));
        // manu_titanium butuh manu_steel dulu.
        assert_eq!(
            start_research(&mut st, &c, "manu_titanium"),
            Err(ResearchError::PrereqNotMet)
        );
        // manu_steel tanpa prasyarat → boleh.
        start_research(&mut st, &c, "manu_steel").unwrap();
        // sudah ada aktif → tolak.
        assert_eq!(
            start_research(&mut st, &c, "manu_steel"),
            Err(ResearchError::AlreadyActive)
        );
    }

    #[test]
    fn completes_when_data_and_time_met() {
        let c = content();
        let mut st = state_with(lab_planet(&c, 1));
        start_research(&mut st, &c, "manu_steel").unwrap();
        // manu_steel: data_cost 500, time 120s. Dorong Data berlebih + cukup waktu.
        let mut done = None;
        for _ in 0..200 {
            done = advance(&mut st, &c, 10.0).or(done);
        }
        assert_eq!(done.as_deref(), Some("manu_steel"));
        assert!(st.research.completed.contains("manu_steel"));
        assert!(st.research.active.is_none());
    }

    #[test]
    fn warp_tier_applied_on_completion() {
        let c = content();
        let mut st = state_with(lab_planet(&c, 1));
        // Pra-selesaikan rantai prasyarat aero_warp_mk2.
        for id in ["manu_steel", "manu_titanium"] {
            st.research.completed.insert(id.to_string());
        }
        start_research(&mut st, &c, "aero_warp_mk2").unwrap();
        for _ in 0..700 {
            advance(&mut st, &c, 100.0);
        }
        assert!(st.research.completed.contains("aero_warp_mk2"));
        assert_eq!(st.ship.warp_tier, 6); // WarpTier(6).
    }

    #[test]
    fn tech_mult_from_completed() {
        let c = content();
        let iron = ResourceId(c.resources.id("iron").unwrap());
        let copper = ResourceId(c.resources.id("copper").unwrap_or(iron.0));
        let mut completed = HashSet::new();
        assert!((tech_mult(&completed, &c, iron) - 1.0).abs() < 1e-9);
        // ext_efficiency: +0.25 utk iron & carbon.
        completed.insert("ext_efficiency".to_string());
        assert!((tech_mult(&completed, &c, iron) - 1.25).abs() < 1e-9);
        let _ = copper;
    }
}
