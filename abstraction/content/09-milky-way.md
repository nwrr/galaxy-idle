# 09 — Milky Way: Sol System (Galaxy Awal / Anchor)

> Bimasakti = **Sol system** yang handcrafted. `GalaxyKind::Fixed`, level 0, **Anchor** (tak pernah
> reset — lihat [`04-prestige.md`](../design/04-prestige.md)). Ini area tutorial + early-game. Struct: `Galaxy`,
> `Planet`, `ResourceNode` di [`01-data-model.md`](../design/01-data-model.md). Semua `resource` di sini = id
> dari [`10-resources.md`](10-resources.md). Implementasi: `data/milky_way.ron`.

## Konsep

- "Planet" di sini = **body** tata surya (planet, bulan utama, sabuk asteroid, dwarf).
- `distance` dalam **AU abstrak** → menentukan travel time saat unlock (rumus di
  [`03-progression.md`](../design/03-progression.md) §4, konstanta di [`08-balancing.md`](../design/08-balancing.md)).
- `richness` per node (0.5–2.0) sudah ditetapkan manual (bukan ProcGen) agar tutorial terkurasi.
- `slots` = jumlah `factory_slots` planet. Bulan punya slot lebih sedikit.

## Skema entri (`Planet`, dari RON)

```ron
(
    id: "earth", name: "Earth", tier: 1, biome: Terran,
    distance: 1.0, slots: 6, unlock_req: None,
    nodes: [ (resource: "iron", richness: 1.5), (resource: "carbon", richness: 1.2),
             (resource: "water", richness: 2.0), (resource: "copper", richness: 1.0),
             (resource: "oxygen", richness: 1.5), (resource: "silicon_ore", richness: 0.8) ],
    moons: [ "luna" ],
),
```

## Daftar Body

### Inner System

| id | name | tier | biome | dist (AU) | slots | unlock_req | nodes (resource@richness) |
|----|------|------|-------|-----------|-------|-----------|---------------------------|
| `sun` | Sol (Star) | — | LavaWorld | 0.0 | 2 | WarpTier(4) | `solar_flux`@2.0, `hydrogen`@2.0, `helium`@1.5 (Solar Collector saja) |
| `mercury` | Mercury | 1 | DeadWorld | 0.4 | 3 | WarpTier(2) | `iron`@1.8, `nickel`@1.2, `sulfur`@1.0 |
| `venus` | Venus | 2 | LavaWorld | 0.7 | 4 | WarpTier(3) | `co2`@2.0, `sulfur`@1.8, `nitrogen`@1.2 |
| `earth` | Earth | 1 | Terran | 1.0 | 6 | **None** (start) | `iron`@1.5, `carbon`@1.2, `water`@2.0, `copper`@1.0, `oxygen`@1.5, `silicon_ore`@0.8 |
| `luna` | Luna (Moon) | 1 | DeadWorld | 1.0 | 3 | WarpTier(1) | `iron`@1.0, `helium3`@1.5, `aluminum`@1.0 |
| `mars` | Mars | 2 | IronWorld | 1.5 | 5 | WarpTier(2) | `iron`@2.0, `aluminum`@1.5, `silicon_ore`@1.2, `water`@0.6 (es) |
| `phobos` | Phobos | 1 | AsteroidBelt | 1.5 | 2 | WarpTier(2) | `nickel`@1.5, `cobalt`@1.0 |
| `deimos` | Deimos | 1 | AsteroidBelt | 1.6 | 2 | WarpTier(2) | `iron`@1.2, `carbon`@1.0 |

### Asteroid Belt

| id | name | tier | biome | dist | slots | unlock_req | nodes |
|----|------|------|-------|------|-------|-----------|-------|
| `asteroid_belt` | Asteroid Belt | 2 | AsteroidBelt | 2.7 | 4 | WarpTier(3) | `iron`@1.5, `nickel`@1.5, `platinum`@0.8, `iridium`@0.6, `gold`@0.5 |
| `ceres` | Ceres (Dwarf) | 2 | IceWorld | 2.8 | 3 | WarpTier(3) | `water`@1.8, `ammonia`@1.2, `rare_earth`@0.7 |

### Outer System (Gas Giants + Moons)

