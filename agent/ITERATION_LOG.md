# ITERATION LOG (append-only)

Agent menambahkan satu blok di **atas** (terbaru dulu) tiap iterasi. Jangan edit blok lama.

## Format Blok

```
### iter <N> — <YYYY-MM-DD HH:MM> — <milestone> / <checklist item>
- Changed: <file yang disentuh, ringkas>
- Why: <alasan / item checklist>
- Verify: <fmt|clippy|test|snapshot → hasil>
- Snapshot: <view@WxH → cocok? assertion gagal apa?>
- Assumptions: <asumsi desain bila spec ambigu>
- Next: <item berikutnya>
```

---

<!-- entri pertama akan ditambahkan agent di sini -->

### iter 18 — 2026-06-29 — M4 / keybinding + wiring aksi (M4 SELESAI)
- Changed: `src/app.rs` (App +field sel, active_pi, planet_keys: j/k pilih slot + 1/2/Enter aksi → actions; 2 test)
- Why: M4 item 3 — navigasi list + dispatch aksi ke UI
- Verify: conventions ✓, fmt ✓, clippy ✓, test 25/25 ✓ → HIJAU
- Tests: jk_navigation_clamps (sel 0→1→clamp 3→k=2), key_upgrades_selected_factory (sel slot1 + '2' → level 3, credits terpotong).
- Assumptions: '1' upgrade node (sel di-clamp ke nodes), '2'/Enter upgrade factory terpilih; '3'-'5' (build/lab/ship) belum diwire (butuh sub-menu pilih building → M-lanjut/M9 UI). Highlight slot terpilih belum dirender (sel tak ubah snapshot; cukup logika+test). Error aksi di-ignore (feedback toast UI menyusul).
- Next: M5 — game/research.rs (Data/sec, alokasi tech, completion) + tech tree efek + UI research

### iter 17 — 2026-06-29 — M4 / game/actions.rs build/upgrade
- Changed: `src/game/actions.rs` (BARU: build_factory, upgrade_factory, upgrade_node, try_pay, ActionError + 3 test), `src/game/mod.rs`
- Why: M4 item 2 — aksi build/upgrade + aturan slot & tier (`02`,`13`)
- Verify: conventions ✓, fmt ✓, clippy ✓, test 23/23 ✓ → HIJAU
- Tests: build mining_drill (−50 cr), upgrade (×GROWTH=−57.5 → lvl2), tier-gate (arc_furnace tier2 ditolak di tier1), dana kurang ditolak, slot terisi ditolak.
- Assumptions: (1) biaya dibayar credits (entri "credits") dari GameState.credits + resource lain dari stockpile planet. (2) slot_cost dicek thd jumlah slot bebas tapi building tetap menempati 1 slot index (model slot vektor; multi-slot fisik disederhanakan — catat). (3) hanya Extractor/Refinery/ResearchLab buildable; Storage/Special → NotBuildable (butuh penanganan modifier khusus, M-lanjut). (4) Extractor butuh node binding, Refinery butuh recipe binding.
- Next: M4 item 3 — keybinding navigasi list (j/k, Enter, 1-9) + wiring aksi ke UI

### iter 16 — 2026-06-29 — M4 / panel planet + planet_view snapshot
- Changed: `src/ui/panels/planet.rs` (BARU), `src/ui/panels/mod.rs`, `src/ui/mod.rs` (route planet_view + draw_planet_view), `src/app.rs` (demo Earth diperkaya: 3 node + 4 factory slot + ship), `tests/snapshots.rs` (test planet_view), golden `planet_view_80x30.txt` (regen)
- Why: M4 item 1 (panel planet) + item 4 (snapshot planet_view)
- Verify: conventions ✓, fmt ✓, clippy ✓, test 20/20 (main_menu + planet_view golden) ✓ → HIJAU
- Visual checks planet_view (LIHAT 80×30): header "EARTH - Sector 001 (Lvl 1)" ✓, RESOURCE NODES/FACTORIES/SHIP/RESEARCH ✓, [1]-[5] aksi ✓, Status: Idle ✓. (FACTORIES line ke-4 ter-clip di tepi panel — kosmetik, terbaca.)
- Catatan: enrich demo Earth TAK ubah main_menu (RESOURCES baca stockpile saja) — golden main_menu tetap. Clippy field_reassign_with_default dihindari via struct-update Ship.
- Next: M4 item 2 — aksi build/upgrade extractor/refinery/lab (`02`,`13`) + item 3 keybinding j/k/Enter/1-9

