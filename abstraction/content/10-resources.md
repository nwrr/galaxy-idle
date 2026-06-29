# 10 — Katalog Resource

> **Sumber kebenaran semua `ResourceId`.** Tiap id di file `09`, `11`, `12`, `13` harus terdaftar
> di sini. Skema = `ResourceDef` di [`01-data-model.md`](../design/01-data-model.md). Implementasi: file
> `data/resources.ron`. `base_price` = Credits/unit (lihat market di [`02-economy.md`](../design/02-economy.md)).

## Skema (`ResourceDef`)

```ron
// data/resources.ron  (Vec<ResourceDef>)
(
    id: "iron", name: "Iron", tier: Basic, category: Ore,
    base_price: 1.0, stackable: true,
    desc: "Logam dasar paling umum; bahan baku Steel.",
),
```

`tier`: `Basic | Advanced | Rare | Special`
`category`: `Ore | Ingot | Gas | Liquid | Crystal | Component | Exotic | Data | Currency`

---

## Tier 1 — Basic

### Ores (raw, kategori `Ore`)
| id | name | price | sumber (biome) | desc |
|----|------|-------|----------------|------|
| `iron` | Iron | 1 | IronWorld, DeadWorld, Terran | Logam dasar, bahan Steel |
| `copper` | Copper | 1 | IronWorld, Terran | Konduktor; bahan kabel/elektronik |
| `aluminum` | Aluminum | 1.5 | IronWorld, DeadWorld | Logam ringan, bahan Alloy |
| `nickel` | Nickel | 2 | IronWorld, AsteroidBelt | Bahan baja tahan karat |
| `silicon_ore` | Silicon Ore | 2 | OceanPlanet, DeadWorld | Bahan wafer Silicon |
| `carbon` | Carbon | 1 | Terran, GasGiant | Bahan Steel, polimer |
| `sulfur` | Sulfur | 1 | DeadWorld, GasGiant | Bahan kimia/asam |
| `phosphorus` | Phosphorus | 1.5 | Terran | Bahan agrikultur/kimia |
| `magnesium` | Magnesium | 2 | DeadWorld | Logam ringan reaktif |
| `cobalt` | Cobalt | 3 | AsteroidBelt | Bahan magnet & superalloy |
| `manganese` | Manganese | 2 | IronWorld | Aditif baja |
| `chromium` | Chromium | 3 | IronWorld | Lapisan anti-korosi |
| `tin` | Tin | 2 | Terran | Solder, bahan perunggu |
| `zinc` | Zinc | 2 | DeadWorld | Galvanisasi |
| `lead` | Lead | 1.5 | DeadWorld | Perisai radiasi dasar |

### Gases (kategori `Gas`)
| id | name | price | sumber | desc |
|----|------|-------|--------|------|
| `hydrogen` | Hydrogen | 1 | GasGiant, OceanPlanet | Bahan bakar fusi dasar |
| `oxygen` | Oxygen | 1 | OceanPlanet, Terran | Oksidator, life support |
| `nitrogen` | Nitrogen | 1 | Terran, GasGiant | Inert, pendingin |
| `helium` | Helium | 2 | GasGiant | Pendingin, lifting gas |
| `helium3` | Helium-3 | 8 | GasGiant (kaya) | Bahan bakar fusi efisien |
| `methane` | Methane | 1.5 | GasGiant, OceanPlanet | Bahan bakar, sumber Carbon |
| `ammonia` | Ammonia | 1.5 | GasGiant | Bahan kimia/pendingin |
| `co2` | Carbon Dioxide | 1 | Terran | Bahan agrikultur tertutup |
| `argon` | Argon | 2 | Terran | Gas inert industri |
| `neon` | Neon | 3 | GasGiant | Pencahayaan, laser |

### Liquids (kategori `Liquid`)
| id | name | price | sumber | desc |
|----|------|-------|--------|------|
| `water` | Water | 1 | OceanPlanet, Terran | Pendingin, life support, elektrolisis |
| `brine` | Brine | 1.5 | OceanPlanet | Sumber garam/mineral terlarut |
| `liquid_methane` | Liquid Methane | 2 | OceanPlanet (dingin) | Bahan bakar kriogenik |
| `crude_oil` | Crude Oil | 2 | Terran | Bahan polimer/plastik |

