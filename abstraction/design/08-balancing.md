# 08 — Balancing: Konstanta Tunable

> Satu tempat untuk semua **konstanta global** (rumus, growth, rate). Dikristalkan ke `src/balance.rs`
> sebagai `const`. Nilai di sini adalah **baseline awal**; di-tune saat playtest. Item `TBD` perlu
> keputusan playtest.
>
> **Migrasi data-driven:** angka **per-entri** (harga tiap resource, input/output recipe, biaya
> building) **tidak lagi** di sini — pindah ke katalog [`10`](../content/10-resources.md)–[`13`](../content/13-buildings.md)
> (file `data/*.ron`). `08` hanya menyimpan konstanta yang berlaku lintas-entri. Tabel "Harga jual"
> & "Recipes" di bawah dipertahankan sebagai **contoh/baseline** tetapi sumber kebenarannya kini ada
> di katalog.

## Tick & Loop

| Const | Nilai | Catatan |
|-------|-------|---------|
| `TICK_DURATION` | 1.0 s | satu langkah simulasi = 1 detik game-time |
| `RENDER_INTERVAL` | 75 ms | ~13 FPS untuk animasi |
| `AUTOSAVE_INTERVAL` | 30 s | + saat quit & sebelum warp |
| `MAX_TICKS_PER_FRAME` | 16 | batas catch-up; sisanya jadi offline |

## Offline Progress

| Const | Nilai | Catatan |
|-------|-------|---------|
| `OFFLINE_CAP` | 28,800 s (8 jam) | maksimum waktu offline yang dihitung |
| `OFFLINE_BASE_EFF` | 0.75 | efisiensi dasar offline (<1.0) |
| `offline_eff` upgrade | +0.05 / level | dari permanent upgrade, cap 1.0 |

## Economy — Scaling

| Const | Nilai | Catatan |
|-------|-------|---------|
| `GROWTH` | 1.15 | `cost(n) = base * GROWTH^n` untuk semua upgrade |
| `BASE_EXTRACTOR_RATE` | 1.0 /s | × level × richness × tech_mult |
| `BASE_DATA_RATE` | 5.0 /s | per level ResearchLab |

### Base cost upgrade (Credits / resource)

| Upgrade | base_cost | Mata bayar |
|---------|-----------|-----------|
| Factory level | 50 | Credits + crafted terkait |
| Node level | 100 | Credits |
| Ship Engine | 200 | Steel/Titanium |
| Ship Cargo | 200 | Steel |
| Ship Scanner | 250 | Silicon/Titanium |

### Harga jual (`price(resource)`, Credits/unit)

| Tier | Resource | Harga |
|------|----------|-------|
| Basic | Iron / Carbon / Water | 1 |
| Basic | Energy | 2 |
| Advanced | Silicon / Steel | 5 |
| Advanced | Titanium / Alloys / ConsumerGoods | 8 |
| Rare | ExoticGas / Plasma | 25 |
| Rare | RareCrystals / DarkMatter / Antimatter | 50 |
| Special | SingularityMatter | 500 |

## Recipes (final MVP)

| recipe_id | inputs | outputs | craft_time | blueprint |
|-----------|--------|---------|-----------|-----------|
| `steel_mill` | 2 Iron, 1 Carbon | 1 Steel | 0 | no |
| `alloy_forge` | 2 Steel, 1 Titanium | 1 Alloys | 0 | no |
| `goods_fab` | 2 Silicon, 1 Water | 1 ConsumerGoods | 0 | no |
| `plasma_refinery` | 5 ExoticGas, 2 Energy | 1 Plasma | 2.0 | **yes** |
| `quantum_assembler` | 1 Silicon, 1 Plasma | 1 QuantumChip | 3.0 | **yes** |

## Ship / Travel

| Const | Nilai | Catatan |
|-------|-------|---------|
| `BASE_TRAVEL` | 600 s | basis travel time (×distance) |
| `ENGINE_FACTOR` | 0.90 | `travel *= ENGINE_FACTOR^engine_level` |
| `BASE_CAP` | 1,000 | stockpile cap dasar planet |
| `CARGO_PER_LVL` | 0.25 | `cap = BASE_CAP*(1+CARGO_PER_LVL*cargo)` |
| `DIST_YIELD_K` | 0.15 | `yield_mult = 1 + DIST_YIELD_K*distance` |

### Warp Tier → akses

| Warp Tier | Akses |
|-----------|-------|
| 1–5 | Milky Way |
| 6 | Outer Tier 1 |
| 10 | Outer Tier 2 |

> **ProcGen & BIOME_TABLE** dipindah ke dekat tabel sistem visual di bawah (seksi "ProcGen (lihat
> `16-procgen-logic.md`)") agar konstanta generatif & animasi berkumpul. Jangan duplikasi di sini.

## Research — Tech Tree (final MVP, dengan `depends_on`)

| tech_id | cabang | data_cost | time(s) | depends_on | unlock |
|---------|--------|-----------|---------|-----------|--------|
| `manu_steel` | Manufacturing | 500 | 120 | — | recipe `steel_mill` |
| `manu_titanium` | Manufacturing | 2,000 | 300 | `manu_steel` | recipe `alloy_forge` |
| `ext_deep_core` | Extraction | 8,000 | 1,200 | `manu_steel` | tambang planet T2 |
| `ext_efficiency` | Extraction | 4,000 | 600 | — | tech_mult +25% Iron/Carbon |
| `aero_warp_mk2` | Aerospace | 5,000 | 600 | `manu_titanium` | Warp Tier → 6 |
| `aero_warp_mk3` | Aerospace | 25,000 | 1,800 | `aero_warp_mk2` | Warp Tier → 10 |
| `astro_far_warp` | Astro-Cartography | 20,000 | 2,400 | `aero_warp_mk2` | akses Outer Tier 2 |