### iter 15 — 2026-06-29 — M3 / app.rs event loop (M3 SELESAI)
- Changed: `src/app.rs` (App +field view, handle_key, run/run_loop/event_loop crossterm)
- Why: M3 item 1 — event loop dua-frekuensi + quit
- Verify: conventions ✓, fmt ✓, clippy ✓, test 19/19 ✓ → HIJAU
- Detail: raw mode + AlternateScreen setup/teardown bersih; input non-blocking (poll 10ms) → quit q/Ctrl+c, key g/p/r/m/w/Esc ganti view; tick catch-up dibatasi MAX_TICKS_PER_FRAME (sisa→offline M7); render decoupled RENDER_INTERVAL_MS; caller naikkan state.tick.
- Fix: 2 baris doc >100; ticks u32.
- Assumptions: run() pakai App::demo() sbg state awal (new_game builder belum ada; M-lanjut). Loop runtime tak bisa di-snapshot (butuh TTY) → diverifikasi via compile+clippy; render path sudah terbukti via snapshot main_menu. G1 belum dicentang (perlu run nyata utk "tak panic").
- Next: M4 — panel planet (nodes/factory/spaceport/research) + aksi build/upgrade

### iter 14 — 2026-06-29 — M3 / split ui/panels/
- Changed: `src/ui/panels/{mod,resources,menu,main_view,shortcuts}.rs` (BARU), `src/ui/mod.rs` (draw_main_menu pakai panels::*::render, format_num jadi pub(crate), hapus inline panel)
- Why: M3 item 4 — pisah panel ke modul (CONVENTIONS + `06`)
- Verify: conventions ✓, fmt ✓, clippy ✓, test 19/19 (snapshot main_menu golden TETAP sama → refactor preserve output) ✓ → HIJAU
- Fix: 1 baris doc >100.
- Next: M3 item 1 — `app.rs` event loop crossterm (input non-blocking, tick 1Hz, render decoupled, quit q/Ctrl+c)

### iter 13 — 2026-06-29 — M3 / lib+bin + snapshot infra + main_menu
- Changed: `src/lib.rs` (BARU, crate root + demo_app), `src/main.rs` (tipis → galaxy_idle::app::run), `src/app.rs` (App{state,content} + App::demo), `src/ui/theme.rs` (BARU, palet per ThemeChoice), `src/ui/mod.rs` (buffer_to_text + draw_view + panel resources/menu/main_view/shortcuts inline), `src/bin/snapshot.rs` (BARU), `tests/snapshots.rs` (BARU), regen `agent/test/snapshots/main_menu_120x40.txt`
- Why: M3 item 3 (theme) + 5 (snapshot bin) + 6 (snapshot main_menu) + konversi crate lib+bin agar snapshot pakai `galaxy_idle::`
- Verify: conventions ✓, fmt ✓, clippy ✓, test 19/19 (+ snapshot main_menu golden) ✓ → HIJAU
- Visual checks main_menu (LIHAT frame 120×40): STELLAR IDLE ✓, border utuh ✓, no overflow ✓, RESOURCES 3 tier+angka ✓, MENU (Galaxy Map/Planets/Fleet/Research/Merchant/Settings/Save) ✓, ► fokus tepat 1 ✓, MAIN VIEW "Milky Way (Lvl 0)" ✓, SHORTCUTS 5 ✓.
- Fix: clippy is_multiple_of, 1 baris doc >100.
- Assumptions: (1) draw_view(f,&App,view) (App=state+content), bukan signature README (state-only) — perlu Content utk nama resource. snapshot.rs/tests disesuaikan. (2) Panel di-inline di ui/mod.rs; split ke ui/panels/ menyusul (item CHECKLIST belum dicentang). (3) Tanpa emoji (lebar sel deterministik); golden diregen UPDATE_SNAPSHOTS=1.
- Next: M3 — split ui/panels/ (item 4) + app.rs event loop (item 1) + Compact/Minimal

