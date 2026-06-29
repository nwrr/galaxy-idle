# 12 — Katalog Crafting (Recipes)

> Skema = `RecipeDef` ([`01-data-model.md`](../design/01-data-model.md)). Implementasi: `data/recipes.ron`.
> `inputs`/`outputs` merujuk id [`10-resources.md`](10-resources.md); `building` merujuk
> [`13-buildings.md`](13-buildings.md); `item_outputs` merujuk [`11-items.md`](11-items.md).
> `blueprint=yes` → recipe baru jalan setelah ada di `PrestigeState.blueprints`.

## Skema

```ron
// data/recipes.ron (Vec<RecipeDef>)
(
    id: "steel_mill",
    inputs: [ ("iron", 2.0), ("carbon", 1.0) ],
    outputs: [ ("steel", 1.0) ],
    item_outputs: [],
    craft_time_secs: 0.0,
    building_id: "steel_mill_bld",
    requires_blueprint: false,
    tier: Advanced,
),
```

Notasi tabel: `inputs → outputs | time(s) | building | bp`.

---

## Smelting / Refining (Ingot)

| id | inputs → outputs | time | building | bp |
|----|------------------|------|----------|----|
| `smelt_iron` | 2 iron → 1 iron_ingot | 1 | `smelter` | no |
| `smelt_copper` | 2 copper → 1 copper_ingot | 1 | `smelter` | no |
| `smelt_aluminum` | 2 aluminum → 1 aluminum_ingot | 1 | `smelter` | no |
| `smelt_titanium` | 3 iron + 1 magnesium → 1 titanium | 3 | `arc_furnace` | no |
| `steel_mill` | 2 iron + 1 carbon → 1 steel | 0 | `steel_mill_bld` | no |
| `stainless_mill` | 2 steel + 1 chromium + 1 nickel → 1 stainless_steel | 2 | `steel_mill_bld` | no |
| `alloy_bronze` | 2 copper_ingot + 1 tin → 1 bronze | 1 | `alloy_forge_bld` | no |
| `alloy_brass` | 2 copper_ingot + 1 zinc → 1 brass | 1 | `alloy_forge_bld` | no |
| `alloy_forge` | 2 steel + 1 titanium → 1 alloys | 0 | `alloy_forge_bld` | no |
| `superalloy_forge` | 2 alloys + 1 cobalt + 1 chromium → 1 superalloy | 4 | `arc_furnace` | no |

## Material / Component dasar

| id | inputs → outputs | time | building | bp |
|----|------------------|------|----------|----|
| `refine_silicon` | 2 silicon_ore → 1 silicon | 1 | `refinery` | no |
| `make_glass` | 2 silicon_ore → 1 glass | 1 | `kiln` | no |
| `make_ceramic` | 1 silicon_ore + 1 aluminum → 1 ceramic | 2 | `kiln` | no |
| `draw_wire` | 1 copper_ingot → 2 wire | 0 | `assembler` | no |
| `make_polymer` | 2 crude_oil → 1 polymer | 1 | `chem_plant` | no |
| `make_composite` | 2 polymer + 1 aluminum_ingot → 1 composite | 2 | `assembler` | no |
| `make_magnet` | 1 cobalt + 1 iron_ingot → 1 magnet | 2 | `assembler` | no |
| `goods_fab` | 2 silicon + 1 water → 1 consumer_goods | 0 | `assembler` | no |

## Elektronik

| id | inputs → outputs | time | building | bp |
|----|------------------|------|----------|----|
| `make_circuit` | 1 silicon + 2 wire → 1 circuit | 2 | `electronics_lab` | no |
| `make_microchip` | 1 silicon + 1 circuit → 1 microchip | 3 | `electronics_lab` | no |
| `make_capacitor` | 1 ceramic + 2 wire → 1 capacitor | 2 | `electronics_lab` | no |
| `make_sensor` | 1 microchip + 1 glass → 1 sensor | 3 | `electronics_lab` | no |
| `make_battery` | 1 lithium + 2 wire → 1 battery | 2 | `chem_plant` | no |
| `make_motor` | 1 magnet + 2 wire + 1 steel → 1 motor | 3 | `assembler` | no |
| `make_fuel_cell` | 2 hydrogen + 1 battery → 1 fuel_cell | 3 | `chem_plant` | no |

## Energy & Gas

| id | inputs → outputs | time | building | bp |
|----|------------------|------|----------|----|
| `electrolysis` | 2 water → 2 hydrogen + 1 oxygen | 1 | `chem_plant` | no |
| `gen_energy_solar` | 1 solar_flux → 5 energy | 0 | `solar_collector` | no |
| `gen_energy_fusion` | 1 helium3 + 1 deuterium → 50 energy | 0 | `fusion_reactor` | no |
| `gen_energy_fission` | 1 uranium → 80 energy | 0 | `fission_reactor` | no |
| `refine_deuterium` | 5 water → 1 deuterium | 2 | `refinery` | no |
| `refine_tritium` | 2 deuterium → 1 tritium | 3 | `fusion_reactor` | no |

