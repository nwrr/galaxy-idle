# CHECKLIST — Build Flow (Awal → Goal)

Agent mencentang `[x]` saat item selesai **dan** verify hijau. Kerjakan urut per milestone.
Spec acuan di kolom terakhir → file `abstraction/`.

## M0 — Scaffold
- [x] `Cargo.toml`: tambah deps `ratatui`, `crossterm`, `serde`(derive), `serde_json`, `ron` — `07`
- [x] Buat skeleton modul sesuai layout `src/` (`app.rs`, `game/`, `sim/`, `ui/`, `save/`, `rng.rs`, `content.rs`, `balance.rs`) — `07`
- [x] `rng.rs`: `SplitMix64`, `planet_rng`, `derive` + test determinisme — `07`,`16`
- [x] `balance.rs`: const global dari `08` — `08`
- [x] `cargo build` + `cargo test` hijau (skeleton)

## M1 — Content Loader (data-driven)
- [x] Buat `data/resources.ron` dari katalog — `10`
- [x] Buat `data/milky_way.ron` (Sol system) — `09`
- [x] Buat `data/items.ron`, `recipes.ron`, `buildings.ron`, `tech_tree.ron` — `11`,`12`,`13`,`03`
- [x] `content.rs`: load RON → `Content`, intern id, `validate()` referensi silang — `07`,`01`
- [x] Test: load semua data, validasi id (resource↔recipe↔building) lulus

## M2 — Core Simulation
- [x] `game/state.rs`: `GameState` + sub-state sesuai `01`
- [x] `game/economy.rs`: extractor (node→stockpile), refinery (recipe), auto-sell, market — `02`
- [x] `sim/tick.rs`: `step()` 9 fase berurutan — `07`
- [x] Test: 1 tick menghasilkan net income benar; deficit tertangani — `02`

## M3 — TUI Shell
- [x] `app.rs`: event loop (input non-blocking, tick 1Hz, render decoupled), quit `q`/Ctrl+c — `07`
- [x] `ui/layout.rs`: breakpoint Full/Compact/Minimal — `06`
- [x] `ui/theme.rs`: color scheme + fallback mono — `06`,`assets/icons/glyphs.md`
- [x] `ui/panels/`: resources, menu, main_view, shortcuts (layout Full) — `06`
- [x] `src/bin/snapshot.rs`: render view ke `TestBackend` → stdout teks — `agent/test/README.md`
- [x] Snapshot `main_menu_120x40` cocok + lulus `visual_checks.md`

## M4 — Planet & Building
- [x] Panel planet (nodes, factory slots, spaceport, research) — `06`
- [x] Aksi: build/upgrade extractor, refinery, lab; aturan slot & tier — `02`,`13`
- [x] Keybinding navigasi list (j/k, Enter, 1–9) — `06`
- [x] Snapshot `planet_view_80x30` cocok

## M5 — Research
- [x] `game/research.rs`: Data/sec, alokasi ke tech node, completion — `03`
- [x] Tech tree dari `data/tech_tree.ron`, efek (Warp Tier ↑, recipe/multiplier) — `03`
- [x] UI research terminal (available/current + progress bar) — `03`,`06`

## M6 — Ship & Travel
- [x] `game/ship.rs`: warp tier gatekeeping, part buff (engine/cargo/scanner) — `03`
- [x] Travel time + unlock body Sol system; status Traveling di UI — `03`,`09`

## M7 — Save / Offline
- [x] `save/mod.rs`: write atomic JSON (XDG), read, id↔string, migrasi versi — `07`
- [x] `sim/offline.rs`: time-delta batch, cap, efisiensi — `07`
- [x] Test round-trip: save→load identik; offline progress masuk akal

## M8 — Prestige
- [x] `game/prestige.rs`: warp jump (reset galaksi aktif, simpan prestige/anchor), Warp Core calc — `04`
- [x] Anchor feed Milky Way → galaksi aktif; permanent upgrade — `04`
- [x] UI Warp Navigation + konfirmasi — `04`,`06`

## M9 — ProcGen + Polish
- [x] `game/procgen.rs`: generate galaxy/planet deterministik dari seed — `16`
- [x] Galaxy Map view (planet list ProcGen, scan/send) — `06`,`16`
- [x] `ui/galaxy_anim.rs`: animasi spiral menu utama — `14`
- [x] `ui/particles.rs`: particle dasar (engine exhaust / warp trail) — `15`
- [x] `ui/portrait.rs`: render portrait karakter PNG via `ratatui-image` (Picker + StatefulImage, cache per id, fallback half-block) — `07`,`06`
- [x] `ui/sprite.rs`: muat sprite celestial `.ans` half-block berwarna (parse via `ansi-to-tui`); pilih ukuran sm/md/lg per panel + depth tc/256/16 per kapabilitas terminal — `07`,`06`
- [x] Responsive penuh 3 breakpoint; snapshot tiap ukuran cocok — `06`
- [x] Void Merchant + event dasar (queue, resolusi) — `05`,`04`

## Goal Gate
- [x] Semua DoD di [`GOALS.md`](GOALS.md) (G1–G10) ✅
- [x] `scripts/verify.sh` hijau penuh
- [x] `PROGRESS.md` ditutup dengan ringkasan rilis
