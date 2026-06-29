# 11 — Katalog Item

> Item = entri `non-resource` yang disimpan di `GameState.inventory: HashMap<ItemId, u32>`
> ([`01-data-model.md`](../design/01-data-model.md)). Skema = `ItemDef`. Implementasi: `data/items.ron`.
> `kind`: `ShipPart | Artifact | Consumable | DataCore | Blueprint`.

## Skema & Efek

```ron
// data/items.ron (Vec<ItemDef>)
(
    id: "engine_mk3", name: "Ion Engine Mk III", kind: ShipPart,
    effect: SetShipPart(part: Engine, level: 3),
    base_price: 5000.0,
),
```

```rust
pub enum ItemEffect {
    SetShipPart { part: ShipPart, level: u32 },  // pasang → set level part (consume item)
    ProductionMult { mult: f64, scope: Scope },  // buff produksi (permanen utk artifact)
    ExtraSlot { n: u8 },                         // +slot factory
    InstantResources(Vec<(ResourceId, f64)>),    // consumable: langsung tambah resource
    TempBuff { mult: f64, secs: f64 },           // consumable: buff sementara
    DataGain { amount: f64 },                     // data core → Data riset instan
    UnlockRecipe { recipe: RecipeId },            // blueprint item → masuk prestige.blueprints
}
pub enum ShipPart { Engine, Cargo, Scanner, Shield, Cloaking }
pub enum Scope { Global, ActiveGalaxy, Planet }
```

`ShipPart` item bersifat **consume-on-install**: memasang Mk-N men-set `Ship.<part> = N` lalu item
hilang dari inventory. Dibuat via crafting ([`12-crafting.md`](12-crafting.md)) atau dibeli.

---

## Ship Parts (`kind: ShipPart`)

Tiap part punya Mk I–V (5 tingkat). Mk lebih tinggi butuh resource lebih langka (lihat recipe `12`).

### Engine — ↓ travel time
| id | name | set level | base_price |
|----|------|-----------|------------|
| `engine_mk1` | Chemical Thruster | 1 | 200 |
| `engine_mk2` | Plasma Drive | 2 | 1,200 |
| `engine_mk3` | Ion Engine Mk III | 3 | 5,000 |
| `engine_mk4` | Fusion Drive | 4 | 20,000 |
| `engine_mk5` | Warp Nacelle | 5 | 80,000 |

### Cargo — ↑ stockpile cap
| id | name | set level | base_price |
|----|------|-----------|------------|
| `cargo_mk1` | Cargo Bay I | 1 | 200 |
| `cargo_mk2` | Cargo Bay II | 2 | 1,200 |
| `cargo_mk3` | Expanded Hold | 3 | 5,000 |
| `cargo_mk4` | Bulk Freighter Hold | 4 | 20,000 |
| `cargo_mk5` | Dimensional Storage | 5 | 80,000 |

### Scanner — ↑ chance & kualitas event
| id | name | set level | base_price |
|----|------|-----------|------------|
| `scanner_mk1` | Basic Sensor | 1 | 250 |
| `scanner_mk2` | Deep Scanner | 2 | 1,500 |
| `scanner_mk3` | Quantum Scanner | 3 | 6,000 |
| `scanner_mk4` | Tachyon Array | 4 | 24,000 |
| `scanner_mk5` | Omni Sensor | 5 | 96,000 |

### Shield — bertahan dari hazard
| id | name | set level | base_price |
|----|------|-----------|------------|
| `shield_mk1` | Hull Plating | 1 | 300 |
| `shield_mk2` | Deflector Shield | 2 | 1,800 |
| `shield_mk3` | Gravity Plating | 3 | 7,000 |
| `shield_mk4` | Graviton Shield | 4 | 28,000 |
| `shield_mk5` | Singularity Shield | 5 | 110,000 |

### Cloaking — buka opsi "jarah" / hindari hazard
| id | name | set level | base_price |
|----|------|-----------|------------|
| `cloak_mk1` | Stealth Coating | 1 | 500 |
| `cloak_mk2` | Cloaking Field | 2 | 3,000 |
| `cloak_mk3` | Phase Cloak | 3 | 12,000 |
| `cloak_mk4` | Void Cloak | 4 | 48,000 |

---

## Artifacts (`kind: Artifact`, efek permanen, dari event/salvage)