| id | name | tier | biome | dist | slots | unlock_req | nodes |
|----|------|------|-------|------|-------|-----------|-------|
| `jupiter` | Jupiter | 2 | GasGiant | 5.2 | 5 | WarpTier(3) | `hydrogen`@2.0, `helium`@2.0, `helium3`@1.2, `methane`@1.0 |
| `io` | Io | 2 | LavaWorld | 5.2 | 3 | WarpTier(4) | `sulfur`@2.0, `iron`@1.0 |
| `europa` | Europa | 2 | IceWorld | 5.2 | 3 | WarpTier(4) | `water`@2.0, `deuterium`@1.0, `oxygen`@1.2 |
| `ganymede` | Ganymede | 2 | IceWorld | 5.2 | 4 | WarpTier(4) | `water`@1.5, `silicon_ore`@1.2, `magnesium`@1.0 |
| `callisto` | Callisto | 2 | DeadWorld | 5.2 | 3 | WarpTier(4) | `iron`@1.2, `co2`@1.0, `ammonia`@1.0 |
| `saturn` | Saturn | 3 | GasGiant | 9.5 | 5 | WarpTier(5) | `hydrogen`@2.0, `helium3`@1.8, `exotic_gas`@0.8, `methane`@1.2 |
| `titan` | Titan | 3 | OceanPlanet | 9.5 | 4 | WarpTier(5) | `liquid_methane`@2.0, `methane`@1.5, `nitrogen`@1.5 |
| `enceladus` | Enceladus | 3 | IceWorld | 9.5 | 3 | WarpTier(5) | `water`@2.0, `deuterium`@1.2, `brine`@1.0 |
| `uranus` | Uranus | 3 | GasGiant | 19.2 | 4 | WarpTier(5) | `hydrogen`@1.8, `helium`@1.8, `methane`@2.0, `exotic_gas`@1.0 |
| `titania` | Titania | 3 | IceWorld | 19.2 | 3 | WarpTier(5) | `water`@1.5, `rare_earth`@0.8, `quartz`@0.8 |
| `neptune` | Neptune | 3 | GasGiant | 30.1 | 4 | WarpTier(5) | `hydrogen`@1.8, `helium3`@2.0, `exotic_gas`@1.2 |
| `triton` | Triton | 3 | IceWorld | 30.1 | 3 | WarpTier(5) | `nitrogen`@1.8, `liquid_methane`@1.5, `deuterium`@1.2 |

### Kuiper Belt / Edge (gateway ke ProcGen)

| id | name | tier | biome | dist | slots | unlock_req | nodes |
|----|------|------|-------|------|-------|-----------|-------|
| `pluto` | Pluto (Dwarf) | 3 | IceWorld | 39.5 | 3 | WarpTier(5) | `water`@1.2, `liquid_methane`@1.5, `xenon`@0.6 |
| `kuiper_belt` | Kuiper Belt | 3 | AsteroidBelt | 45.0 | 4 | WarpTier(5) | `xenon`@1.0, `iridium`@0.8, `rare_crystals`@0.5, `exotic_matter`@0.3 |

## Urutan Unlock (Tutorial → Early-Game)

```
START: Earth (gratis) ──▶ bangun extractor Iron/Carbon/Water, refinery Steel
  │  Warp Tier 1 (research aero) ──▶ Luna (He-3 utk Energy)
  │  Warp Tier 2 ──▶ Mercury, Mars, Phobos/Deimos (logam T2)
  │  Warp Tier 3 ──▶ Venus, Asteroid Belt, Ceres, Jupiter (rare metal, gas)
  │  Warp Tier 4 ──▶ Sun (Solar Collector), bulan Jupiter (es/sulfur)
  │  Warp Tier 5 ──▶ Saturn..Neptune + bulan, Pluto, Kuiper (exotic_gas, exotic_matter)
  └─ Warp Tier 6 ──▶ keluar Sol → Galaksi ProcGen Tier 1 (lihat 03 & 16)
```

Earth adalah satu-satunya body `unlock_req: None`. Sisanya gated oleh Warp Tier (dinaikkan lewat
research Aerospace, [`03-progression.md`](../design/03-progression.md) §5) → menciptakan jalur progres jelas.

## Peran Anchor

Karena Sol system tak pernah reset, pasca-Warp-Jump pertama ia jadi **mesin pasif** yang men-*feed*
resource dasar ke galaksi aktif (rumus `anchor_feed` di [`04-prestige.md`](../design/04-prestige.md) §2). Player
didorong membangun Sol system padat sebelum warp pertama agar boost anchor besar.

## Catatan

- Biome `IceWorld` & `LavaWorld` ditambahkan ke enum `Biome` ([`01-data-model.md`](../design/01-data-model.md))
  untuk akurasi tata surya; keduanya juga dipakai ProcGen ([`16-procgen-logic.md`](16-procgen-logic.md)).
- Total **27 body**. Cukup untuk berjam-jam early-game sebelum prestige pertama.
- Semua id node merujuk [`10-resources.md`](10-resources.md); `kuiper_belt` sengaja memberi
  `exotic_matter`/`rare_crystals` tipis sebagai "teaser" konten ProcGen.