### Energy & Data
| id | name | tier | category | price | sumber | desc |
|----|------|------|----------|-------|--------|------|
| `energy` | Energy | Basic | Currency | 2 | Generator/Solar | Upkeep factory, biaya aksi |
| `solar_flux` | Solar Flux | Basic | Gas | 1 | dekat Sun | Energy mentah dari panel surya |
| `data` | Data | Advanced | Data | — | ResearchLab | Mata uang riset (lihat 03) |

---

## Tier 2 — Advanced

### Ingots / Refined (kategori `Ingot`, crafted)
| id | name | price | recipe | desc |
|----|------|-------|--------|------|
| `iron_ingot` | Iron Ingot | 3 | `smelt_iron` | Iron dimurnikan |
| `steel` | Steel | 5 | `steel_mill` | Iron + Carbon |
| `stainless_steel` | Stainless Steel | 8 | `stainless_mill` | Steel + Chromium + Nickel |
| `copper_ingot` | Copper Ingot | 3 | `smelt_copper` | Konduktor murni |
| `aluminum_ingot` | Aluminum Ingot | 4 | `smelt_aluminum` | Logam ringan murni |
| `titanium` | Titanium | 8 | `smelt_titanium` | Logam kuat-ringan |
| `bronze` | Bronze | 4 | `alloy_bronze` | Copper + Tin |
| `brass` | Brass | 4 | `alloy_brass` | Copper + Zinc |
| `alloys` | Alloys | 8 | `alloy_forge` | Steel + Titanium (struktur kapal) |
| `superalloy` | Superalloy | 15 | `superalloy_forge` | Alloys + Cobalt + Chromium |

### Components (kategori `Component`, crafted)
| id | name | price | recipe | desc |
|----|------|-------|--------|------|
| `silicon` | Silicon Wafer | 5 | `refine_silicon` | Dari Silicon Ore |
| `glass` | Glass | 3 | `make_glass` | Silica → kaca |
| `wire` | Copper Wire | 4 | `draw_wire` | Copper Ingot → kabel |
| `circuit` | Circuit Board | 10 | `make_circuit` | Silicon + Wire |
| `microchip` | Microchip | 18 | `make_microchip` | Silicon + Circuit |
| `polymer` | Polymer | 4 | `make_polymer` | Crude Oil → plastik |
| `composite` | Composite Panel | 12 | `make_composite` | Polymer + Aluminum |
| `ceramic` | Ceramic | 6 | `make_ceramic` | Silica + heat; tahan panas |
| `battery` | Battery | 14 | `make_battery` | Lithium + Wire |
| `capacitor` | Capacitor | 12 | `make_capacitor` | Ceramic + Wire |
| `magnet` | Magnet | 10 | `make_magnet` | Cobalt + Iron |
| `motor` | Electric Motor | 22 | `make_motor` | Magnet + Wire + Steel |
| `sensor` | Sensor Module | 25 | `make_sensor` | Microchip + Glass |
| `fuel_cell` | Fuel Cell | 20 | `make_fuel_cell` | Hydrogen + Battery |
| `consumer_goods` | Consumer Goods | 8 | `goods_fab` | Polimer + Silicon (upkeep pop) |

### Mineral / Crystal T2 (kategori `Crystal`)
| id | name | price | sumber | desc |
|----|------|-------|--------|------|
| `lithium` | Lithium | 6 | DeadWorld, brine | Bahan baterai |
| `quartz` | Quartz | 4 | CrystalWorld | Optik, osilator |
| `rare_earth` | Rare Earth Elements | 12 | CrystalWorld | Magnet kuat, elektronik |
| `uranium` | Uranium | 20 | DeadWorld (kaya) | Bahan reaktor fisi |
| `thorium` | Thorium | 18 | DeadWorld | Bahan bakar reaktor alternatif |

---

## Tier 3 — Rare

