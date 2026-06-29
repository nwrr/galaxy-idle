# 01 — Data Model

> Kontrak data inti. Semua mekanik di file `02`–`05` harus bisa dipetakan ke struct di sini.
> Semua struct yang ikut disimpan ke save mendapat `#[derive(Serialize, Deserialize)]`.
> Angka default ada di [`08-balancing.md`](08-balancing.md), bukan hard-coded di struct.

## Prinsip

- **State adalah pohon yang dimiliki `GameState`.** Tidak ada referensi silang via pointer;
  relasi antar-entitas pakai **id** (newtype `u32`/`u64`) + lookup di map.
- **ProcGen tidak menyimpan planet.** Galaksi luar hanya menyimpan `seed` + status visited;
  planet di-generate on-the-fly. Lihat [`03-progression.md`](03-progression.md).
- **Resource disimpan sebagai map**, bukan field per-jenis, agar resource baru tidak mengubah struct.

## Id Types

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GalaxyId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlanetId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactoryId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);
```

## Content Registry (data-driven)

> **Keputusan:** karena konten berskala ratusan entri, resource/item/recipe/building **tidak**
> pakai enum tertutup. Mereka **id-based** dan didefinisikan di file data RON (lihat
> [`07-architecture.md`](07-architecture.md) → `content.rs`, dir `data/`). Katalog sumber kebenaran:
> [`10-resources.md`](../content/10-resources.md), [`11-items.md`](../content/11-items.md),
> [`12-crafting.md`](../content/12-crafting.md), [`13-buildings.md`](../content/13-buildings.md).

### Id (interned)

Tiap entri konten punya **id string** stabil (mis. `"iron"`, `"exotic_gas"`, `"steel_mill"`). Saat
load, string di-intern jadi handle `u32` untuk perf; registry memetakan dua arah.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecipeId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BuildingId(pub u32);
```

> **Save portability:** id di-serialize ke save sebagai **bentuk string-nya**, bukan `u32` interned
> (yang tidak stabil bila urutan data berubah). Layer `save` mengkonversi `ResourceId ⇄ String`
> lewat registry saat (de)serialisasi. Lihat [`07-architecture.md`](07-architecture.md).

### Definisi (statis, dari RON — bukan state)

```rust
#[derive(Clone, Serialize, Deserialize)]
pub enum ResourceTier { Basic, Advanced, Rare, Special }

#[derive(Clone, Serialize, Deserialize)]
pub enum ResourceCategory { Ore, Ingot, Gas, Liquid, Crystal, Component, Exotic, Data, Currency }

pub struct ResourceDef {
    pub id: String,                  // "iron" (di-intern → ResourceId)
    pub name: String,                // "Iron"
    pub tier: ResourceTier,
    pub category: ResourceCategory,
    pub base_price: f64,             // Credits/unit (market)
    pub stackable: bool,             // false utk item unik (ship part/artifact)
    pub desc: String,
}

/// Registry global, di-load sekali saat startup, read-only selama runtime.
pub struct Content {
    pub resources: Registry<ResourceId, ResourceDef>,
    pub items:     Registry<ItemId, ItemDef>,        // 11-items.md
    pub recipes:   Registry<RecipeId, RecipeDef>,    // 12-crafting.md
    pub buildings: Registry<BuildingId, BuildingDef>,// 13-buildings.md
}

/// Map id→def + tabel intern string↔id. (lihat content.rs di 07)
pub struct Registry<I, D> { /* by_id: Vec<D>, by_name: HashMap<String, I> */ }
```

`ResourceDef`/`ItemDef`/`RecipeDef`/`BuildingDef` field-by-field = skema di katalog `10`–`13`.
`RecipeDef` didefinisikan di bawah; `ItemDef` & `BuildingDef` ringkas:

```rust
pub struct ItemDef {
    pub id: String,                  // "engine_mk3", "artifact_void_lens"
    pub name: String,
    pub kind: ItemKind,              // ShipPart / Artifact / Consumable / DataCore / Blueprint
    pub effect: ItemEffect,          // efek saat dipakai/dipasang (lihat 11-items.md)
    pub base_price: f64,
}
pub enum ItemKind { ShipPart, Artifact, Consumable, DataCore, Blueprint }

pub struct BuildingDef {
    pub id: String,                  // "iron_mine", "steel_mill"
    pub name: String,
    pub min_planet_tier: u8,         // syarat tier planet
    pub slot_cost: u8,               // berapa factory slot dipakai
    pub kind: FactoryKind,           // Extractor / Refinery / ResearchLab (template)
    pub base_cost: Vec<(ResourceId, f64)>, // biaya bangun (level 1)
    pub upkeep: Vec<(ResourceId, f64)>,    // /detik (mis. Energy/Credits)
    pub recipes: Vec<RecipeId>,      // recipe yang bisa dijalankan (utk Refinery)
}
```