### iter 12 — 2026-06-29 — M3 / ui/layout.rs breakpoint
- Changed: `src/ui/layout.rs` (LayoutMode + layout_mode + test), `src/ui/mod.rs` (pub mod layout)
- Why: M3 item 2 — breakpoint responsif Full/Compact/Minimal (`06`)
- Verify: conventions ✓, fmt ✓, clippy ✓, test 18/18 ✓ → HIJAU
- Catatan: item kecil sengaja dipilih dulu (murni+teruji). M3 berikut butuh konversi crate ke lib+bin (snapshot.rs pakai `galaxy_idle::`) — direncanakan iter selanjutnya bareng app.rs/draw_view/snapshot bin.
- Next: M3 — konversi lib+bin, ui::theme, buffer_to_text, draw_view, panels, snapshot.rs, app.rs loop

### iter 11 — 2026-06-29 — M2 / sim/tick.rs step() 9 fase
- Changed: `src/sim/tick.rs` (step + TickReport + advance_travel + 3 test), `src/sim/mod.rs`, `src/game/state.rs` (Factory +field `building: BuildingId`), `src/game/economy.rs` (test Factory + building)
- Why: M2 item 3+4 — orkestrasi tick 9 fase + test net income/deficit
- Verify: conventions ✓, fmt ✓, clippy ✓, test 17/17 ✓ → HIJAU
- Tests: one_tick_net_income (tambang 1 iron → auto-sell → +1 credit, upkeep energy 10→9, no defisit), energy_deficit_flagged (tanpa energy → flag), travel_arrival_unlocks (Traveling→Idle, target unlocked).
- Gap spec ditutup: `Factory` TAK punya link ke BuildingDef di `01` → tambah field `building: BuildingId` agar upkeep/cost bisa di-lookup (FactoryKind tetap runtime behavior). Asumsi: upkeep tetap PER-building (tak ×level) sesuai tabel `13` literal; ×level tuning playtest. Fase 6 travel dasar jalan; fase 7 anchor/8 merchant/9 buff = stub (M8/M9). step() tak naikkan state.tick (caller/app yang naikkan, `07`).
- Next: M3 — `app.rs` event loop + TUI shell

### iter 10 — 2026-06-29 — M2 / game/economy.rs
- Changed: `src/game/economy.rs` (extractor_output, upgrade_cost, price, add_capped, run_extractors, run_refineries+deficit, auto_sell_planet + 4 test), `src/game/mod.rs`
- Why: M2 item 2 — mekanik ekonomi (`02`)
- Verify: conventions ✓, fmt ✓, clippy ✓, test 14/14 ✓ → HIJAU
- Fix: clippy collapsible_if di run_extractors → let-chain `if let A && let B` (edition 2024).
- Tests: cost geometrik (50→57.5), extractor isi stockpile (rate×level×richness=6/tick), refinery steel_mill craft + deficit saat input habis, auto-sell jaga keep_above (jual 70 → +70 credits, sisa 30).
- Assumptions: (1) stockpile_cap = cap PER-resource (01 simpan f64 tunggal). (2) recipe ber-waktu (craft_time>0) DITUNDA (butuh progress per-factory; M5/UI). Recipe instan (=0) diproses penuh `level` kali/tick. (3) fungsi murni per-planet; orkestrasi (galaksi aktif, upkeep, tech_mult) di sim::tick (item berikut).
- Next: M2 item 3 — `sim/tick.rs` step() 9 fase

### iter 9 — 2026-06-29 — M2 / game/state.rs
- Changed: `src/game/state.rs` (GameState + semua sub-state: ids, Factory/Node, Planet/Galaxy/Biome/UnlockReq, Ship/ShipStatus, Research, Prestige/Anchor, Merchant, Settings), `src/game/events.rs` (GameEvent/EventQueue stub), `src/game/mod.rs`
- Why: M2 item 1 — kontrak state runtime (`01`)
- Verify: conventions ✓, fmt ✓, clippy ✓, test 10/10 ✓ → HIJAU
- Fix: 1 baris >100 kolom (Ship::default & enum varian) → `cargo fmt` reflow.
- Assumptions: (1) Serialize/Deserialize DITUNDA ke M7 (id interned tak stabil utk save; `01` string-layer). State pakai Clone+Debug saja. (2) ResearchState.completed & PrestigeState.permanent_upgrades pakai String key (tech/upgrade data-driven), bukan &'static str spt `01` (pra-data-driven). blueprints tetap HashSet<RecipeId>. (3) Ship::default warp_tier=1 (asumsi start; tuning M3). (4) ThemeChoice didefinisi di state.rs (di-resolve palet di ui::theme M3). (5) GameEvent placeholder (varian M9).
- Next: M2 item 2 — `game/economy.rs` extractor/refinery/auto-sell/market (`02`)