### Gas & Exotic (kategori `Gas`/`Exotic`)
| id | name | price | sumber | desc |
|----|------|-------|--------|------|
| `exotic_gas` | Exotic Gas | 25 | GasGiant (jauh) | Bahan Plasma & refinery lanjut |
| `xenon` | Xenon | 22 | GasGiant | Propelan ion-drive |
| `tritium` | Tritium | 30 | refinery/fusi | Bahan bakar fusi tinggi |
| `deuterium` | Deuterium | 18 | OceanPlanet | Bahan bakar fusi |
| `plasma` | Plasma | 40 | `plasma_refinery` (blueprint) | Bahan komponen tinggi |

### Crystal & Rare (kategori `Crystal`/`Exotic`)
| id | name | price | sumber | desc |
|----|------|-------|--------|------|
| `rare_crystals` | Rare Crystals | 50 | CrystalWorld (jauh) | Optik kuantum, riset |
| `dilithium` | Dilithium | 60 | CrystalWorld (jauh) | Stabilisator warp |
| `iridium` | Iridium | 45 | AsteroidBelt (kaya) | Katalis & elektroda |
| `platinum` | Platinum | 40 | AsteroidBelt | Katalis presisi |
| `palladium` | Palladium | 38 | AsteroidBelt | Penyimpan hidrogen |
| `gold` | Gold | 35 | AsteroidBelt | Konduktor presisi |

### Komponen tingkat tinggi (kategori `Component`)
| id | name | price | recipe | desc |
|----|------|-------|--------|------|
| `quantum_chip` | Quantum Chip | 80 | `quantum_assembler` (blueprint) | Silicon + Plasma |
| `superconductor` | Superconductor | 70 | `make_superconductor` | Rare Earth + Helium-3 |
| `warp_coil` | Warp Coil | 120 | `make_warp_coil` | Dilithium + Superconductor |
| `nano_assembler` | Nano Assembler | 150 | `make_nano` | Quantum Chip + Composite |
| `ai_core` | AI Core | 200 | `make_ai_core` | Quantum Chip + Sensor + Data |

---

## Tier Special / Prestige (kategori `Exotic`/`Currency`)

| id | name | price | sumber | desc |
|----|------|-------|--------|------|
| `antimatter` | Antimatter | 50 | Galaksi Lvl ≥2 / refinery | Mata uang Merchant, bahan endgame |
| `dark_matter` | Dark Matter | 100 | Galaksi jauh | Bahan upgrade prestige |
| `dark_energy` | Dark Energy | 120 | event/exotic | Bahan ascension |
| `singularity_matter` | Singularity Matter | 500 | event Black Hole | Ultra-rare, upgrade prestige |
| `exotic_matter` | Exotic Matter | 150 | Galaksi jauh | Stabilkan wormhole |
| `void_essence` | Void Essence | 200 | Void Merchant / anomali | Blueprint eksklusif |
| `warp_core` | Warp Core | — | prestige (Warp Jump) | Mata uang prestige (lihat 04) |
| `stellar_core` | Stellar Core | — | bintang collapse (endgame) | Currency ascension lanjut |
| `void_drill_part` | Void Drill Component | 300 | `make_void_drill` (blueprint) | Komponen untuk membangun Void Drill (13) |

### Currency (kategori `Currency`)
| id | name | price | sumber | desc |
|----|------|-------|--------|------|
| `credits` | Credits | 1 | market (jual resource) | Soft currency; biaya bangun/upgrade dasar |
| `energy` | Energy | 2 | power plant / solar | Upkeep factory, biaya aksi (juga di Tier 1) |
| `data` | Data | — | ResearchLab | Mata uang riset (lihat 03) |

---

## Catatan

- **Total ≈ 165 resource.** Cukup mengisi early→endgame tanpa membengkakkan UI (lihat
  pengelompokan tier di panel `[ RESOURCES ]`, [`06-ui.md`](../design/06-ui.md)).
- `energy`, `data`, `warp_core`, `stellar_core` berkategori `Currency`/`Data` → tidak punya node;
  diproduksi building khusus / prestige, bukan ditambang.
- `base_price` di sini **menggantikan** tabel harga lama di `08`; `08-balancing.md` kini hanya
  menyimpan konstanta global. Lihat catatan migrasi di [`08-balancing.md`](../design/08-balancing.md).
- Resource yang ditandai "recipe" diproduksi di [`12-crafting.md`](12-crafting.md); yang "sumber
  biome" ditambang via node ([`13-buildings.md`](13-buildings.md) extractor).
