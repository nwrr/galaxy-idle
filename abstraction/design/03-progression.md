# 03 — Progression: ProcGen, Ship, Research

> Bagaimana player membuka konten baru. Struct: `Galaxy`, `Planet`, `Ship`, `ResearchState`,
> `UnlockReq` di [`01-data-model.md`](01-data-model.md). RNG di [`07-architecture.md`](07-architecture.md).

## 1. Procedural Generation (Seed-based, Deterministik)

Galaksi luar **tidak menyimpan jutaan planet**. Tiap galaksi ProcGen punya satu `seed: u64`;
planet di-generate on-the-fly dari `(seed, planet_index)`.

```
planet_rng = splitmix64(seed ^ mix(planet_index))
biome      = BIOME_TABLE[ planet_rng.next() % BIOME_TABLE.len() ]
richness   = lerp(RICH_MIN, RICH_MAX, planet_rng.next_f64()) * galaxy_tier_mult
distance   = lerp(DIST_MIN, DIST_MAX, planet_rng.next_f64()) * galaxy_tier_mult
node_set   = NODES_FOR_BIOME[biome]   // resource apa yang bisa ditambang
```

- **Save hanya menyimpan** `seed` + `visited: HashSet<planet_index>`. Planet dimaterialisasi
  (`Planet` struct) hanya saat dikunjungi/aktif; sisanya dihitung ulang saat render list.
- Biome → menentukan node resource (IronWorld→Iron, GasGiant→ExoticGas, OceanPlanet→Water, dst).
  Tabel `NODES_FOR_BIOME` di [`08-balancing.md`](08-balancing.md).
- Determinisme: seed sama → galaksi identik di mesin mana pun (penting utk save kecil & share).

## 2. Struktur Galaksi

| Galaksi | Kind | Isi |
|---------|------|-----|
| **Milky Way (Lvl 0)** | `Fixed` | 10–20 planet handcrafted. Tutorial + early-game. **Anchor**, tak pernah reset. |
| **Outer Galaxies (Lvl 1..∞)** | `Procedural` | Tak terbatas. Resource makin langka & yield makin besar makin jauh. |

Milky Way detail (MVP minimal 3): Earth (T1, Terran), Mars (T1/T2), Jupiter (T2, GasGiant).

## 3. Ship sebagai Gatekeeper & Multiplier

Ship **bukan unit combat**. Dua peran:

### a. Gatekeeper (Warp Tier)
`Ship.warp_tier` menentukan galaksi/planet mana yang bisa diakses lewat `UnlockReq::WarpTier`.

| Warp Tier | Akses |
|-----------|-------|
| 1–5 | Hanya Milky Way (planet T1–T2 bertahap) |
| 6 | Buka Galaksi Tetangga (ProcGen Tier 1) |
| 10 | Buka Galaksi Jauh (ProcGen Tier 2, resource lebih langka) |

Warp Tier dinaikkan lewat **research Aerospace** (lihat §5), bukan beli langsung.

### b. Buff Pasif (Part — upgradable via §5/balancing)

| Part | Efek |
|------|------|
| **Engine** | ↓ travel time & ↑ kecepatan unlock planet baru. `travel_time *= ENGINE_FACTOR ^ engine_level` |
| **Cargo** | ↑ `Planet.stockpile_cap` planet luar. `cap = BASE_CAP * (1 + CARGO_PER_LVL * cargo_level)` |
| **Scanner** | ↑ chance & kualitas event saat travel (lihat [`05-events.md`](05-events.md)) |

Biaya upgrade part pakai rumus geometrik standar (`02-economy.md` §5), dibayar resource T2/T3.

## 4. Travel Time & Yield

Jarak diukur sebagai **Travel Time** (game-time), bukan posisi.

```
travel_secs = BASE_TRAVEL * distance * ENGINE_FACTOR ^ ship.engine
yield_mult  = 1.0 + DIST_YIELD_K * distance      // makin jauh makin besar yield resource
```

Saat `ShipStatus::Traveling`, tiap tick menambah `elapsed_secs`; setibanya (`elapsed >= total`),
planet target jadi `unlocked`/aktif dan ship kembali `Idle`. Selama travel, sistem event berjalan
(lihat `05`).

## 5. Research & Tech Tree

`ResearchLab` factory menghasilkan **Data/sec** (bukan resource fisik). Player **mengalokasikan**
Data ke satu `Tech Node` aktif yang butuh Data + waktu.

```
Data/sec total = Σ (lab.level * BASE_DATA_RATE)  untuk semua ResearchLab enabled
penyelesaian tech: butuh tech.data_cost terkumpul DAN elapsed >= tech.time_secs
```

Hanya **satu** `active` research pada satu waktu (`ResearchState.active: Option`). Selesai →
masuk `completed`, efek permanen diterapkan.

### Cabang Tech Tree

| Cabang | Efek |
|--------|------|
| **Extraction** | Buka bangunan tambang baru / `tech_multiplier(resource)` naik |
| **Manufacturing** | Buka recipe crafting baru (yang tidak butuh blueprint merchant) |
| **Aerospace** | ↑ Warp Tier ship & buka part upgrade tier baru |
| **Astro-Cartography** | Buka akses galaksi ProcGen yang lebih jauh (Tier 2, 3, …) |

Contoh tech node (final di `08`):

| Tech id | Cabang | data_cost | time | Unlock |
|---------|--------|-----------|------|--------|
| `aero_warp_mk2` | Aerospace | 5,000 | 10m | Warp Tier → 6 (akses Outer T1) |
| `manu_titanium` | Manufacturing | 2,000 | 5m | Recipe `alloy_forge` |
| `ext_deep_core` | Extraction | 8,000 | 20m | Building tambang utk planet T2 |
| `astro_far_warp` | Astro-Cartography | 20,000 | 40m | Akses Outer Galaxy Tier 2 |

Definisi prasyarat antar-tech (`depends_on`) disimpan di tabel statis tech tree (lihat `08`), bukan
di state — state hanya menyimpan `completed` + `active`.

## 6. Loop Progression

```
Research Data ──▶ tech node ──┬─▶ Warp Tier ↑ ──▶ buka planet/galaksi baru
                              ├─▶ recipe/multiplier baru ──▶ ekonomi lebih kuat
                              └─▶ akses galaksi jauh ──▶ resource rare ──▶ syarat Warp Jump
```

Ujung loop ini menyiapkan **Warp Threshold** untuk prestige di [`04-prestige.md`](04-prestige.md).