`ItemEffect` (buff produksi, +slot, +scanner, dll) didefinisikan di [`11-items.md`](../content/11-items.md).

### State runtime

```rust
/// Inventory: jumlah per resource. Hanya menyimpan id+jumlah; definisi ada di Content.
pub type ResourceMap = std::collections::HashMap<ResourceId, f64>;
```

`Credits`, `Data`, dan `WarpCore` **bukan** bagian `ResourceMap` — currency, disimpan sebagai field
terpisah di `GameState`/sub-state (perilakunya beda dari resource fisik).

## RecipeDef (statis, dari `12-crafting.md`)

```rust
pub struct RecipeDef {
    pub id: String,                  // "steel_mill"
    pub inputs: Vec<(ResourceId, f64)>,  // per-craft
    pub outputs: Vec<(ResourceId, f64)>, // output resource
    pub item_outputs: Vec<(ItemId, u32)>,// output item (mis. ship part), umumnya kosong
    pub craft_time_secs: f64,        // 0.0 = instan per tick
    pub building_id: BuildingId,     // building yang menjalankan recipe (13-buildings.md)
    pub requires_blueprint: bool,    // true → harus dimiliki dulu (PrestigeState.blueprints)
    pub tier: ResourceTier,
}
```

## Factory

Satu enum membedakan Extractor vs Refinery; perilaku tick beda (lihat [`02-economy.md`](02-economy.md)).
`Factory` = **instance** runtime; parameter dasarnya dari `BuildingDef` (`13-buildings.md`).

```rust
pub enum FactoryKind {
    Extractor { node: NodeId },      // menambang dari ResourceNode
    Refinery  { recipe: RecipeId },  // recipe aktif yang dijalankan
    ResearchLab,                     // menghasilkan Data, bukan resource fisik
}

pub struct Factory {
    pub id: FactoryId,
    pub kind: FactoryKind,
    pub level: u32,                  // 0 = empty slot
    pub enabled: bool,               // bisa di-pause player utk hemat upkeep
}
```

## ResourceNode

```rust
pub struct ResourceNode {
    pub id: NodeId,
    pub resource: ResourceId,        // raw yang dihasilkan
    pub richness: f64,               // multiplier yield (dari biome/ProcGen)
    pub level: u32,                  // upgrade menaikkan base output
}
```

## Planet

```rust
pub struct Planet {
    pub id: PlanetId,
    pub name: String,
    pub tier: u8,                    // 1..=3, gating unlock building
    pub biome: Biome,                // menentukan node yang tersedia
    pub unlocked: bool,
    pub unlock_req: UnlockReq,       // syarat buka (warp tier / resource)
    pub nodes: Vec<ResourceNode>,
    pub factory_slots: Vec<Option<Factory>>, // panjang = kapasitas slot planet
    pub stockpile: ResourceMap,      // buffer lokal sebelum dikirim/dijual
    pub stockpile_cap: f64,          // dipengaruhi Cargo ship (lihat Ship)
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Biome { IronWorld, OceanPlanet, GasGiant, DeadWorld, CrystalWorld, Terran, AsteroidBelt, IceWorld, LavaWorld }

pub enum UnlockReq {
    None,
    WarpTier(u8),
    Resource(ResourceId, f64),
    All(Vec<UnlockReq>),
}
```

## Galaxy

```rust
pub struct Galaxy {
    pub id: GalaxyId,
    pub name: String,
    pub level: u8,                   // 0 = Milky Way (anchor)
    pub kind: GalaxyKind,
    pub planets: Vec<Planet>,        // utk ProcGen: hanya planet yang sudah di-"materialize"
}

pub enum GalaxyKind {
    Fixed,                           // Milky Way: handcrafted
    Procedural { seed: u64, visited: std::collections::HashSet<u32> }, // index planet yg sudah dikunjungi
}
```

Planet ProcGen di-generate dari `(seed, planet_index)` lewat RNG deterministik
([`07-architecture.md`](07-architecture.md) → `rng.rs`). Hanya planet yang sudah dikunjungi/aktif
yang dimaterialisasi ke `planets`; sisanya dihitung ulang saat dibutuhkan.

## Ship

Ship **bukan unit combat**. Fungsinya: (a) gatekeeper unlock planet/galaksi via `warp_tier`,
(b) pemberi buff pasif via part. Lihat [`03-progression.md`](03-progression.md).