### iter 8 — 2026-06-29 — M1 / content.rs loader + validasi
- Changed: `src/game/defs.rs` (struct/enum semua Def + Registry + Content + HasId), `src/game/mod.rs` (pub mod defs), `src/content.rs` (load_content + validate + ContentError + 2 test)
- Why: M1 item 4+5 — loader data-driven + validasi referensi silang
- Verify: conventions ✓, fmt ✓ (cargo fmt), clippy -D warnings ✓, test 8/8 ✓ → HIJAU
- Hasil penting: SEMUA 5 RON (resources/items/recipes/buildings/tech) parse bersih + 0 referensi menggantung → katalog konsisten. Test: loads_and_validates (counts + id ter-intern), dangling_ref_rejected (validate menolak ref rusak).
- Fix: fmt awal merah → `cargo fmt`.
- Assumptions: tech_tree dimuat ke `Content.techs` & divalidasi di sini (bukan modul terpisah). milky_way.ron BELUM divalidasi (ditunda M2 galaxy materialize; node resource id-nya akan dicek thd resources).
- Next: M2 — `game/state.rs` GameState + sub-state (`01`)

### iter 7 — 2026-06-29 — M1 / items+recipes+buildings+tech_tree.ron
- Changed: `data/items.ron` (58), `data/recipes.ron` (51), `data/buildings.ron` (31), `data/tech_tree.ron` (7)
- Why: M1 item 3 — katalog item/recipe/building/tech (`11`,`12`,`13`,`03`/`08`)
- Verify: conventions ✓, fmt ✓, clippy ✓, test 6/6 ✓ → HIJAU (parse RON + cross-ref divalidasi M1 item 4 content.rs)
- Assumptions (2 gap spec ditutup eksplisit, dicatat di header file):
  - ItemEffect: tambah variant `Special(kind:String, value:f64)` sbg catch-all efek artifact/consumable yang `11` deskripsikan dlm prosa (travel_mult, scanner_bonus, upkeep_energy_mult, solar_mult, dst) — enum spec hanya 7 variant. Aplikasi modifier konkret di M9.
  - BuildingDef.kind: pakai template `BuildingKind{Extractor,Refinery,ResearchLab,Storage,Special}` (tanpa payload), beda dari runtime `FactoryKind` (bawa node/recipe). `01` sebut FactoryKind tapi tak punya Storage/Special utk storage_depot/warehouse/anchor_booster/warp_gate/trade_hub.
  - solar_collector kind Extractor tapi punya recipes:["gen_energy_solar"] (penghasil energy lewat recipe).
  - tech unlock: TechMult(resources,add), UnlockBuilding(id), WarpTier(n), AccessOuterTier(n), Recipe(id).
- Next: M1 item 4 — `content.rs`: load RON → Content, intern id, validate() referensi silang

### iter 6 — 2026-06-29 — M1 / data/milky_way.ron
- Changed: `data/milky_way.ron` — wrapper (id/name/bodies) + 24 body Sol System (inner 8, belt 2, outer 12, kuiper 2), tiap body nodes+richness+unlock_req+moons
- Why: M1 item 2 — galaksi anchor handcrafted (`09`)
- Verify: conventions ✓, fmt ✓, clippy ✓, test 6/6 ✓ → HIJAU (parse RON divalidasi saat loader M1/M2)
- Assumptions: (1) wrapper top-level `(id,name,bodies:[...])` — nama field dipilih sekarang, loader galaxy (M2) menyesuaikan. (2) sun tier=0 (bintang, bukan planet-tier 1..3). (3) `09` prosa sebut "27 body" tapi tabel hanya list 24 → ikut tabel (24), prosa stale. (4) moons parent→child id; child tetap entri body sendiri.
- Next: M1 item 3 — `data/items.ron`, `recipes.ron`, `buildings.ron`, `tech_tree.ron`

### iter 5 — 2026-06-29 — M1 / data/resources.ron
- Changed: `data/resources.ron` — 88 ResourceDef (Basic ore/gas/liquid/energy, Advanced ingot/component/crystal, Rare gas/crystal/metal/component, Special exotic/currency)
- Why: M1 item 1 — katalog resource sumber kebenaran (`10`)
- Verify: conventions ✓, fmt ✓, clippy ✓, test 6/6 ✓ → HIJAU (parse RON belum diuji; menyusul saat content.rs M1 item 4)
- Assumptions: (1) metal rare (iridium/platinum/palladium/gold) → category Ore (ditambang AsteroidBelt), bukan Crystal/Exotic. (2) plasma → Exotic. (3) base_price "—" (data/warp_core/stellar_core) → 0.0. (4) void_drill_part stackable:false (komponen unik). (5) energy & data tak diduplikasi; credits ditambah. Katalog sebut ≈165 tapi yang ter-list 88 → implement yang ada.
- Next: M1 item 2 — `data/milky_way.ron` (Sol system) dari `09`

