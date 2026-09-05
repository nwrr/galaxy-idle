//! `GameState` (root) + sub-state runtime, pemetaan 1:1 ke `01-data-model.md`.
//!
//! State adalah pohon yang dimiliki `GameState`; relasi antar-entitas pakai id + lookup,
//! bukan pointer. Resource disimpan sebagai map (resource baru tak mengubah struct).
//!
//! NOTE(scaffold): `allow(dead_code)` — field dikonsumsi mulai M2 economy/sim, M3 UI.
//! NOTE(save): turunan `Serialize/Deserialize` ditambah M7 bersama layer konversi
//! id↔string (`07` §Save ↔ Content); id interned tak stabil utk disimpan langsung.
#![allow(dead_code)]

use crate::game::defs::{BuildingId, RecipeId, ResourceId};
use crate::game::events::EventQueue;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ── Id runtime (instance, bukan konten) ─────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GalaxyId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct PlanetId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FactoryId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(pub u32);

/// Inventory resource: jumlah per id. Definisi ada di `Content` (read-only).
pub type ResourceMap = HashMap<ResourceId, f64>;

// ── Factory & Node ───────────────────────────────────────────────────────────
/// Runtime kind (bawa node/recipe aktif); beda dari template `defs::BuildingKind`.
#[derive(Clone, Copy, Debug)]
pub enum FactoryKind {
    Extractor { node: NodeId },
    Refinery { recipe: RecipeId },
    ResearchLab,
}

#[derive(Clone, Copy, Debug)]
pub struct Factory {
    pub id: FactoryId,
    /// Template building (param statis: upkeep, base_cost, slot_cost) — lookup ke `Content`.
    pub building: BuildingId,
    pub kind: FactoryKind,
    pub level: u32, // 0 = slot kosong
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct ResourceNode {
    pub id: NodeId,
    pub resource: ResourceId,
    pub richness: f64,
    pub level: u32,
}

// ── Planet & Galaxy ──────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub enum Biome {
    IronWorld,
    OceanPlanet,
    GasGiant,
    DeadWorld,
    CrystalWorld,
    Terran,
    AsteroidBelt,
    IceWorld,
    LavaWorld,
}

#[derive(Clone, Debug)]
pub enum UnlockReq {
    None,
    WarpTier(u8),
    Resource(ResourceId, f64),
    All(Vec<UnlockReq>),
}

