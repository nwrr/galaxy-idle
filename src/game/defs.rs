//! Definisi konten statis (dari `data/*.ron`) + `Registry`/`Content`.
//!
//! Skema field-by-field = katalog `10`–`13` & `01-data-model.md`. Hanya `Deserialize`
//! (load); turunan save (`Serialize`) ditambahkan saat dibutuhkan (M7).
//!
//! NOTE(scaffold): `allow(dead_code)` — banyak field/akses dipakai mulai M2 (economy/sim).
#![allow(dead_code)]

use serde::Deserialize;
use std::collections::HashMap;

// ── Id newtypes (interned u32) ─────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ResourceId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ItemId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RecipeId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BuildingId(pub u32);

// ── Resource ───────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum ResourceTier {
    Basic,
    Advanced,
    Rare,
    Special,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum ResourceCategory {
    Ore,
    Ingot,
    Gas,
    Liquid,
    Crystal,
    Component,
    Exotic,
    Data,
    Currency,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ResourceDef {
    pub id: String,
    pub name: String,
    pub tier: ResourceTier,
    pub category: ResourceCategory,
    pub base_price: f64,
    pub stackable: bool,
    pub desc: String,
}

// ── Item ────────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum ItemKind {
    ShipPart,
    Artifact,
    Consumable,
    DataCore,
    Blueprint,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum ShipPart {
    Engine,
    Cargo,
    Scanner,
    Shield,
    Cloaking,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum Scope {
    Global,
    ActiveGalaxy,
    Planet,
}

#[derive(Clone, Debug, Deserialize)]
pub enum ItemEffect {
    SetShipPart {
        part: ShipPart,
        level: u32,
    },
    ProductionMult {
        mult: f64,
        scope: Scope,
    },
    ExtraSlot {
        n: u8,
    },
    InstantResources(Vec<(String, f64)>),
    TempBuff {
        mult: f64,
        secs: f64,
    },
    DataGain {
        amount: f64,
    },
    UnlockRecipe {
        recipe: String,
    },
    /// Catch-all modifier artifact/consumable (lihat header items.ron). `kind` = label efek.
    Special {
        kind: String,
        value: f64,
    },
}

#[derive(Clone, Debug, Deserialize)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub kind: ItemKind,
    pub effect: ItemEffect,
    pub base_price: f64,
}

// ── Recipe ───────────────────────────────────────────────────────────────────
#[derive(Clone, Debug, Deserialize)]
pub struct RecipeDef {
    pub id: String,
    pub inputs: Vec<(String, f64)>,
    pub outputs: Vec<(String, f64)>,
    pub item_outputs: Vec<(String, u32)>,
    pub craft_time_secs: f64,
    pub building_id: String,
    pub requires_blueprint: bool,
    pub tier: ResourceTier,
}

// ── Building ─────────────────────────────────────────────────────────────────
/// Template kind (tanpa payload); beda dari runtime `FactoryKind` yang bawa node/recipe.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum BuildingKind {
    Extractor,
    Refinery,
    ResearchLab,
    Storage,
    Special,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BuildingDef {
    pub id: String,
    pub name: String,
    pub min_planet_tier: u8,
    pub slot_cost: u8,
    pub kind: BuildingKind,
    pub base_cost: Vec<(String, f64)>,
    pub upkeep: Vec<(String, f64)>,
    pub recipes: Vec<String>,
}

// ── Tech ─────────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum TechBranch {
    Extraction,
    Manufacturing,
    Aerospace,
    AstroCartography,
}

#[derive(Clone, Debug, Deserialize)]
pub enum TechUnlock {
    Recipe(String),
    WarpTier(u8),
    UnlockBuilding(String),
    TechMult { resources: Vec<String>, add: f64 },
    AccessOuterTier(u8),
}

#[derive(Clone, Debug, Deserialize)]
pub struct TechDef {
    pub id: String,
    pub branch: TechBranch,
    pub data_cost: f64,
    pub time_secs: f64,
    pub depends_on: Vec<String>,
    pub unlock: TechUnlock,
}

