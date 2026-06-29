# 07 — Arsitektur, Game Loop, Save

> Keputusan teknis yang **dikunci**. Memetakan domain di [`01-data-model.md`](01-data-model.md) ke
> modul Rust konkret. `edition = "2024"`.

## Tech Stack (dikunci)

| Kebutuhan | Pilihan | Alasan |
|-----------|---------|--------|
| TUI render | `ratatui` + `crossterm` | Standar TUI Rust, widget + backend event |
| Portrait karakter | `ratatui-image` + `image` | Render PNG asli di terminal (wajah butuh resolusi; ASCII tak memadai) |
| Sprite celestial | `.ans` half-block + `ansi-to-tui` | Warna per-biome (biome harus jelas beda); 9 varian size×depth |
| Serialisasi save | `serde` + `serde_json` | Save human-readable, mudah export/import |
| Konten statis | `serde` + `ron` | Katalog di-edit tangan (komentar, enum, trailing comma) — lihat `content.rs` |
| RNG ProcGen | `splitmix64` custom (no crate) | Deterministik dari seed, zero-dep, reproducible |
| Waktu | `std::time` (`SystemTime`/`Instant`) | Tick & offline delta |
| Path save | XDG (`$XDG_DATA_HOME`, fallback `~/.local/share`) | Konvensi Linux |

`Cargo.toml` dependency yang ditargetkan:

```toml
[dependencies]
ratatui = "0.29"
crossterm = "0.28"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
ron = "0.8"
ratatui-image = "1"   # render portrait karakter (PNG) di terminal (Sixel/Kitty/iTerm2 + half-block)
image = "0.25"        # decode PNG portrait
ansi-to-tui = "7"     # parse .ans (sprite celestial half-block berwarna) → ratatui Text/Style
```

(Versi indikatif; pin saat implementasi.)

## Game Loop

Dua frekuensi yang **decoupled**: simulasi logical & render.

```
loop {
    now = Instant::now();

    // 1. Input (non-blocking poll crossterm, timeout pendek)
    handle_input(&mut app);

    // 2. Simulasi: jalankan tick yang "due"
    while accumulated_time >= TICK_DURATION {        // TICK_DURATION = 1s
        sim::tick::step(&mut app.state);             // satu detik game-time
        accumulated_time -= TICK_DURATION;
        app.state.tick += 1;
    }

    // 3. Render (~10–15 FPS, animasi pakai frame counter)
    if now - last_render >= RENDER_INTERVAL {        // ~66–100ms
        ui::draw(&mut terminal, &app);
        app.render_frame += 1;
        last_render = now;
    }

    // 4. Autosave berkala
    if now - last_save >= AUTOSAVE_INTERVAL {        // 30s
        save::write(&app.state)?;
        last_save = now;
    }
}
```

- **`sim::tick::step`** adalah satu langkah simulasi 1 detik game-time. Akumulasi fractional
  resource diperbolehkan (`f64`). Idempoten terhadap urutan: produksi → konsumsi/craft → auto-sell
  → research → travel/event → merchant.
- Catch-up: jika frame lambat, loop menjalankan beberapa tick sampai `accumulated_time` habis
  (mencegah drift), dengan **batas maksimum** tick per frame agar tidak freeze (sisanya jadi
  offline progress).

### Urutan dalam satu tick (`sim::tick::step`)

```
1. Extractor: node → stockpile (hormati cap)
2. Refinery: konsumsi raw → crafted (atau catat deficit)
3. Upkeep: kurangi Energy/Credits; terapkan penalti deficit
4. Auto-sell: surplus → credits
5. Research: tambah Data; majukan active research
6. Ship travel: majukan elapsed; resolusi tiba; roll event (interval)
7. Anchor feed: passive resource Milky Way → galaksi aktif
8. Merchant: cek expire/restock
9. Apply temp buffs expiry
```

## Offline Progress