| id | name | effect | base_price |
|----|------|--------|------------|
| `artifact_void_lens` | Void Lens | ProductionMult(+10% Global) | 50,000 |
| `artifact_star_compass` | Star Compass | ScannerBonus permanen +1 | 40,000 |
| `artifact_ancient_core` | Ancient Reactor Core | Energy upkeep −20% Global | 60,000 |
| `artifact_warp_sigil` | Warp Sigil | Travel time −15% | 55,000 |
| `artifact_gaia_seed` | Gaia Seed | +1 slot semua planet Terran | 70,000 |
| `artifact_quantum_eye` | Quantum Eye | Event loot quality +25% | 45,000 |
| `artifact_forge_relic` | Forge Relic | Refinery speed +20% | 50,000 |
| `artifact_dyson_fragment` | Dyson Fragment | Solar Collector output ×2 | 90,000 |
| `artifact_chronos_shard` | Chronos Shard | Offline efficiency +10% | 65,000 |
| `artifact_nebula_heart` | Nebula Heart | Rare resource yield +15% | 80,000 |
| `artifact_singularity_pearl` | Singularity Pearl | Warp Core gain +10% | 120,000 |
| `artifact_alien_codex` | Alien Codex | Research speed +25% | 75,000 |

> Artifact tidak di-consume; efeknya aktif selama dimiliki (terapkan sebagai modifier saat
> menghitung produksi/event — lihat fase tick di [`07-architecture.md`](../design/07-architecture.md)).

---

## Consumables (`kind: Consumable`, sekali pakai)

| id | name | effect | base_price |
|----|------|--------|------------|
| `boost_overdrive` | Overdrive Chip | TempBuff(×2 produksi, 600s) | 5,000 |
| `boost_warp_fuel` | Warp Fuel | TempBuff(travel −50%, 1800s) | 4,000 |
| `boost_data_surge` | Data Surge | DataGain(+50,000) | 6,000 |
| `cache_iron` | Iron Cache | InstantResources(iron +10,000) | 800 |
| `cache_alloy` | Alloy Cache | InstantResources(alloys +2,000) | 3,000 |
| `cache_rare` | Rare Cache | InstantResources(rare_crystals +200) | 8,000 |
| `repair_kit` | Repair Kit | Pulihkan durabilitas ship penuh | 2,000 |
| `scan_pulse` | Scan Pulse | Reveal loot event terdekat | 1,000 |
| `emergency_warp` | Emergency Warp | Batalkan hazard travel aktif | 3,500 |
| `credit_voucher` | Credit Voucher | InstantResources(credits +100,000) | — (loot) |

---

## Data Cores (`kind: DataCore`, → Data riset)

| id | name | effect | base_price |
|----|------|--------|------------|
| `datacore_small` | Fragmented Data Core | DataGain(+5,000) | 600 |
| `datacore_medium` | Encrypted Data Core | DataGain(+25,000) | 2,500 |
| `datacore_large` | Quantum Data Core | DataGain(+100,000) | 9,000 |
| `datacore_alien` | Alien Data Vault | DataGain(+500,000) | 40,000 |

---

## Blueprints sebagai Item (`kind: Blueprint`)

Item blueprint = unlock recipe saat dipakai → masuk `PrestigeState.blueprints`. (Alternatif beli
langsung di Void Merchant, [`04-prestige.md`](../design/04-prestige.md).)

| id | name | unlock recipe | base_price |
|----|------|---------------|------------|
| `bp_plasma_refinery` | Blueprint: Plasma Refinery | `plasma_refinery` | 25,000 |
| `bp_quantum_assembler` | Blueprint: Quantum Assembler | `quantum_assembler` | 60,000 |
| `bp_superconductor` | Blueprint: Superconductor | `make_superconductor` | 45,000 |
| `bp_warp_coil` | Blueprint: Warp Coil | `make_warp_coil` | 120,000 |
| `bp_nano_assembler` | Blueprint: Nano Assembler | `make_nano` | 150,000 |
| `bp_ai_core` | Blueprint: AI Core | `make_ai_core` | 200,000 |
| `bp_antimatter_trap` | Blueprint: Antimatter Trap | `make_antimatter` | 180,000 |
| `bp_void_drill` | Blueprint: Void Drill | `make_void_drill` | 250,000 |

---

## Catatan

- **Total ≈ 80 item** (23 ship part, 12 artifact, 10 consumable, 4 data core, 8 blueprint, + sisanya
  loot khusus event).
- ShipPart Mk-N dibuat di [`12-crafting.md`](12-crafting.md) atau dibeli; artifact hanya dari
  event/salvage ([`05-events.md`](../design/05-events.md)) — tidak craftable.
- Efek artifact & buff dihitung sebagai **modifier** di fase produksi tick (`07`), bukan mengubah
  base value resource/building.
