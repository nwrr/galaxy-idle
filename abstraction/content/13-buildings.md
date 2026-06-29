# 13 — Katalog Building

> Skema = `BuildingDef` ([`01-data-model.md`](../design/01-data-model.md)). Implementasi: `data/buildings.ron`.
> `kind` = template `FactoryKind` (Extractor/Refinery/ResearchLab). `base_cost` = biaya bangun level 1
> (rumus level berikutnya: `cost*GROWTH^n`, [`02-economy.md`](../design/02-economy.md) §5). `recipes` merujuk
> [`12-crafting.md`](12-crafting.md). Semua resource = id [`10-resources.md`](10-resources.md).

## Skema

```ron
// data/buildings.ron (Vec<BuildingDef>)
(
    id: "steel_mill_bld", name: "Steel Mill",
    min_planet_tier: 1, slot_cost: 1, kind: Refinery,
    base_cost: [ ("steel", 20.0), ("credits", 200.0) ],
    upkeep: [ ("energy", 2.0) ],
    recipes: [ "steel_mill", "stainless_mill" ],
),
```

Notasi tabel: `tier` = min_planet_tier, `slot` = slot_cost, `cost` = base_cost ringkas,
`upkeep` /detik, `recipes` yang bisa dijalankan.

---

## Extractors (`kind: Extractor` — menambang node)

Extractor mengikat ke `NodeId` di planet; resource yang ditambang = `node.resource`. Tipe extractor
dibatasi kategori resource node (drill utk ore, pump utk liquid, dll).

| id | name | tier | slot | cost | upkeep | menambang |
|----|------|------|------|------|--------|-----------|
| `mining_drill` | Mining Drill | 1 | 1 | 50 cr | 1 energy | Ore (iron, copper, …) |
| `gas_extractor` | Gas Extractor | 1 | 1 | 80 cr | 2 energy | Gas (hydrogen, helium, …) |
| `liquid_pump` | Liquid Pump | 1 | 1 | 60 cr | 1 energy | Liquid (water, brine, …) |
| `crystal_harvester` | Crystal Harvester | 2 | 1 | 200 cr | 3 energy | Crystal (quartz, rare_earth, …) |
| `deep_core_miner` | Deep Core Miner | 2 | 2 | 500 cr, 50 steel | 5 energy | Ore T2/T3 (richness ×1.5) |
| `asteroid_harvester` | Asteroid Harvester | 2 | 2 | 800 cr, 50 alloys | 4 energy | AsteroidBelt (platinum, iridium, …) |
| `solar_collector` | Solar Collector | 1 | 1 | 120 cr | 0 | solar_flux → energy (recipe `gen_energy_solar`) |
| `void_drill` | Void Drill | 3 | 3 | 1 void_drill_part | 10 energy | Exotic (exotic_matter, dark_matter) |

> `void_drill` butuh item `void_drill_part` (craft via blueprint, `12`) untuk dibangun.

## Refineries / Assemblers (`kind: Refinery`)