Saat load, hitung `delta = now_unix - state.last_saved_unix`.

```
offline_secs = min(delta, OFFLINE_CAP)          // cap mis. 8 jam (di 08)
efficiency   = OFFLINE_BASE_EFF * (1 + offline_eff_upgrade_bonus)  // <1.0, di-tune
sim_offline_secs = floor(offline_secs * efficiency)
```

Strategi: jalankan `sim::tick::step` sebanyak `sim_offline_secs` **secara batch** (loop cepat tanpa
render). Karena event/travel juga jalan, batasi: event saat offline boleh di-**queue** untuk
direview player saat kembali (tidak auto-resolve kecuali ada rule). Tampilkan ringkasan
"Selama kamu pergi: +X Iron, +Y Credits, 2 event menunggu".

> Optimisasi opsional (pasca-MVP): hitung produksi steady-state secara analitik untuk durasi panjang
> alih-alih loop per-detik. MVP cukup loop batch dengan `OFFLINE_CAP` wajar.

## RNG Deterministik (`rng.rs`)

```rust
pub struct SplitMix64(pub u64);

impl SplitMix64 {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    pub fn next_f64(&mut self) -> f64 { // [0,1)
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Planet ProcGen: turunkan RNG dari (galaxy_seed, planet_index) — selalu sama.
pub fn planet_rng(galaxy_seed: u64, planet_index: u32) -> SplitMix64 {
    SplitMix64(galaxy_seed ^ (planet_index as u64).wrapping_mul(0x9E3779B97F4A7C15))
}
```

ProcGen pakai `planet_rng` → galaksi/planet reproducible hanya dari `seed` (save tetap kecil,
lihat [`03-progression.md`](03-progression.md)).

Catatan: RNG **event** saat travel boleh memakai sumber non-deterministik (waktu) karena tidak perlu
reproducible dan tidak disimpan; pisahkan dari RNG ProcGen.

## Save System

- **Format:** JSON via `serde_json` (pretty optional). `GameState.version` untuk migrasi.
- **Lokasi:** `${XDG_DATA_HOME:-~/.local/share}/galaxy-idle/save.json`. Buat dir bila belum ada.
- **Autosave:** tiap `AUTOSAVE_INTERVAL` (30s) + saat quit (`q`/`Ctrl+c`) + sebelum warp jump.
- **Atomic write:** tulis ke `save.json.tmp` lalu `rename` → cegah korup saat crash.
- **Export/Import:** salin file JSON mentah (player incremental suka backup/share). Import =
  validasi `version` + deserialize + offline progress dihitung dari `last_saved_unix`.
- **Migrasi:** bila `version` lebih lama, jalankan fungsi migrasi berurutan sebelum dipakai.

```rust
pub fn write(state: &GameState) -> io::Result<()> {
    let path = save_path();                 // XDG
    fs::create_dir_all(path.parent().unwrap())?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(state)?)?;
    fs::rename(tmp, path)                    // atomic
}
```

## Content Loading (`content.rs`, data-driven)

Konten (resource/item/recipe/building) berskala ratusan → **data-driven**, di-load dari `data/*.ron`
sekali saat startup jadi `Content` registry read-only ([`01-data-model.md`](01-data-model.md)).

```rust
pub fn load_content(dir: &Path) -> Result<Content, ContentError> {
    let resources: Vec<ResourceDef> = ron::from_str(&fs::read_to_string(dir.join("resources.ron"))?)?;
    let items:     Vec<ItemDef>     = ron::from_str(&read(dir, "items.ron")?)?;
    let recipes:   Vec<RecipeDef>   = ron::from_str(&read(dir, "recipes.ron")?)?;
    let buildings: Vec<BuildingDef> = ron::from_str(&read(dir, "buildings.ron")?)?;
    let content = Content::intern(resources, items, recipes, buildings); // string → u32 handle
    content.validate()?;   // tiap id yang dirujuk ADA (recipe→resource/building, building→recipe, dst)
    Ok(content)
}
```

