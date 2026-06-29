# 16 — Procedural Generation: Logika & Math

> Logika ProcGen mendalam untuk galaksi luar (Lvl ≥1). Memperluas overview di
> [`03-progression.md`](../design/03-progression.md) §1 dan RNG di [`07-architecture.md`](../design/07-architecture.md)
> (`rng::SplitMix64`, `planet_rng`). **Deterministik penuh dari seed** → save hanya menyimpan seed +
> `visited`, planet dihitung ulang on-the-fly.

## Kontrak Determinisme

1. Output **hanya** fungsi dari `(seed, index)` — tidak ada sumber waktu, thread-id, atau state global.
2. Urutan konsumsi RNG **tetap** (lihat pipeline). Menambah field baru → tambahkan di **akhir**
   konsumsi agar field lama tidak bergeser (kompatibilitas seed).
3. RNG ProcGen **terpisah** dari RNG event (yang non-deterministik, [`07`](../design/07-architecture.md)).

## Hirarki Seed

```
GLOBAL_SEED (u64, dipilih saat new game, disimpan di save)
   │  galaxy_seed(level, idx) = splitmix(GLOBAL_SEED ^ mix(level) ^ mix(idx))
   ▼
GALAXY_SEED ──┬─ derive("name")     → name generator
              ├─ derive("layout")   → posisi & jumlah planet
              └─ planet_rng(seed, i)→ tiap planet
```

`mix(x) = x.wrapping_mul(0x9E3779B97F4A7C15)`. `derive(tag)` = `SplitMix64(seed ^ hash(tag))` →
sub-stream independen agar penambahan fitur tidak mengganggu stream lain.

## Pipeline Generate Galaksi

```rust
fn generate_galaxy(global_seed: u64, level: u8, idx: u32) -> GalaxyMeta {
    let gseed = galaxy_seed(global_seed, level, idx);
    let mut layout = derive(gseed, "layout");

    let tier = galaxy_tier(level);                         // mempengaruhi richness & jarak
    let planet_count = PLANET_MIN + (layout.next_u64() % (PLANET_MAX - PLANET_MIN + 1)) as u32;
    let name = gen_name(derive(gseed, "name"));

    GalaxyMeta { seed: gseed, level, tier, planet_count, name }
}
```

`galaxy_tier_mult` = `1.0 + 0.5·(level-1)` ([`08-balancing.md`](../design/08-balancing.md)) — galaksi makin
tinggi: richness & jarak (yield) naik, resource rare lebih sering.

## Pipeline Generate Planet (urutan RNG TETAP)

```rust
fn generate_planet(galaxy_seed: u64, i: u32, level: u8) -> Planet {
    let mut r = planet_rng(galaxy_seed, i);   // SplitMix64(seed ^ mix(i))

    // 1. Posisi pada peta (polar) — untuk Galaxy Map & travel distance
    let dist  = lerp(DIST_MIN, DIST_MAX, r.next_f64()) * galaxy_tier_mult(level);
    let angle = r.next_f64() * TAU;

    // 2. Biome — weighted pick (+ value-noise clustering opsional)
    let biome = weighted_biome(&mut r, level);

    // 3. Tier planet — naik dgn jarak (planet jauh lebih kaya/rare)
    let tier  = tier_from_distance(dist, level);

    // 4. Nodes — dari biome, jumlah & richness di-roll
    let nodes = gen_nodes(&mut r, biome, level);

    // 5. Anomaly seeding — peluang ada anomali/event tetap di planet ini
    let anomaly = r.next_f64() < ANOMALY_CHANCE * (1.0 + 0.1*level as f64);

    // 6. Nama
    let name = gen_name(derive(galaxy_seed, &format!("planet{i}")));

    Planet { /* unlocked:false, unlock_req: WarpTier(tier_to_warp(level)), ... */ }
}
```

### 2. Biome — Weighted Pick

Pakai tabel bobot `BIOME_TABLE` ([`08-balancing.md`](../design/08-balancing.md)); galaksi tinggi menggeser
bobot ke biome rare (CrystalWorld, GasGiant kaya exotic).

```
total = Σ weight_b · level_bias(b, level)
roll  = r.next_f64() · total
pilih b pertama yang membuat cumulative ≥ roll
```

**Value-noise clustering (opsional, agar biome tidak acak total):** alih-alih murni weighted, sample
nilai noise `n = value_noise(dist, angle, gseed)` lalu petakan `n` ke biome via threshold. Ini
membuat region galaksi punya "tema" (sektor es, sektor gas) — lebih natural. `value_noise` =
interpolasi nilai hash pada grid + smoothstep (deterministik dari koordinat+seed).

### 3. Tier dari Jarak

