//! DTO save (id konten sebagai **string** untuk portabilitas) ↔ `GameState` runtime (id interned).
//!
//! `07-architecture.md` §Save ↔ Content: handle `u32` interned tak stabil bila urutan `data/`
//! berubah, jadi JSON menyimpan id string (`"iron"`). Saat load: string → handle via `Content`;
//! id yang tak dikenal (konten dihapus) → **skip**, jangan crash.
//!
//! Id **instance** (Galaxy/Planet/Node/Factory) tetap `u32` — stabil di dalam satu save (saling
//! rujuk internal). `merchant`/`events` (transient, M9) tak disimpan → default saat load.

use crate::game::defs::Content;
use crate::game::state::{
    AutoSellRule, Factory, FactoryId, FactoryKind, Galaxy, GalaxyId, GalaxyKind, GameState, NodeId,
    Planet, PlanetId, PrestigeState, ResourceMap, ResourceNode, Settings, UnlockReq,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub last_saved_unix: u64,
    pub tick: u64,
    pub credits: f64,
    pub inventory: Vec<(String, u32)>,
    pub galaxies: Vec<SaveGalaxy>,
    pub active_galaxy: u32,
    pub anchor_galaxy: u32,
    pub ship: crate::game::state::Ship,
    pub research: crate::game::state::ResearchState,
    pub prestige: SavePrestige,
    pub settings: SaveSettings,
    /// M3: `#[serde(default)]` — v1 save lama tak py field ini di JSON; `migrate` (`save/mod.rs`)
    /// jg menambahkannya eksplisit ke `null` sblm deserialize, tp default di sini jd jaring
    /// pengaman kedua (konsisten prinsip modul "id tak dikenal → skip, jangan crash").
    #[serde(default)]
    pub tutorial_step: Option<u8>,
    /// M6: `QuestState` sudah String-keyed (quest/stage id) sama spt `research` di atas --
    /// tak perlu DTO konversi terpisah, disimpan RAW (pola SAMA `research`). `#[serde(default)]`
    /// spy save v1/v2 (blm py quest) tetap load bersih (`migrate` jg tambahkan eksplisit).
    #[serde(default)]
    pub quests: crate::game::state::QuestState,
}

#[derive(Serialize, Deserialize)]
pub struct SaveGalaxy {
    pub id: u32,
    pub name: String,
    pub level: u8,
    pub kind: GalaxyKind,
    pub planets: Vec<SavePlanet>,
}

#[derive(Serialize, Deserialize)]
pub struct SavePlanet {
    pub id: u32,
    pub name: String,
    pub tier: u8,
    pub biome: crate::game::state::Biome,
    pub distance: f64,
    pub unlocked: bool,
    pub unlock_req: SaveUnlockReq,
    pub nodes: Vec<SaveNode>,
    pub factory_slots: Vec<Option<SaveFactory>>,
    pub stockpile: Vec<(String, f64)>,
    pub stockpile_cap: f64,
}

#[derive(Serialize, Deserialize)]
pub struct SaveNode {
    pub id: u32,
    pub resource: String,
    pub richness: f64,
    pub level: u32,
}