#[derive(Clone, Debug)]
pub struct Planet {
    pub id: PlanetId,
    pub name: String,
    pub tier: u8,
    pub biome: Biome,
    /// Jarak (AU abstrak) — basis travel time (`03` §4, `09`).
    pub distance: f64,
    pub unlocked: bool,
    pub unlock_req: UnlockReq,
    pub nodes: Vec<ResourceNode>,
    pub factory_slots: Vec<Option<Factory>>,
    pub stockpile: ResourceMap,
    pub stockpile_cap: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GalaxyKind {
    Fixed,
    Procedural { seed: u64, visited: HashSet<u32> },
}

#[derive(Clone, Debug)]
pub struct Galaxy {
    pub id: GalaxyId,
    pub name: String,
    pub level: u8, // 0 = Milky Way (anchor)
    pub kind: GalaxyKind,
    pub planets: Vec<Planet>, // ProcGen: hanya yang sudah dimaterialisasi
}

// ── Ship ─────────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ShipStatus {
    Idle,
    Traveling {
        to: PlanetId,
        total_secs: f64,
        elapsed_secs: f64,
    },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Ship {
    pub warp_tier: u8,
    pub engine: u32,
    pub cargo: u32,
    pub scanner: u32,
    pub shield: u32,
    pub cloaking: u32,
    pub status: ShipStatus,
}

impl Default for Ship {
    fn default() -> Self {
        // Asumsi start: Warp Tier 1 (Earth gratis; Luna butuh Tier 1, lihat `09`). Tuning M3.
        Ship {
            warp_tier: 1,
            engine: 0,
            cargo: 0,
            scanner: 0,
            shield: 0,
            cloaking: 0,
            status: ShipStatus::Idle,
        }
    }
}

// ── Research ─────────────────────────────────────────────────────────────────
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResearchState {
    pub data: f64,
    /// Tech id (string, data-driven) yang sudah selesai. (`01` pakai &'static; tech kini dari RON.)
    pub completed: HashSet<String>,
    pub active: Option<ActiveResearch>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveResearch {
    pub tech_id: String,
    pub data_invested: f64,
    pub elapsed_secs: f64,
}

// ── Prestige ─────────────────────────────────────────────────────────────────
#[derive(Clone, Debug, Default)]
pub struct PrestigeState {
    pub warp_cores: u64,
    pub permanent_upgrades: HashMap<String, u32>,
    pub blueprints: HashSet<RecipeId>,
    pub galaxy_level_reached: u8,
    pub anchor: AnchorState,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct AnchorState {
    pub passive_upgrade_level: u32,
}

// ── Merchant ─────────────────────────────────────────────────────────────────
#[derive(Clone, Debug)]
pub enum MerchantOffer {
    Blueprint {
        recipe: RecipeId,
        cost_cores: u64,
        owned: bool,
    },
    BuyResource {
        resource: ResourceId,
        amount: f64,
        cost_cores: u64,
    },
    SellResource {
        resource: ResourceId,
        amount: f64,
        gain_credits: f64,
    },
}

#[derive(Clone, Debug, Default)]
pub struct Merchant {
    pub active: bool,
    pub expires_at_tick: u64,
    pub stock: Vec<MerchantOffer>,
    pub restock_at_tick: u64,
}

// ── Quest (M6) ────────────────────────────────────────────────────────────
/// Progres quest pemain: quest aktif→stage id sekarang, quest id selesai, histori pilihan
/// cabang (`quest_id` → `label` yg dipilih -- utk save round-trip + UI nunjukkan pilihan lama).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct QuestState {
    pub active: HashMap<String, String>,
    pub completed: std::collections::HashSet<String>,
    pub choices: HashMap<String, String>,
}

// ── Settings ─────────────────────────────────────────────────────────────────
/// Pilihan tema warna (detail di `06-ui.md`; di-resolve ke palet di `ui::theme` M3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeChoice {
    #[default]
    Default,
    HighContrast,
    Mono,
}

#[derive(Clone, Debug)]
pub struct AutoSellRule {
    pub resource: ResourceId,
    pub keep_above: f64,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Settings {
    pub auto_sell: Vec<AutoSellRule>,
    pub theme: ThemeChoice,
}

// ── GameState (root) ─────────────────────────────────────────────────────────
#[derive(Clone, Debug)]
pub struct GameState {
    pub version: u32,
    pub last_saved_unix: u64,
    pub tick: u64,

    pub credits: f64,
    pub inventory: HashMap<crate::game::defs::ItemId, u32>,

    pub galaxies: Vec<Galaxy>,
    pub active_galaxy: GalaxyId,
    pub anchor_galaxy: GalaxyId,

    pub ship: Ship,
    pub research: ResearchState,
    pub prestige: PrestigeState,
    pub merchant: Merchant,
    pub events: EventQueue,

    pub settings: Settings,
    /// M6: progres questline (mirrors `research`'s Data-driven pattern).
    pub quests: QuestState,

    /// M3: langkah onboarding aktif (`0`=bangun extractor pertama, `1`=mulai riset pertama,
    /// `2`=kirim ship travel, `3`=warp jump pertama, `None`=tutorial selesai/dilewati). Bukan
    /// dismiss manual — auto-advance saat kondisi REAL terpenuhi (`game::tutorial`). Save lama
    /// (v1, sblm field ini ada) di-migrasi ke `None` (pemain existing dianggap sudah lewat
    /// onboarding, bukan dipaksa ulang — lihat `save::migrate`).
    pub tutorial_step: Option<u8>,
}

/// Versi schema save saat ini. v1→v2 (M3): tambah `tutorial_step`. v2→v3 (M6): tambah `quests`.
pub const SAVE_VERSION: u32 = 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ship_default_idle_tier1() {
        let s = Ship::default();
        assert_eq!(s.warp_tier, 1);
        assert!(matches!(s.status, ShipStatus::Idle));
    }

    #[test]
    fn substate_defaults_empty() {
        let r = ResearchState::default();
        assert!(r.completed.is_empty() && r.active.is_none());
        let p = PrestigeState::default();
        assert_eq!(p.warp_cores, 0);
        assert!(p.blueprints.is_empty());
        assert_eq!(Settings::default().theme, ThemeChoice::Default);
    }
}