```
tier_from_distance(dist, level):
    base = if dist < D1 {1} else if dist < D2 {2} else {3}
    clamp(base + (level≥3 ? 1 : 0), 1, 3)    // galaksi jauh menaikkan tier dasar
```

Planet dekat = T1 (mudah, resource basic); jauh = T3 (rare, butuh warp tier tinggi). Sinkron dengan
`UnlockReq::WarpTier`.

### 4. Generate Nodes dari Biome

```
templates = NODES_FOR_BIOME[biome]          // mis. GasGiant → [hydrogen, helium, exotic_gas]
node_count = NODE_MIN + r.range(0, NODE_MAX-NODE_MIN)
untuk tiap node:
    resource = weighted_pick(templates, r)   // resource rare punya bobot kecil
    richness = lerp(RICH_MIN, RICH_MAX, r.next_f64()) * galaxy_tier_mult(level)
    push ResourceNode { resource, richness, level:0 }
```

`NODES_FOR_BIOME` final di [`08-balancing.md`](../design/08-balancing.md); semua id ada di
[`10-resources.md`](10-resources.md).

## Name Generator (deterministik)

Markov-ringan / penggabung suku-kata dari tabel:

```
PREFIX = ["And","Ori","Vega","Cyg","Lyr","Cas","Dra","Hel","Xan","Zor","Kep","Tau"]
MID    = ["o","a","e","ome","ade","ira","une","ax","yx","or","el","is"]
SUFFIX = ["da"," on","us","ar","ix","eus","ara","ion","is","ae","or"]
GREEK  = ["Alpha","Beta","Gamma","Delta","Prime","Minor","Major"]   // opsional sufiks

gen_name(rng):
    s = PREFIX[rng % len] + MID[rng % len] + SUFFIX[rng % len]
    if rng.f64() < 0.3: s += " " + GREEK[rng % len]
    // contoh: "Andomeda", "Vegaireus Prime", "Zoryxis"
```

Galaksi & planet pakai tabel sama tapi sub-seed beda (`derive("name")` vs `derive("planet{i}")`) →
nama unik tapi reproducible.

## Layout Peta (Galaxy Map view)

Planet diplot di [`06-ui.md`](../design/06-ui.md) Galaxy Map pakai `(dist, angle)` → proyeksi polar sama
seperti [`14-galaxy-animation.md`](14-galaxy-animation.md) §Proyeksi (koreksi `ASPECT`). **Spacing**:
bila dua planet jatuh ke sel sama, geser salah satu `angle` sedikit (deterministik via
`r.next_f64()` cadangan) agar tidak tumpang-tindih.

## Materialisasi & Save

```
saat player SCAN/SEND ship ke planet index i:
    planet = generate_planet(galaxy.seed, i, level)   // hitung on-the-fly
    galaxy.planets.push(planet)                        // materialize
    galaxy.visited.insert(i)                            // hanya index ini yang disimpan
```

- **Belum dikunjungi** → tidak ada di `planets`; daftar di Galaxy Map dihitung ulang dari seed tiap
  render (cepat, `O(planet_count)`).
- **Save** menyimpan `GalaxyKind::Procedural { seed, visited }` + perubahan state planet yang sudah
  dimaterialisasi (factory, node level). Definisi dasar tidak disimpan → save tetap kecil.

## Pseudocode End-to-End

```rust
// Render daftar planet galaksi ProcGen i pada level L:
let gmeta = generate_galaxy(global_seed, L, i);
for idx in 0..gmeta.planet_count {
    let p = if galaxy.visited.contains(&idx) {
        galaxy.planet_by_index(idx)          // versi termaterialisasi (punya progress)
    } else {
        generate_planet(gmeta.seed, idx, L)  // versi murni dari seed (preview)
    };
    draw_planet_row(p);                       // nama, biome, jarak, travel time, status
}
```

## Konstanta (ke `08-balancing.md`)

| Const | Nilai | Catatan |
|-------|-------|---------|
| `PLANET_MIN` / `PLANET_MAX` | 5 / 12 | jumlah planet per galaksi ProcGen |
| `NODE_MIN` / `NODE_MAX` | 2 / 5 | node per planet |
| `D1` / `D2` | 3.5 / 7.0 | ambang jarak utk tier planet |
| `ANOMALY_CHANCE` | 0.15 | peluang planet punya anomali |
| `RICH_MIN/MAX`, `DIST_MIN/MAX`, `BIOME_TABLE`, `NODES_FOR_BIOME` | — | sudah ada di `08` |

## Verifikasi Determinisme (test)

- `generate_planet(seed, i, L)` dipanggil dua kali → hasil identik (field-by-field).
- Round-trip save: simpan `{seed, visited}`, muat ulang, regenerasi planet belum dikunjungi → sama.
- Mengubah `GLOBAL_SEED` → galaksi berbeda; seed sama di mesin lain → identik.