#[derive(Serialize, Deserialize)]
pub struct SaveFactory {
    pub id: u32,
    pub building: String,
    pub kind: SaveFactoryKind,
    pub level: u32,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub enum SaveFactoryKind {
    Extractor { node: u32 },
    Refinery { recipe: String },
    ResearchLab,
}

#[derive(Serialize, Deserialize)]
pub enum SaveUnlockReq {
    None,
    WarpTier(u8),
    Resource(String, f64),
    All(Vec<SaveUnlockReq>),
}

#[derive(Serialize, Deserialize)]
pub struct SavePrestige {
    pub warp_cores: u64,
    pub permanent_upgrades: HashMap<String, u32>,
    pub blueprints: Vec<String>,
    pub galaxy_level_reached: u8,
    pub anchor: crate::game::state::AnchorState,
}

#[derive(Serialize, Deserialize)]
pub struct SaveSettings {
    pub auto_sell: Vec<SaveAutoSell>,
    pub theme: crate::game::state::ThemeChoice,
}

#[derive(Serialize, Deserialize)]
pub struct SaveAutoSell {
    pub resource: String,
    pub keep_above: f64,
    pub enabled: bool,
}

// ── handle → string (save) ──────────────────────────────────────────────────
fn res_name(c: &Content, r: crate::game::defs::ResourceId) -> String {
    c.resources.get(r.0).id.clone()
}

fn save_req(c: &Content, req: &UnlockReq) -> SaveUnlockReq {
    match req {
        UnlockReq::None => SaveUnlockReq::None,
        UnlockReq::WarpTier(t) => SaveUnlockReq::WarpTier(*t),
        UnlockReq::Resource(r, a) => SaveUnlockReq::Resource(res_name(c, *r), *a),
        UnlockReq::All(l) => SaveUnlockReq::All(l.iter().map(|r| save_req(c, r)).collect()),
    }
}

fn save_factory(c: &Content, f: &Factory) -> SaveFactory {
    let kind = match f.kind {
        FactoryKind::Extractor { node } => SaveFactoryKind::Extractor { node: node.0 },
        FactoryKind::Refinery { recipe } => SaveFactoryKind::Refinery {
            recipe: c.recipes.get(recipe.0).id.clone(),
        },
        FactoryKind::ResearchLab => SaveFactoryKind::ResearchLab,
    };
    SaveFactory {
        id: f.id.0,
        building: c.buildings.get(f.building.0).id.clone(),
        kind,
        level: f.level,
        enabled: f.enabled,
    }
}

fn save_planet(c: &Content, p: &Planet) -> SavePlanet {
    SavePlanet {
        id: p.id.0,
        name: p.name.clone(),
        tier: p.tier,
        biome: p.biome,
        distance: p.distance,
        unlocked: p.unlocked,
        unlock_req: save_req(c, &p.unlock_req),
        nodes: p
            .nodes
            .iter()
            .map(|n| SaveNode {
                id: n.id.0,
                resource: res_name(c, n.resource),
                richness: n.richness,
                level: n.level,
            })
            .collect(),
        factory_slots: p
            .factory_slots
            .iter()
            .map(|s| s.as_ref().map(|f| save_factory(c, f)))
            .collect(),
        stockpile: p
            .stockpile
            .iter()
            .map(|(r, v)| (res_name(c, *r), *v))
            .collect(),
        stockpile_cap: p.stockpile_cap,
    }
}

/// `GameState` → `SaveData` (id konten jadi string). `merchant`/`events` dilewati (default load).
pub fn to_save(state: &GameState, c: &Content) -> SaveData {
    SaveData {
        version: crate::game::state::SAVE_VERSION,
        last_saved_unix: state.last_saved_unix,
        tick: state.tick,
        credits: state.credits,
        inventory: state
            .inventory
            .iter()
            .map(|(i, n)| (c.items.get(i.0).id.clone(), *n))
            .collect(),
        galaxies: state
            .galaxies
            .iter()
            .map(|g| SaveGalaxy {
                id: g.id.0,
                name: g.name.clone(),
                level: g.level,
                kind: g.kind.clone(),
                planets: g.planets.iter().map(|p| save_planet(c, p)).collect(),
            })
            .collect(),
        active_galaxy: state.active_galaxy.0,
        anchor_galaxy: state.anchor_galaxy.0,
        ship: state.ship,
        research: state.research.clone(),
        prestige: SavePrestige {
            warp_cores: state.prestige.warp_cores,
            permanent_upgrades: state.prestige.permanent_upgrades.clone(),
            blueprints: state
                .prestige
                .blueprints
                .iter()
                .map(|r| c.recipes.get(r.0).id.clone())
                .collect(),
            galaxy_level_reached: state.prestige.galaxy_level_reached,
            anchor: state.prestige.anchor,
        },
        settings: SaveSettings {
            auto_sell: state
                .settings
                .auto_sell
                .iter()
                .map(|r| SaveAutoSell {
                    resource: res_name(c, r.resource),
                    keep_above: r.keep_above,
                    enabled: r.enabled,
                })
                .collect(),
            theme: state.settings.theme,
        },
        tutorial_step: state.tutorial_step,
        quests: state.quests.clone(),
    }
}

// ── string → handle (load); id tak dikenal → skip ───────────────────────────
fn load_req(c: &Content, req: &SaveUnlockReq) -> UnlockReq {
    match req {
        SaveUnlockReq::None => UnlockReq::None,
        SaveUnlockReq::WarpTier(t) => UnlockReq::WarpTier(*t),
        SaveUnlockReq::Resource(r, a) => match c.resources.id(r) {
            Some(h) => UnlockReq::Resource(crate::game::defs::ResourceId(h), *a),
            None => UnlockReq::None, // resource hilang → gerbang terbuka (aman).
        },
        SaveUnlockReq::All(l) => UnlockReq::All(l.iter().map(|r| load_req(c, r)).collect()),
    }
}

/// `Option<SaveFactory>` → `Option<Factory>`; building/recipe hilang → slot kosong.
fn load_factory(c: &Content, f: &SaveFactory) -> Option<Factory> {
    let building = crate::game::defs::BuildingId(c.buildings.id(&f.building)?);
    let kind = match &f.kind {
        SaveFactoryKind::Extractor { node } => FactoryKind::Extractor {
            node: NodeId(*node),
        },
        SaveFactoryKind::Refinery { recipe } => FactoryKind::Refinery {
            recipe: crate::game::defs::RecipeId(c.recipes.id(recipe)?),
        },
        SaveFactoryKind::ResearchLab => FactoryKind::ResearchLab,
    };
    Some(Factory {
        id: FactoryId(f.id),
        building,
        kind,
        level: f.level,
        enabled: f.enabled,
    })
}

fn load_planet(c: &Content, p: &SavePlanet) -> Planet {
    let mut stockpile = ResourceMap::new();
    for (r, v) in &p.stockpile {
        if let Some(h) = c.resources.id(r) {
            stockpile.insert(crate::game::defs::ResourceId(h), *v);
        }
    }
    Planet {
        id: PlanetId(p.id),
        name: p.name.clone(),
        tier: p.tier,
        biome: p.biome,
        distance: p.distance,
        unlocked: p.unlocked,
        unlock_req: load_req(c, &p.unlock_req),
        nodes: p
            .nodes
            .iter()
            .filter_map(|n| {
                c.resources.id(&n.resource).map(|h| ResourceNode {
                    id: NodeId(n.id),
                    resource: crate::game::defs::ResourceId(h),
                    richness: n.richness,
                    level: n.level,
                })
            })
            .collect(),
        factory_slots: p
            .factory_slots
            .iter()
            .map(|s| s.as_ref().and_then(|f| load_factory(c, f)))
            .collect(),
        stockpile,
        stockpile_cap: p.stockpile_cap,
    }
}

/// `SaveData` → `GameState`. Id konten yang hilang di-skip; `merchant`/`events` default.
pub fn from_save(save: &SaveData, c: &Content) -> GameState {
    let mut inventory = HashMap::new();
    for (i, n) in &save.inventory {
        if let Some(h) = c.items.id(i) {
            inventory.insert(crate::game::defs::ItemId(h), *n);
        }
    }
    let mut blueprints = std::collections::HashSet::new();
    for r in &save.prestige.blueprints {
        if let Some(h) = c.recipes.id(r) {
            blueprints.insert(crate::game::defs::RecipeId(h));
        }
    }
    let auto_sell = save
        .settings
        .auto_sell
        .iter()
        .filter_map(|r| {
            c.resources.id(&r.resource).map(|h| AutoSellRule {
                resource: crate::game::defs::ResourceId(h),
                keep_above: r.keep_above,
                enabled: r.enabled,
            })
        })
        .collect();
    GameState {
        version: save.version,
        last_saved_unix: save.last_saved_unix,
        tick: save.tick,
        credits: save.credits,
        inventory,
        galaxies: save
            .galaxies
            .iter()
            .map(|g| Galaxy {
                id: GalaxyId(g.id),
                name: g.name.clone(),
                level: g.level,
                kind: g.kind.clone(),
                planets: g.planets.iter().map(|p| load_planet(c, p)).collect(),
            })
            .collect(),
        active_galaxy: GalaxyId(save.active_galaxy),
        anchor_galaxy: GalaxyId(save.anchor_galaxy),
        ship: save.ship,
        research: save.research.clone(),
        prestige: PrestigeState {
            warp_cores: save.prestige.warp_cores,
            permanent_upgrades: save.prestige.permanent_upgrades.clone(),
            blueprints,
            galaxy_level_reached: save.prestige.galaxy_level_reached,
            anchor: save.prestige.anchor,
        },
        merchant: Default::default(),
        events: Default::default(),
        settings: Settings {
            auto_sell,
            theme: save.settings.theme,
        },
        tutorial_step: save.tutorial_step,
        quests: save.quests.clone(),
    }
}