```rust
pub struct Ship {
    pub warp_tier: u8,               // level utama → akses galaksi
    pub engine: u32,                 // ↓ travel time / ↑ kecepatan unlock
    pub cargo: u32,                  // ↑ stockpile_cap planet luar
    pub scanner: u32,                // ↑ chance & kualitas event saat travel
    pub shield: u32,                 // bertahan dari hazard (Black Hole, dll) — lihat 05
    pub cloaking: u32,               // buka opsi "jarah" pada encounter — lihat 05
    pub status: ShipStatus,
}

pub enum ShipStatus {
    Idle,
    Traveling { to: PlanetId, total_secs: f64, elapsed_secs: f64 },
}
```

## ResearchState

```rust
pub struct ResearchState {
    pub data: f64,                          // Data terkumpul (currency riset)
    pub completed: std::collections::HashSet<&'static str>, // tech node ids
    pub active: Option<ActiveResearch>,
}

pub struct ActiveResearch {
    pub tech_id: &'static str,
    pub data_invested: f64,
    pub elapsed_secs: f64,
}
```

Definisi tech tree (statis, bukan state) ada di tabel [`03-progression.md`](03-progression.md).

## PrestigeState (tidak ikut reset saat warp)

```rust
pub struct PrestigeState {
    pub warp_cores: u64,
    pub permanent_upgrades: std::collections::HashMap<&'static str, u32>, // id → level
    pub blueprints: std::collections::HashSet<RecipeId>,                  // recipe yg sudah dimiliki
    pub galaxy_level_reached: u8,                                         // progress tertinggi
    pub anchor: AnchorState,                                              // Milky Way persistent
}

pub struct AnchorState {
    pub passive_upgrade_level: u32,  // dibeli pakai Warp Core, ↑ passive income ke galaksi aktif
}
```

## Merchant & Event

```rust
pub struct Merchant {
    pub active: bool,
    pub expires_at_tick: u64,        // roaming: hilang setelah window
    pub stock: Vec<MerchantOffer>,
    pub restock_at_tick: u64,
}

pub enum MerchantOffer {
    Blueprint { recipe: RecipeId, cost_cores: u64, owned: bool },
    BuyResource { resource: ResourceId, amount: f64, cost_cores: u64 },
    SellResource { resource: ResourceId, amount: f64, gain_credits: f64 },
}

pub struct EventQueue {
    pub pending: std::collections::VecDeque<GameEvent>,
}
```

Definisi `GameEvent` lengkap ada di [`05-events.md`](05-events.md).

## GameState (root)

```rust
pub struct GameState {
    pub version: u32,                // schema version utk migrasi save
    pub last_saved_unix: u64,        // basis offline progress
    pub tick: u64,                   // total tick sejak new game

    // Currencies
    pub credits: f64,
    // (data ada di research; warp_cores ada di prestige)

    // Inventory item global (ship part belum terpasang, artifact, consumable, data core)
    pub inventory: std::collections::HashMap<ItemId, u32>,

    // Dunia
    pub galaxies: Vec<Galaxy>,
    pub active_galaxy: GalaxyId,     // galaksi tempat player sedang bermain (yg akan di-reset)
    pub anchor_galaxy: GalaxyId,     // selalu Milky Way (Lvl 0)

    pub ship: Ship,
    pub research: ResearchState,
    pub prestige: PrestigeState,     // PERSISTENT lintas warp jump
    pub merchant: Merchant,
    pub events: EventQueue,

    pub settings: Settings,          // auto-sell rules, ui prefs
}

pub struct Settings {
    pub auto_sell: Vec<AutoSellRule>,
    pub theme: ThemeChoice,          // lihat 06-ui.md
}

pub struct AutoSellRule {
    pub resource: ResourceId,
    pub keep_above: f64,             // jual kelebihan di atas ambang ini
    pub enabled: bool,
}
```

## Cross-check (wajib saat review)

- `Ship.warp_tier` mendukung `UnlockReq::WarpTier` di `Planet.unlock_req` → gatekeeping (pilar 2).
- `Ship.cargo` mempengaruhi `Planet.stockpile_cap` → buff pasif (pilar 2).
- `Ship.scanner` dipakai rumus trigger event di [`05-events.md`](05-events.md).
- `PrestigeState` & `AnchorState` di luar `active_galaxy` → tidak ikut reset saat warp (pilar 3).
- `Content` registry (`Resource/Item/Recipe/Building Def`) menampung ratusan entri dari katalog
  `10`–`13` tanpa mengubah struct state; menambah konten = menambah baris data RON, bukan recompile.
- Semua `ResourceId`/`RecipeId`/`BuildingId`/`ItemId` yang dirujuk state harus ada di registry
  (di-validasi `content.rs` saat load — lihat [`07-architecture.md`](07-architecture.md)).