### iter 4 — 2026-06-29 — M0 / balance.rs
- Changed: `src/balance.rs` — semua const lintas-entri dari `08` (tick, offline, economy, ship/travel, warp, prestige, events, merchant, procgen, galaxy-anim, particles) + 9 compile-time invariant
- Why: M0 item 4 — kristalisasi balancing global
- Verify: conventions ✓, fmt ✓, clippy -D warnings ✓, test 6/6 ✓ → HIJAU
- Fix: clippy `assertions_on_constants` → ganti runtime test jadi `const _: () = assert!(...)`
- Assumptions: angka per-entri (harga/recipe/cost building) TIDAK di-port (sesuai `08` → katalog `data/*.ron` M1). RENDER_INTERVAL disimpan `_MS:u64`, durasi lain `_SECS:f64`. Nama const di-suffix unit utk jelas (TICK_DURATION_SECS dst).
- Next: M0 item 5 — `cargo build`+`cargo test` hijau (verifikasi skeleton M0 selesai)

### iter 3 — 2026-06-29 — M0 / rng.rs
- Changed: `src/rng.rs` — SplitMix64 (new/next_u64/next_f64/below), planet_rng, derive(seed,label) + 6 unit test
- Why: M0 item 3 — RNG deterministik ProcGen sesuai `07`
- Verify: conventions ✓, fmt ✓, clippy -D warnings ✓, test 6/6 ✓ → HIJAU
- Snapshot: n/a
- Assumptions: `below()` abaikan modulo bias (cukup utk ProcGen kosmetik). `derive(label)` = hash FNV-ish×golden lalu splitmix1 (tak ada di spec eksplisit; default aman, deterministik+label-sensitive). `#![allow(dead_code)]` sementara di rng.rs sampai dikonsumsi M9.
- Next: M0 item 4 — `balance.rs` const global dari `08`

### iter 2 — 2026-06-29 — M0 / skeleton modul
- Changed: `src/main.rs` (deklarasi mod tree), stub `app.rs` `balance.rs` `content.rs` `rng.rs`, `game/mod.rs` `sim/mod.rs` `ui/mod.rs` `save/mod.rs`
- Why: M0 item 2 — kerangka modul sesuai layout `07`
- Verify: conventions ✓, fmt ✓, clippy -D warnings ✓, test 0/0 ✓ → HIJAU
- Snapshot: n/a
- Assumptions: sub-modul (state/economy/galaxy/…) belum dibuat; dideklarasi saat diimplementasi tiap milestone agar tak ada dead_code & `game/` tetap ≤10 file
- Next: M0 item 3 — `rng.rs` SplitMix64 + planet_rng + test determinisme

### iter 1 — 2026-06-29 — M0 / Cargo.toml deps
- Changed: `Cargo.toml` (+ratatui 0.29, crossterm 0.28, serde+derive, serde_json, ron 0.8, ratatui-image 1, image 0.25, ansi-to-tui 7)
- Why: M0 item 1 — fondasi deps sesuai tech stack `07`
- Verify: conventions ✓, fmt ✓, clippy -D warnings ✓, test 0/0 ✓ → VERIFY HIJAU
- Snapshot: n/a (belum UI)
- Assumptions: `game/` di `07` punya 11 file datar → langgar CONVENTIONS §1 (≤10/folder). Saat populasi nanti akan dikelompokkan ke sub-folder tematik (mis. `game/world/` galaxy+procgen+ship, `game/econ/` economy+merchant). Belum dibuat sekarang.
- Next: M0 item 2 — skeleton modul `src/` (app.rs, rng.rs, content.rs, balance.rs, game/, sim/, ui/, save/)

### iter 0 — bootstrap — meta
- Changed: agent/ dan assets/ dibuat (tooling loop, belum ada kode game)
- Why: menyiapkan infrastruktur looping-agent + asset strategy
- Verify: n/a (belum ada crate selain hello-world)
- Next: M0 — `Cargo.toml` deps + skeleton modul