## Tier 3 / Exotic (sebagian butuh blueprint)

| id | inputs → outputs | time | building | bp |
|----|------------------|------|----------|----|
| `plasma_refinery` | 5 exotic_gas + 2 energy → 1 plasma | 2 | `plasma_refinery_bld` | **yes** |
| `quantum_assembler` | 1 silicon + 1 plasma → 1 quantum_chip | 3 | `quantum_lab` | **yes** |
| `make_superconductor` | 1 rare_earth + 1 helium3 → 1 superconductor | 4 | `quantum_lab` | **yes** |
| `make_warp_coil` | 1 dilithium + 1 superconductor → 1 warp_coil | 5 | `quantum_lab` | **yes** |
| `make_nano` | 1 quantum_chip + 1 composite → 1 nano_assembler | 5 | `nano_forge` | **yes** |
| `make_ai_core` | 1 quantum_chip + 1 sensor + 1000 data → 1 ai_core | 6 | `nano_forge` | **yes** |
| `make_antimatter` | 100 energy + 1 exotic_matter → 1 antimatter | 8 | `antimatter_trap_bld` | **yes** |
| `make_void_drill` | 1 nano_assembler + 1 dilithium → 1 void_drill_part* | 6 | `nano_forge` | **yes** |

\* `void_drill_part` = resource komponen (kategori Component) untuk membangun building Void Drill
(lihat [`13`](13-buildings.md) & [`10`](10-resources.md)).

## Ship Parts (output = item, `item_outputs`)

| id | inputs → item_outputs | time | building | bp |
|----|------------------------|------|----------|----|
| `craft_engine_mk1` | 5 steel + 2 motor → `engine_mk1` ×1 | 5 | `shipyard` | no |
| `craft_engine_mk2` | 10 alloys + 2 fuel_cell → `engine_mk2` ×1 | 10 | `shipyard` | no |
| `craft_engine_mk3` | 20 alloys + 5 plasma → `engine_mk3` ×1 | 20 | `shipyard` | no |
| `craft_engine_mk4` | 30 superalloy + 5 fuel_cell + 2 superconductor → `engine_mk4` ×1 | 40 | `shipyard` | no |
| `craft_engine_mk5` | 10 warp_coil + 5 quantum_chip → `engine_mk5` ×1 | 80 | `shipyard` | no |
| `craft_cargo_mk2` | 15 steel + 5 composite → `cargo_mk2` ×1 | 10 | `shipyard` | no |
| `craft_cargo_mk3` | 20 alloys + 10 composite → `cargo_mk3` ×1 | 20 | `shipyard` | no |
| `craft_scanner_mk2` | 5 sensor + 3 circuit → `scanner_mk2` ×1 | 10 | `shipyard` | no |
| `craft_scanner_mk3` | 5 sensor + 2 quantum_chip → `scanner_mk3` ×1 | 20 | `shipyard` | no |
| `craft_shield_mk2` | 15 stainless_steel + 5 capacitor → `shield_mk2` ×1 | 12 | `shipyard` | no |
| `craft_shield_mk3` | 20 superalloy + 5 superconductor → `shield_mk3` ×1 | 25 | `shipyard` | no |
| `craft_cloak_mk1` | 10 composite + 5 microchip → `cloak_mk1` ×1 | 15 | `shipyard` | no |

> Ship part Mk yang lebih tinggi (cargo_mk4/5, scanner_mk4/5, dst) mengikuti pola sama: input naik
> ke tier resource berikutnya. Daftar lengkap di `data/recipes.ron` (≈60+ recipe total).

---

## Rantai Craft (contoh visual)

```
iron ─smelt─▶ iron_ingot ─┐
carbon ───────────────────┴─steel_mill─▶ steel ─┐
titanium ───────────────────────────────────────┴─alloy_forge─▶ alloys ─▶ ship parts
silicon_ore ─refine─▶ silicon ─┬─make_circuit─▶ circuit ─▶ microchip ─▶ sensor
copper ─smelt─▶ copper_ingot ─draw_wire─▶ wire ─┘
exotic_gas ─plasma_refinery(bp)─▶ plasma ─quantum_assembler(bp)─▶ quantum_chip ─▶ ai_core
```

## Catatan

- **Total ≈ 60 recipe.** Rantai mendalam (ore→ingot→component→part) menciptakan "tech depth" tanpa
  perlu ratusan recipe.
- Recipe `craft_time_secs: 0` diproses penuh per tick (throughput = building level, lihat
  [`02-economy.md`](../design/02-economy.md) §3); `>0` butuh akumulasi waktu → progress bar UI.
- Recipe blueprint (`make_*` tier 3) hanya bisa dijalankan setelah blueprint dimiliki — gerbang
  konten mid-late game lewat Void Merchant ([`04-prestige.md`](../design/04-prestige.md)).