| id | name | tier | slot | cost | upkeep | recipes |
|----|------|------|------|------|--------|---------|
| `smelter` | Smelter | 1 | 1 | 100 cr | 3 energy | smelt_iron, smelt_copper, smelt_aluminum |
| `arc_furnace` | Arc Furnace | 2 | 1 | 400 cr, 30 steel | 8 energy | smelt_titanium, superalloy_forge |
| `steel_mill_bld` | Steel Mill | 1 | 1 | 200 cr | 2 energy | steel_mill, stainless_mill |
| `alloy_forge_bld` | Alloy Forge | 2 | 1 | 500 cr, 20 steel | 5 energy | alloy_forge, alloy_bronze, alloy_brass |
| `refinery` | Refinery | 1 | 1 | 150 cr | 3 energy | refine_silicon, refine_deuterium |
| `kiln` | Kiln | 1 | 1 | 120 cr | 4 energy | make_glass, make_ceramic |
| `assembler` | Assembler | 1 | 1 | 250 cr | 3 energy | draw_wire, make_composite, make_magnet, make_motor, goods_fab |
| `chem_plant` | Chemical Plant | 2 | 1 | 400 cr | 5 energy | make_polymer, make_battery, make_fuel_cell, electrolysis |
| `electronics_lab` | Electronics Lab | 2 | 1 | 600 cr, 20 silicon | 6 energy | make_circuit, make_microchip, make_capacitor, make_sensor |
| `plasma_refinery_bld` | Plasma Refinery | 3 | 2 | 2000 cr, 50 alloys | 15 energy | plasma_refinery |
| `quantum_lab` | Quantum Lab | 3 | 2 | 5000 cr, 20 superalloy | 20 energy | quantum_assembler, make_superconductor, make_warp_coil |
| `nano_forge` | Nano Forge | 3 | 3 | 10000 cr, 10 quantum_chip | 30 energy | make_nano, make_ai_core, make_void_drill |
| `antimatter_trap_bld` | Antimatter Trap | 3 | 3 | 15000 cr, 5 ai_core | 50 energy | make_antimatter |
| `shipyard` | Shipyard | 2 | 2 | 1000 cr, 50 alloys | 8 energy | craft_engine_*, craft_cargo_*, craft_scanner_*, craft_shield_*, craft_cloak_* |

## Power Plants (`kind: Refinery`, output energy)

| id | name | tier | slot | cost | upkeep | recipes |
|----|------|------|------|------|--------|---------|
| `fusion_reactor` | Fusion Reactor | 2 | 2 | 3000 cr, 30 alloys | 0 | gen_energy_fusion, refine_tritium |
| `fission_reactor` | Fission Reactor | 2 | 2 | 1500 cr, 20 steel | 0 | gen_energy_fission |

> Power plant ber-upkeep 0 (justru menghasilkan energy); recipe-nya mengkonsumsi bahan bakar →
> output energy. Defisit energy → penalti produksi global (lihat [`02-economy.md`](../design/02-economy.md) §6).

## Research (`kind: ResearchLab`, output Data)

| id | name | tier | slot | cost | upkeep | output |
|----|------|------|------|------|--------|--------|
| `research_lab` | Research Lab | 1 | 1 | 300 cr | 5 energy | Data (BASE_DATA_RATE × level) |
| `quantum_research` | Quantum Research Center | 3 | 2 | 8000 cr, 5 ai_core | 25 energy | Data ×4 rate |

## Storage & Special

| id | name | tier | slot | cost | upkeep | efek |
|----|------|------|------|------|--------|------|
| `storage_depot` | Storage Depot | 1 | 1 | 200 cr | 0 | +`stockpile_cap` planet (lokal) |
| `warehouse` | Warehouse | 2 | 2 | 1000 cr, 20 steel | 0 | +`stockpile_cap` besar |
| `anchor_booster` | Anchor Booster | 1 | 2 | beli Warp Core | 5 energy | ↑ `anchor_feed` Milky Way (lihat 04) |
| `warp_gate` | Warp Gate | 3 | 3 | 20000 cr, 10 warp_coil | 100 energy | ↓ travel time semua planet galaksi (buff statis) |
| `trade_hub` | Trade Hub | 2 | 1 | 800 cr | 3 energy | ↑ harga jual market +15% (auto-sell lebih untung) |

---

## Aturan Penempatan

- Building hanya bisa dibangun bila `planet.tier >= min_planet_tier`.
- `slot_cost` mengurangi `factory_slots` tersedia di planet; building besar (nano_forge, warp_gate)
  makan 3 slot → keputusan tata-letak per planet.
- Extractor hanya valid jika ada `ResourceNode` cocok kategori di planet itu.
- Power plant & research wajib hadir agar ekonomi jalan (energy untuk upkeep, data untuk riset).

## Catatan

- **Total ≈ 40 building.** Memetakan 1:1 ke `building_id` yang dirujuk di [`12-crafting.md`](12-crafting.md)
  (cross-check verifikasi #2 di plan).
- `kind` menentukan perilaku tick: Extractor (node→stockpile), Refinery (recipe), ResearchLab (data).
  Storage/special menerapkan modifier, ditangani fase khusus di tick ([`07-architecture.md`](../design/07-architecture.md)).