- **Interning:** `Content::intern` membangun tabel `String → Id(u32)` + `Vec<Def>` indexed by id.
  Selama runtime semua referensi pakai handle `u32` (cepat); string hanya untuk load & save.
- **Validasi referensi:** `validate()` memastikan setiap `ResourceId`/`RecipeId`/`BuildingId`/`ItemId`
  yang dirujuk antar-katalog benar-benar ada — gagal-cepat saat startup, bukan panic saat bermain.
  (Memenuhi verifikasi konsistensi #1/#2 katalog.)
- `data/` bisa di-embed (`include_str!`) untuk rilis single-binary, atau dibaca dari disk untuk
  modding. Default: baca disk, fallback embed.

### Save ↔ Content (id portability)

Save (JSON) menyimpan id sebagai **string** (`"iron"`), bukan handle `u32` interned (tidak stabil bila
urutan `data/` berubah). Saat load save: konversi `String → Id` via `Content`; id yang tak dikenal
(konten dihapus) → laporkan & skip, jangan crash. `save/mod.rs` memegang konversi dua-arah ini.

## Portrait Karakter (`ui/portrait.rs`, ratatui-image)

Karakter (NPC merchant, alien encounter, crew) **bukan ASCII** — wajah butuh resolusi. Dirender
sebagai gambar asli pakai `ratatui-image`.

- **Picker** (deteksi protokol terminal) dibuat **sekali** saat startup:
  `Picker::from_query_stdio()` → memilih Sixel/Kitty/iTerm2 bila didukung, fallback Unicode
  half-block (`▀`) di terminal biasa. Disimpan di `App`.
- Tiap portrait di-decode (`image::open(path)`) → `picker.new_resize_protocol(img)` → `StatefulImage`
  digambar pada `Rect` panel. Protokol di-cache per portrait id (jangan decode ulang tiap frame).
- `AssetEntry.w/h` portrait = **area tampil** (sel) tempat widget digambar; `ratatui-image`
  menyesuaikan gambar ke area (jaga rasio). Resolusi sumber 512×512 (di-publish `gen_assets.py`).
- Hanya portrait yang sedang tampil yang dimuat (lazy); bersihkan cache saat panel ditutup.

```rust
// startup
let mut picker = Picker::from_query_stdio()?;     // atau Picker::from_fontsize((w,h))
// saat menampilkan portrait
let dyn_img = image::ImageReader::open(path)?.decode()?;
let mut proto = picker.new_resize_protocol(dyn_img);   // cache by id
frame.render_stateful_widget(StatefulImage::default(), area, &mut proto);
```

> Fallback: di terminal tanpa protokol grafis, half-block tetap menampilkan wajah yang terbaca
> (jauh lebih baik dari ASCII 44×32). Banner/frame = teks; sprite celestial = ANSI berwarna (§Sprite).

## Sprite Celestial (`ui/sprite.rs`, ANSI half-block berwarna)

Planet/star/ship **bukan ASCII mono** — biome harus bisa dibedakan dari warna. Disimpan sebagai
`.ans` half-block (`▀`/`▄`: tiap sel = 2 piksel vertikal, fg atas / bg bawah → resolusi 2× + warna
penuh). Tiap base punya **9 varian**: ukuran `{sm 20×10, md 40×20, lg 64×32}` × depth
`{tc, 256, 16}` — penamaan `sprites/<base>.<size>.<depth>.ans`.

- **Parse `.ans` → `ratatui` `Style`+`Span`**: pakai crate **`ansi-to-tui`** (`IntoText`) atau parser
  SGR setara → `Text`/`Line` ber-`Style`, digambar via `Paragraph` pada `Rect`.
- **Deteksi color-depth terminal** (mis. `$COLORTERM=truecolor`, terminfo `Tc`/`max_colors`) →
  pilih depth `tc`/`256`/`16`. Gagal-aman ke `16`.
- **Responsif:** pilih `size` per luas `Rect` panel (lg utk Full, md Compact, sm Minimal); ratatui
  meng-clip bila perlu. Untuk resize **mulus** (M9), sprite di-render **procedural langsung di Rust**
  (math identik `gen_assets.py`: sphere shading + noise + palet biome → half-block) pada ukuran
  `Rect` aktual; `.ans` jadi sumber/preview/fallback build-time.
- `AssetEntry.path` = varian canonical `lg.tc`; runtime menurunkan suffix size/depth.

## Layout Modul `src/`

Pemetaan 1:1 ke domain di `01-data-model.md`:

```
src/
  main.rs              // setup terminal, load content, panggil app::run, teardown
  app.rs               // App { state, content, ui_state, timers }, event loop, input dispatch
  rng.rs               // SplitMix64, planet_rng, derive
  content.rs           // load Content dari data/*.ron, intern id, validasi referensi
  game/
    mod.rs
    state.rs           // GameState, Settings, AutoSellRule
    defs.rs            // Resource/Item/Recipe/BuildingDef, Content, Registry (09-13)
    economy.rs         // Factory/Node logic, recipe runtime, market, auto-sell
    galaxy.rs          // Galaxy, Planet, Biome, ProcGen materialize (16)
    procgen.rs         // generate_galaxy/generate_planet, weighted_biome, name gen (16)
    ship.rs            // Ship, ShipStatus, travel, part effects
    research.rs        // ResearchState, tech tree statis, alokasi Data
    prestige.rs        // PrestigeState, warp jump, anchor, warp core calc
    events.rs          // GameEvent, EventQueue, trigger, resolusi
    merchant.rs        // Merchant, MerchantOffer, restock/expire
  sim/
    mod.rs
    tick.rs            // step(): urutan 9 fase di atas
    offline.rs         // hitung & batch offline progress
  ui/
    mod.rs             // draw() dispatch per LayoutMode
    layout.rs          // breakpoints, panel split (ratatui Layout)
    theme.rs           // ThemeChoice → warna ANSI
    galaxy_anim.rs     // animasi galaksi spiral menu utama (14) — kosmetik
    particles.rs       // ParticleSystem, Emitter (15) — kosmetik
    portrait.rs        // render PNG karakter via ratatui-image (Picker + StatefulImage)
    panels/            // resources, menu, main_view, planet, research,
                       // galaxy_map, merchant, events, warp, help
  save/
    mod.rs             // path XDG, write atomic, read, migrasi, export/import (id↔string)
  balance.rs           // semua konstanta dari 08-balancing.md sebagai `const`
data/                  // konten RON (bukan src) — di-load saat startup
  resources.ron        // 10
  milky_way.ron        // 09
  items.ron            // 11
  recipes.ron          // 12
  buildings.ron        // 13
  tech_tree.ron        // 03/08 (tech node + depends_on)
```

`balance.rs` mengkristalkan [`08-balancing.md`](08-balancing.md) jadi `const` bernama → satu tempat
untuk tuning, tidak ada magic number tersebar.

## Cross-check (review)

- Tiap modul `game/*` memetakan ke domain struct di `01`.
- `sim::tick::step` menyentuh persis fase yang dijelaskan di `02`–`05`.
- ProcGen di `galaxy.rs`/`procgen.rs` memakai `rng::planet_rng` → konsisten "save kecil" di `03`/`16`.
- `prestige.rs` tidak menyentuh `anchor_galaxy` & `PrestigeState` saat reset (lihat `04`).
- `content.rs::validate()` menjamin semua id katalog `09`–`13` saling konsisten saat startup.
- `ui::galaxy_anim`/`particles` (14/15) hanya baca `state` (read-only), update di loop render — tidak
  pernah memutasi `GameState`.