`tech_multiplier(resource)` default 1.0; dinaikkan tech Extraction.

## Prestige / Warp

| Const | Nilai | Catatan |
|-------|-------|---------|
| `WARP_K` | 10.0 | `cores = floor(WARP_K*sqrt(total_value/REF))` |
| `WARP_THRESHOLD_REF` | 1,000,000 | referensi normalisasi total_value |
| `PRESTIGE_MULT_PER_LEVEL` | 0.10 | `prod_mult = 1 + 0.10*level_reached` |
| `ANCHOR_BASE` | 5.0 /s | feed dasar Milky Way → galaksi aktif |
| `ANCHOR_PER_LVL` | 0.50 | `feed = ANCHOR_BASE*(1+0.50*anchor_lvl)` |

### Warp Threshold per level target

| Target Lvl | Requirement |
|-----------|-------------|
| 1 | 1,000,000 Energy + 500 Titanium |
| 2 | 5,000,000 Energy + 2,000 Titanium + 100 Antimatter |
| n (≥3) | ×5 Energy, ×4 Titanium tiap level, + rare resource baru (TBD) |

### Permanent upgrades (Warp Core)

| upgrade_id | efek/level | base_cost (Cores) | growth |
|------------|-----------|-------------------|--------|
| `prod_speed` | +10% produksi global | 5 | 1.5 |
| `extra_slot` | +1 factory slot/planet | 10 | 2.0 |
| `auto_collect` | auto-sell/collect lebih agresif | 8 | 1.8 |
| `offline_eff` | +5% efisiensi offline | 6 | 1.6 |

## Events

| Const | Nilai | Catatan |
|-------|-------|---------|
| `BASE_EVENT_CHANCE` | 0.05 | per roll |
| `DIST_FACTOR` | 0.10 | × travel_distance |
| `SCANNER_FACTOR` | 0.20 | × scanner level |
| `EVENT_ROLL_INTERVAL` | 30 ticks | roll event tiap 30s travel |
| Nebula travel penalty | +20% | `AdjustTravelTime` |
| Outpost buff | +50% prod, 3,600 s | `TempProductionBuff` |
| Wreck salvage time | 600 s | pause travel |

## Merchant

| Const | Nilai | Catatan |
|-------|-------|---------|
| Roaming window | 300 s | merchant `active` setelah muncul |
| Restock interval | 7,200 s (2 jam) | rotasi stock |

## ProcGen (lihat `16-procgen-logic.md`)

| Const | Nilai | Catatan |
|-------|-------|---------|
| `PLANET_MIN` / `PLANET_MAX` | 5 / 12 | planet per galaksi ProcGen |
| `NODE_MIN` / `NODE_MAX` | 2 / 5 | node per planet |
| `D1` / `D2` | 3.5 / 7.0 | ambang jarak → tier planet |
| `ANOMALY_CHANCE` | 0.15 | peluang planet punya anomali |
| `RICH_MIN` / `RICH_MAX` | 0.5 / 2.0 | rentang richness node |
| `DIST_MIN` / `DIST_MAX` | 1.0 / 10.0 | rentang jarak (LY abstrak) |
| `galaxy_tier_mult` | 1.0 + 0.5×(level−1) | richness & jarak per level |

### `BIOME_TABLE` (bobot generate ProcGen)

| Biome | Bobot | Node resource (`NODES_FOR_BIOME`) |
|-------|-------|-----------------------------------|
| IronWorld | 22 | iron, copper, nickel, carbon |
| OceanPlanet | 16 | water, brine, silicon_ore, deuterium |
| GasGiant | 14 | hydrogen, helium, helium3, exotic_gas |
| CrystalWorld | 8 | quartz, rare_earth, rare_crystals, dilithium |
| DeadWorld | 18 | iron, sulfur, uranium, lithium |
| Terran | 8 | iron, carbon, water, copper |
| AsteroidBelt | 8 | nickel, platinum, iridium, gold |
| IceWorld | 6 | water, liquid_methane, ammonia, deuterium |

> `level_bias` menggeser bobot ke biome rare (CrystalWorld/GasGiant) saat level galaksi naik.

## Galaxy Animation (lihat `14-galaxy-animation.md`)

| Const | Nilai | | Const | Nilai |
|-------|-------|---|-------|-------|
| `STAR_COUNT` | 600 | | `OMEGA0` | 0.05 rad/s |
| `ARM_COUNT` | 2 | | `OMEGA_R0` | 0.3 |
| `SPIRAL_B` | 0.25 | | `SCATTER` | 0.3 |
| `R_MAX` | 1.0 | | `ASPECT` | 2.0 |
| `R_CORE` | 0.2 | | `TWINKLE_FREQ` | 1.0 Hz |
| | | | `TWINKLE_AMP` | 0.3 |

## Particle Effects (lihat `15-particle-effects.md`)

| Const | Nilai | Catatan |
|-------|-------|---------|
| `PARTICLE_BUDGET` | 500 | hard cap total partikel |
| `EMITTER_MAX` | 16 | emitter aktif bersamaan |
| `ASPECT` | 2.0 | sama dgn animasi (koreksi sel terminal) |

## Catatan TBD (perlu playtest)

- Apakah tech `completed` benar-benar tidak reset saat warp, atau soft-reset sebagian? (lihat `04` §1)
- Skema Reputation alien faction (struktur & efek konkret).
- Rumus threshold warp level ≥3 dan resource rare baru per level.
- Apakah QuantumChip masuk enum `Resource` sejak awal atau ditambah saat assembler diimplementasi.
- Keybinding configurable (file config) — saat ini hard-coded di `06`.