// ── Quest (M6) ────────────────────────────────────────────────────────────
/// Kondisi auto-advance stage quest (pola SAMA `game::tutorial`'s step predicate, tp data-
/// driven — hook nyata ke sistem existing: riset (`TechCompleted`), travel (`ShipTraveling`),
/// prestige (`WarpJumped`). `Manual` = stage butuh pilihan PLAYER (`choose_branch`), tak
/// pernah auto-advance sendiri (dipakai stage dgn >1 branch — titik cabang nyata).
#[derive(Clone, Debug, Deserialize)]
pub enum QuestReq {
    None,
    TechCompleted(String),
    ShipTraveling,
    WarpJumped,
    Manual,
}

/// Efek nyata ke `GameState` saat stage/branch selesai — reuse field EXISTING (credits/warp
/// cores), bukan sistem reward baru.
#[derive(Clone, Debug, Deserialize)]
pub enum QuestEffect {
    GrantCredits(f64),
    GrantWarpCores(u64),
}

/// Satu opsi di stage cabang (>1 branch = titik keputusan NYATA, hasil beda per pilihan).
#[derive(Clone, Debug, Deserialize)]
pub struct QuestBranchDef {
    pub label: String,
    pub next_stage: String,
    #[serde(default)]
    pub effects: Vec<QuestEffect>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct QuestStageDef {
    pub id: String,
    pub text: String,
    pub requirement: QuestReq,
    #[serde(default)]
    pub effects: Vec<QuestEffect>,
    /// Kosong = stage terminal (quest selesai di sini). 1 = auto-chain linear ke `next_stage`
    /// (label diabaikan). >1 = titik cabang NYATA, butuh `choose_branch` (player pick).
    #[serde(default)]
    pub branches: Vec<QuestBranchDef>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct QuestDef {
    pub id: String,
    /// Grup arc naratif (mis. "pioneer"/"warp_frontier") — R6's "≥2 arc" dikelompokkan lewat
    /// field ini, bukan modul/file terpisah per arc.
    pub arc: String,
    pub title: String,
    pub stages: Vec<QuestStageDef>,
}

// ── Registry & Content ───────────────────────────────────────────────────────
/// Entri punya id string stabil → di-intern jadi handle u32.
pub trait HasId {
    fn id_str(&self) -> &str;
}

impl HasId for ResourceDef {
    fn id_str(&self) -> &str {
        &self.id
    }
}
impl HasId for ItemDef {
    fn id_str(&self) -> &str {
        &self.id
    }
}
impl HasId for RecipeDef {
    fn id_str(&self) -> &str {
        &self.id
    }
}
impl HasId for BuildingDef {
    fn id_str(&self) -> &str {
        &self.id
    }
}
impl HasId for TechDef {
    fn id_str(&self) -> &str {
        &self.id
    }
}
impl HasId for QuestDef {
    fn id_str(&self) -> &str {
        &self.id
    }
}

/// Map id-string → handle u32 + def indexed by handle. Read-only setelah build.
pub struct Registry<D> {
    defs: Vec<D>,
    by_name: HashMap<String, u32>,
}

impl<D: HasId> Registry<D> {
    /// Bangun registry; gagal bila ada id duplikat.
    pub fn build(defs: Vec<D>) -> Result<Self, String> {
        let mut by_name = HashMap::with_capacity(defs.len());
        for (i, d) in defs.iter().enumerate() {
            if by_name.insert(d.id_str().to_string(), i as u32).is_some() {
                return Err(d.id_str().to_string());
            }
        }
        Ok(Self { defs, by_name })
    }

    pub fn id(&self, name: &str) -> Option<u32> {
        self.by_name.get(name).copied()
    }
    pub fn contains(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }
    pub fn get(&self, handle: u32) -> &D {
        &self.defs[handle as usize]
    }
    pub fn len(&self) -> usize {
        self.defs.len()
    }
    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = &D> {
        self.defs.iter()
    }
}

/// Registry global konten, di-load sekali saat startup, read-only saat runtime.
pub struct Content {
    pub resources: Registry<ResourceDef>,
    pub items: Registry<ItemDef>,
    pub recipes: Registry<RecipeDef>,
    pub buildings: Registry<BuildingDef>,
    pub techs: Registry<TechDef>,
    pub quests: Registry<QuestDef>,
}
