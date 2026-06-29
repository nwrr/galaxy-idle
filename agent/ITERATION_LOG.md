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

### iter 37 — 2026-06-29 — Goal Gate (G1 smoke + G10 audit) → PROJECT DONE
- Changed: BARU `tests/smoke.rs` (2 test integrasi: `app_runs_sim_and_renders_all_views_without_panic` — 120 tick step+draw lintas 5 view dgn ship Traveling utk memicu roll event baru; `app_renders_every_breakpoint_without_panic` — render 5 view × 4 ukuran 120×40/80×30/60×24/40×12); centang Goal Gate di CHECKLIST.md, semua G1–G10 di GOALS.md
- Why: Goal Gate — verifikasi G1 (stabil/tak panic) & G10 (visual_checks lulus), tutup proyek
- Verify: conventions ✓, fmt ✓, clippy -D warnings ✓, test 82 unit + 7 snapshot + 2 smoke ✓ → HIJAU
- Snapshot: audit visual_checks.md pada golden — SEMUA lulus. Full (120×40): main_menu (RESOURCES 3 tier + MENU lengkap + "Milky Way" + satu ► + SHORTCUTS≥4), research (Data/sec+total, AVAILABLE TECHS+cost/time, bar `[######....] 60% (48s left)`), galaxy_map (list ProcGen + travel time + legend `■ Planet · ◆ Star · ═ Route`), warp (target+requirement+cores+warning reset/anchor+`[1] INITIATE WARP JUMP`). Compact 80×30: planet_view (EARTH-Sector 001, NODES/FACTORIES/SHIP/RESEARCH, `[1..5]`, Status:Idle) + sidebar tipis. Minimal 60×24: tab bar+footer, border utuh, tak ada overflow.
- Assumptions: G1 `cargo run` interaktif tak bisa di-drive headless di harness (TUI butuh TTY) → diverifikasi via test integrasi yg memakai jalur sim/render/quit identik `event_loop` (poll→step→draw); logika quit `q`/Ctrl-C eksplisit di `app::event_loop`. Tak ada visual_check khusus untuk panel event/merchant (model siap, view = polish pasca-DoD).
- Next: TIDAK ADA — semua DoD ✅; loop DIHENTIKAN atas permintaan user setelah Goal Gate. Backlog pasca-DoD: panel [LOG/EVENTS] + Merchant view, wiring sprite/portrait ke view, TempProductionBuff.

### iter 36 — 2026-06-29 — M9 / Void Merchant + event dasar (queue, resolusi)
- Changed: `src/game/events.rs` (ganti skeleton → model penuh: `EventCategory`/`EventKind`/`EventReq`/`EventEffect`/`EventOption`/`GameEvent`; `EventQueue` {pending,next_id,ticks_since_roll} + `event_chance`, `make_event`, `expire`, `take_option`; free fn `tick_travel_events` roll deterministik saat Traveling, `resolve_event`+`apply_effect` ke stockpile/travel time, `restock_merchant`+`update_merchant`; 6 test), `src/sim/tick.rs` (fase 6 panggil `tick_travel_events`, fase 8 `update_merchant`)
- Why: M9 item 8 (terakhir) — Void Merchant + event dasar (queue, resolusi) `05`,`04`
- Verify: conventions ✓, fmt ✓, clippy -D warnings ✓, test 80 unit (6 events) + 7 snapshot ✓ → HIJAU
- Snapshot: n/a (model+sim, belum ada view event/merchant). Test: event_chance naik dgn jarak/scanner & clamp [0,1]; ShipWreck Salvage → +200 iron ke stockpile; AlienCaravan Trade → -500 iron; Nebula expire pd ttl; merchant restock→active+stock, expire→nonaktif+clear; roll tak nyala sebelum interval.
- Assumptions: RNG event **deterministik dari tick** (`derive(tick,"event")`) bukan sumber non-deterministik (`07`) → testable, default aman. Effect terimplementasi: Gain/Lose/AdjustTravelTime/Nothing; **TempProductionBuff ditunda** (butuh field buff di GameState + jalur save). Loot resource → stockpile planet unlocked pertama galaksi aktif (tak ada cargo inventory terpisah). Trade dimodelkan sbg GainResources dgn amount negatif utk resource yg dilepas. Event/merchant transient (save default). **UI panel [LOG/EVENTS] & Merchant view belum dibuat** — model siap, wiring view = polish pasca-DoD (tak ada visual_check khusus event).
- Next: Goal Gate — smoke `cargo run` (G1: stabil, quit `q`) + audit `visual_checks.md` (G10) → tutup PROGRESS bila semua DoD ✅

### iter 35 — 2026-06-29 — M9 / Responsive 3 breakpoint
- Changed: `src/ui/mod.rs` (refactor: `draw_view` kini baca `layout_mode(area)` → dispatch `shell_full`/`shell_compact`/`shell_minimal`; hapus 5 fn `draw_*` duplikat; tambah `view_main` map, `outer_block`, tab bar + footer utk Minimal), `src/ui/panels/resources.rs` (+`render_compact` 1 baris), `tests/snapshots.rs` (+`main_menu_compact` 80×30, +`main_menu_minimal` 60×24); BARU golden `main_menu_80x30.txt`, `main_menu_60x24.txt`; REGEN `planet_view_80x30.txt` (kini Compact)
- Why: M9 item 7 — responsive penuh 3 breakpoint + snapshot tiap ukuran `06`
- Verify: conventions ✓, fmt ✓, clippy -D warnings ✓, test 74 unit + 7 snapshot ✓ → HIJAU
- Snapshot: Full goldens (main_menu/research/warp/galaxy_map 120×40) byte-identik (shell_full = perilaku lama persis). planet_view 80×30 → Compact: sidebar 14 (menu saja, tanpa box SHORTCUTS), panel planet lebih lebar; visual_checks planet_view tetap lulus (header/NODES/FACTORIES/SHIP/RESEARCH/[1..5]/Status). main_menu 80×30 = RES 1 baris + sidebar tipis; 60×24 = tab bar "Menu|Planet|Research|Galaxy|Warp" + main full-width + footer; border utuh, tak ada overflow → visual_checks §Responsiveness lulus.
- Assumptions: strip RESOURCES hanya di main_menu (preserve goldens lama; view lain tak punya strip sejak awal). Compact = sidebar 14 menu-only + RES 1 baris (spec "sidebar tipis, resource ringkas satu baris"). Minimal = tab+footer (spec "tab-based"); truncation ratatui (RES/legend terpotong rapi) dianggap aman, bukan overflow.
- Next: M9 item 8 — Void Merchant + event dasar (queue, resolusi) (`05`,`04`) — item terakhir M9

### iter 34 — 2026-06-29 — M9 / ui/sprite.rs (sprite celestial .ans berwarna)
- Changed: BARU `src/ui/sprite.rs` (`SpriteSize` sm/md/lg + `for_area`/`cells`; `ColorDepth` tc/256/16 + `detect()` via COLORTERM/TERM + `chain()` fallback; `Sprites` cache `HashMap<(base.size.depth), Text>`, `new`/`with_depth`, `ensure` parse `.ans` via `ansi-to-tui` `into_text` dgn fallback depth, `render` pilih size dari Rect + kotak fallback; 6 test headless), `src/ui/mod.rs` (+mod sprite)
- Why: M9 item 6 — sprite celestial `.ans` half-block berwarna, size per-panel + depth per-terminal `07`,`06`
- Verify: conventions ✓, fmt ✓, clippy -D warnings ✓, test 74 unit (6 sprite) + 5 snapshot ✓ → HIJAU
- Snapshot: n/a — golden tak berubah (sprite belum diwire ke view). Test: render planet_ocean 64×32 depth Tc → >100 sel ber-fg & cache=1; render kedua reuse cache; base hilang → fallback tak cache; depth Tc untuk base hanya-`.16` → chain turun, dapat 1.
- Assumptions: layout file = **sub-folder** `sprites/<base>/<size>.<depth>.ans` (sesuai aset nyata + komentar `manifest.ron`), bukan `sprites/<base>.<size>.<depth>.ans` di teks spec `07`. Resize **procedural di Rust** (spec opsi M9 "mulus") DITUNDA — pakai `.ans` statis + clip ratatui dulu (default aman, cukup utk DoD). Belum diwire ke App (integrasi bareng portrait saat view Void Merchant / planet detail).
- Next: M9 item 7 — Responsive penuh 3 breakpoint + snapshot tiap ukuran cocok (`06`)

### iter 33 — 2026-06-29 — M9 / ui/portrait.rs (ratatui-image PNG portrait)
- Changed: BARU `src/ui/portrait.rs` (`Portraits`: Picker + cache `HashMap<id,StatefulProtocol>`; `detect()` query terminal/fallback, `halfblocks()` deterministik; `render()` lazy decode+cache+StatefulImage, kotak fallback bila gagal; `clear()` + 4 test headless), `src/ui/mod.rs` (+mod portrait), `Cargo.toml` (`ratatui-image` → `9`, default-features=false tanpa chafa)
- Why: M9 item 5 — portrait PNG via ratatui-image (Picker + StatefulImage, cache, fallback half-block) `07`,`06`
- Verify: conventions ✓, fmt ✓, clippy -D warnings ✓, test 68 unit (4 portrait) + 5 snapshot ✓ → HIJAU
- Snapshot: n/a — golden tak berubah (portrait belum diwire ke view). Test render PNG 512×512 → half-block 44×32: >100 sel terisi & cache=1 (deterministik headless). missing file → fallback, tak cache. clear → cache kosong.
- Assumptions: **DEP FIX** — spec `ratatui-image="1"` usang: 1.0.x pakai `ratatui>=0.23` → tarik ratatui 0.30/ratatui_core, konflik StatefulWidget dgn ratatui 0.29 kita. Lini untuk ratatui 0.29 = v3–9 (`^0.29.0`) → pin `9` + `default-features=false` (buang `chafa` lib sistem; halfblock/sixel/kitty tetap). Portrait **belum diwire ke App** (Picker `from_query_stdio` query TTY → nondeterministik/headless-risk di snapshot) → integrasi saat Void Merchant view (item terakhir M9), simpan Picker di App via `detect()`.
- Next: M9 item 6 — `ui/sprite.rs` sprite celestial `.ans` half-block berwarna (`ansi-to-tui`), size sm/md/lg + depth tc/256/16 (`07`,`06`)

### iter 32 — 2026-06-29 — M9 / ui/particles.rs (particle system)
- Changed: BARU `src/ui/particles.rs` (`ParticleSystem`/`Emitter`/`Particle`/`ParticleKind`; `update(dt)` spawn fractional + integrasi Euler + cull retain; `burst`; preset `exhaust`/`nebula`; warna fade RGB per kind; render densitas→RAMP_DENSITY + 5 test), `src/ui/mod.rs` (+mod particles), `src/app.rs` (App +`particles`/`exhaust_idx`, demo init exhaust disabled, event_loop: enable saat Traveling + `update(dt)`), `src/ui/panels/main_view.rs` (overlay `particles.render` di atas spiral)
- Why: M9 item 4 — particle dasar (engine exhaust / warp trail) `15`
- Verify: conventions ✓, fmt ✓, clippy -D warnings ✓, test 64 unit + 5 snapshot ✓ → HIJAU
- Snapshot: n/a perubahan — main_menu/lainnya cocok TANPA regen (demo ship Idle → exhaust off → 0 partikel → overlay no-op, golden tak berubah). Efek live hanya di runtime saat Traveling.
- Assumptions: koordinat partikel **dinormalisasi `[0,1]`** (bukan sel mentah, deviasi spec) agar `update` lepas dari ukuran panel; dipetakan ke `area` saat render. `Emitter::new` 8-arg diganti preset per-kind (clippy too_many_args). Warp-trail/sparkle burst tersedia (`burst`) tapi belum dipicu (pemicu Warp Jump/loot = integrasi event lanjutan).
- Next: M9 item 5 — `ui/portrait.rs` render PNG via ratatui-image (Picker + StatefulImage, cache, fallback half-block) (`07`,`06`)

### iter 31 — 2026-06-29 — M9 / ui/galaxy_anim.rs (animasi spiral menu)
- Changed: BARU `src/ui/galaxy_anim.rs` (`GalaxyAnim`: gen bintang lengan+bulge deterministik dari seed, spiral logaritmik `r(θ)=a·e^{bθ}`, rotasi diferensial `ω(r)=ω0/(1+r/r0)`, twinkle, proyeksi polar→grid koreksi `ASPECT`, akumulasi densitas→RAMP glyph, warna per temp core/arm/edge + 2 test), `src/ui/mod.rs` (+mod galaxy_anim), `src/ui/panels/main_view.rs` (header nama galaksi + spiral via `app.anim` + legend, ganti MAP statis), `src/app.rs` (App +`anim`/`anim_secs`, demo init, event_loop set anim_secs dari `start.elapsed()`), golden `main_menu_120x40.txt` (regen — spiral 2-lengan)
- Why: M9 item 3 — animasi spiral galaxy menu utama (`14`)
- Verify: conventions ✓, fmt ✓, clippy -D warnings ✓, test 59 unit + 5 snapshot ✓ → HIJAU
- Snapshot: main_menu@120x40 cocok (deterministik t=0). visual_checks main_menu: RESOURCES 3 tier ✓, MENU entri ✓, MAIN VIEW nama galaksi `Milky Way (Lvl 0)` ✓ (header tetap di atas bidang bintang), fokus ► ✓, SHORTCUTS ✓. Umum: STELLAR IDLE, border utuh, no overflow (spiral di dalam panel) ✓.
- Assumptions: `a` (skala spiral) = `R_CORE`; `r` di-clamp ≥0.005 cegah `ln(0)` di bulge; bulge = `count/6` bintang ekstra. Render via `f.buffer_mut()` set_char+fg. Snapshot pakai `anim_secs=0.0`; animasi nyata hanya di runtime (waktu real). Legend baru (★/✦/· + warna), MAP ASCII statis lama dihapus.
- Next: M9 item 4 — `ui/particles.rs` particle dasar (engine exhaust / warp trail) (`15`)

### iter 30 — 2026-06-29 — M9 / Galaxy Map view (ProcGen + scan) — G5 ✅
- Changed: BARU `src/ui/panels/galaxy_map.rs` (render daftar planet frontier: simbol/nama/biome/jarak/travel-eta/status, legend `■ Planet · ◆ Star · ═ Route`, kursor pilih, hint SCAN/SEND), `src/ui/panels/mod.rs` (+galaxy_map), `src/ui/mod.rs` (route `galaxy_map` + draw_galaxy_map_view), `src/app.rs` (demo +galaksi ProcGen Lvl1 `Drayxar`; `galaxy_map_keys`: j/k pilih, s=scan), `src/game/world/procgen.rs` (+`planet_count`/`galaxy_name`/`materialize_planet` + test), `tests/snapshots.rs` (+galaxy_map), golden `galaxy_map_120x40.txt` (BARU)
- Why: M9 item 2 — Galaxy Map view (planet list ProcGen, scan/send) `06`,`16`
- Verify: conventions ✓, fmt ✓, clippy -D warnings ✓, test 57 unit + 5 snapshot ✓ → HIJAU
- Snapshot: galaxy_map@120x40 cocok. visual_checks galaxy_map(Full): daftar ProcGen 10 planet (nama/biome/jarak) ✓, travel time per planet (kolom `t=`) ✓, legend simbol ✓. Umum: STELLAR IDLE, border utuh, sidebar shortcut g/p/r/m, no overflow ✓.
- Assumptions: Galaxy Map menampilkan galaksi **level tertinggi** (frontier sektor); Milky Way Lvl0 = anchor (tak dieksplor di view ini). `s` = SCAN (materialisasi planet preview → `visited` + push); SEND/travel ke planet outer = integrasi lanjutan (butuh switch active galaxy). G5 ditandai ✅: galaksi luar ProcGen deterministik dari seed + materialisasi==preview (test) + `SaveGalaxy.kind` serialize `Procedural{seed,visited}` (save kecil).
- Next: M9 item 3 — `ui/galaxy_anim.rs` animasi spiral menu utama (`14`)

### iter 29 — 2026-06-29 — M9 / Reorg game/ + game/world/procgen.rs
- Changed: reorg `src/game/world.rs` → `src/game/world/mod.rs` (game/ kembali 9 file datar + sub-folder `world/`, hormati CONVENTIONS §1 ≤10), `+pub mod procgen` di world/mod.rs, BARU `src/game/world/procgen.rs` — galaxy_seed/generate_galaxy (GalaxyMeta), generate_planet (urutan RNG tetap: dist, angle, biome, nodes, anomaly, name), weighted_biome (BIOME_TABLE+level_bias), tier_from_distance, gen_nodes, gen_name (PREFIX/MID/SUFFIX/GREEK) + 4 unit test determinisme
- Why: M9 item 1 — `game/procgen.rs`: generate galaxy/planet deterministik dari seed (`16`)
- Verify: conventions ✓, fmt ✓, clippy -D warnings ✓, test 56 unit + 4 snapshot ✓ → HIJAU
- Snapshot: n/a (logika murni, belum UI; Galaxy Map view = item M9 berikutnya)
- Assumptions: `BIOME_TABLE`/`NODES_FOR_BIOME` di-inline di procgen.rs (terikat enum Biome + id string, bukan const numerik di balance.rs); LavaWorld tak masuk tabel `08`. `level_bias` rare biome +0.15×(level−1). Resource dalam biome: bobot menurun per posisi (4..1). Slot factory ProcGen = `2+tier` (≤8); `unlock_req=WarpTier(tier)`. `GalaxyMeta.tier=level`. angle & anomaly dikonsumsi RNG (jaga urutan) tapi belum ada field di Planet → disimpan untuk Galaxy Map/event berikutnya.
- Next: M9 item 2 — Galaxy Map view (planet list ProcGen, scan/send) `06`,`16`

### iter 28 — 2026-06-29 — M8 / UI Warp Navigation (M8 SELESAI)
- Changed: `src/ui/panels/warp.rs` (BARU: render — current/target galaksi, REQUIREMENT resource+amount [OK]/[--], estimasi Warp Core, warning reset + anchor note, [1] INITIATE WARP JUMP), `src/ui/panels/mod.rs`, `src/ui/mod.rs` (route "warp" + draw_warp_view), `tests/snapshots.rs` (test warp), golden `warp_120x40.txt` (BARU)
- Why: M8 item 3 — UI Warp Navigation + konfirmasi
- Verify: conventions ✓, fmt ✓, clippy ✓, test 53 unit + 4 snapshot ✓ → HIJAU
- Snapshot: warp@120x40 cocok. visual_checks warp(Full): target+requirement (Outer Galaxy Lvl1; Energy 1,240/1,000,000 [--], Titanium 0/500 [--]) ✓, estimasi Warp Core (0) ✓, warning reset + anchor note ✓, [1] INITIATE WARP JUMP ✓. Umum: STELLAR IDLE, border, sidebar shortcut, no overflow ✓.
- Assumptions: (1) Target galaksi dilabeli "Outer Galaxy (Lvl {target})" — nama ProcGen aktual menyusul M9. (2) [1] INITIATE WARP JUMP menampilkan [READY]/[LOCKED] dari meets_threshold; keybinding eksekusi (panggil prestige::warp_jump) belum diwire ke handle_key → wiring + transisi galaksi ProcGen Lvl+1 = M9. (3) G4 ("Warp Jump penuh") tetap false sampai M9 menambah pembuatan/transisi galaksi baru + wiring keybinding.
- Next: M9 — game/procgen.rs (reorg game/ ke sub-folder dulu: 10 file → batas) + galaxy map + animasi + particle + responsive + events

### iter 27 — 2026-06-29 — M8 / anchor feed + permanent upgrade (beli Warp Core)
- Changed: `src/game/prestige.rs` (+anchor_feed, apply_anchor_feed, upgrade_base_cost, permanent_upgrade_cost, buy_permanent_upgrade, buy_anchor_upgrade + 2 test; PrestigeError +InsufficientCores/UnknownUpgrade), `src/sim/tick.rs` (fase 7 → apply_anchor_feed)
- Why: M8 item 2 — anchor feed Milky Way→galaksi aktif + permanent upgrade (sink Warp Core)
- Verify: conventions ✓, fmt ✓, clippy ✓, test 53 unit + 3 snapshot ✓ → HIJAU
- Tests: anchor_feed_formula_and_apply (ANCHOR_BASE×(1+0.5·lvl); feed iron ke galaksi non-anchor, tak ada feed saat aktif=anchor), permanent_and_anchor_upgrades_cost_cores (prod_speed 5→ceil(5·1.5)=8, unknown id, anchor base 10, dana habis Insufficient).
- Assumptions: (1) anchor feed me-nambah resource "iron" (representatif basic; `04` §2 "resource dasar" tak spesifik) ke planet pertama unlocked galaksi aktif, hanya saat aktif≠anchor. (2) permanent upgrade cost = ceil(base·growth^level) Warp Core (base/growth dari `08`: prod_speed 5/1.5, extra_slot 10/2.0, auto_collect 8/1.8, offline_eff 6/1.6). (3) Anchor upgrade cost: `08` tak tetapkan kurva → default geometrik base 10 growth 2.0 (TBD). (4) Efek permanent upgrade (prod_speed mult, extra_slot, dll) ke ekonomi belum diterapkan di tick — level tersimpan; aplikasi efek menyusul (M9 polish/balancing). Fase 7 tick kini memanggil apply_anchor_feed.
- Next: M8 item 3 — UI Warp Navigation + konfirmasi (snapshot warp view)

### iter 26 — 2026-06-29 — M8 / game/prestige.rs (Warp Jump + Warp Core calc)
- Changed: `src/game/prestige.rs` (BARU: production_mult, warp_cores_gain, total_value, warp_threshold, meets_threshold, warp_jump, WarpReport/PrestigeError + 4 test), `src/game/mod.rs` (pub mod prestige)
- Why: M8 item 1 — Warp Jump (reset galaksi aktif, pertahankan prestige/anchor/ship/tech), Warp Core calc
- Verify: conventions ✓, fmt ✓, clippy ✓, test 50 unit + 3 snapshot ✓ → HIJAU
- Tests: cores_and_mult_formulas (mult 1+0.1·lvl; cores floor(WARP_K·sqrt(tv/REF))), warp_jump_resets_active_keeps_persistent (cores>0, level+1, credits/data=0, completed tetap, galaksi aktif non-anchor wiped), warp_jump_threshold_gate (ThresholdNotMet), anchor_not_reset_when_active (anchor=aktif → stockpile tetap).
- Assumptions: (1) warp_threshold: L1 1e6 Energy+500 Titanium, L2 +100 Antimatter, ≥3 ×5 Energy/×4 Titanium/level (Antimatter ×5/level default; rare-resource lanjut TBD `08`). (2) Reset "cash" = credits + research.data + research.active=None; tech completed dipertahankan (`04` §1). (3) Reset ekonomi galaksi aktif (stockpile clear, factory→None, node.level→0) HANYA bila active≠anchor (Milky Way tak pernah reset). (4) Pemindahan ke galaksi ProcGen Lvl+1 + set active baru = M9 (warp_jump iter ini fokus efek persisten + reset); active_galaxy belum dipindah. (5) total_value = credits + Σ(stockpile×base_price) galaksi aktif.
- GUARDRAIL: game/ kini 10 file (batas CONVENTIONS §1). Sebelum tambah game/procgen.rs (M9) → reorg ke sub-folder tematik.
- Next: M8 item 2 — anchor feed Milky Way → galaksi aktif + permanent upgrade (beli pakai Warp Core)

### iter 25 — 2026-06-29 — M7 / sim/offline.rs + round-trip test (M7 SELESAI, G6 ✅)
- Changed: `src/sim/offline.rs` (BARU: offline_efficiency, apply_offline, OfflineReport + 3 test), `src/sim/mod.rs` (pub mod offline), `src/save/mod.rs` (+test save_load_then_offline_progresses)
- Why: M7 item 2 (offline batch+cap+efisiensi) + item 3 (round-trip save→load + offline masuk akal)
- Verify: conventions ✓, fmt ✓, clippy ✓, test 46 unit + 3 snapshot ✓ → HIJAU
- Tests: efficiency_base_and_capped (0.75 base, cap 1.0), applies_batched_income_with_efficiency (100s→75 tick, +75 credit, last_saved update), caps_offline_window (delta>cap → OFFLINE_CAP_SECS), save_load_then_offline_progresses (demo save→load→offline 75 tick).
- Assumptions: (1) eff = OFFLINE_BASE_EFF*(1+OFFLINE_EFF_PER_LVL*level) cap 1.0; level dari prestige.permanent_upgrades["offline_eff"] (default 0). (2) sim_secs = floor(min(delta,cap)*eff) → loop tick::step (event/travel ikut jalan per fase, event auto-queue). (3) apply_offline meng-update last_saved_unix=now. (4) Optimisasi analitik durasi panjang (`07` catatan) = pasca-MVP; MVP loop batch.
- Next: M8 — game/prestige.rs (Warp Jump reset galaksi aktif, Warp Core calc, anchor)

### iter 24 — 2026-06-29 — M7 / save/mod.rs (atomic JSON XDG + id↔string + migrasi)
- Changed: `src/save/dto.rs` (BARU: SaveData DTO id-string + to_save/from_save), `src/save/mod.rs` (BARU isi: save_path XDG, write/write_to atomic, read/read_from, migrate, SaveError + 4 test), `src/game/state.rs` (derive Serialize/Deserialize: PlanetId, ShipStatus, Ship, ResearchState, ActiveResearch, AnchorState, ThemeChoice, GalaxyKind; Biome +Serialize)
- Why: M7 item 1 — save JSON atomic XDG + portabilitas id↔string + migrasi versi
- Verify: conventions ✓, fmt ✓, clippy ✓, test 42 unit + 3 snapshot ✓ → HIJAU
- Tests: save_path_uses_xdg (XDG_DATA_HOME), write_then_read_roundtrips_demo (skalar+galaksi+planet+factory+stockpile+research aktif identik), json_stores_string_ids ("iron" muncul di JSON), version_too_new_rejected.
- Assumptions: (1) Id INSTANCE (Galaxy/Planet/Node/Factory) tetap u32 di save (stabil & self-konsisten dalam satu file); hanya id KONTEN (resource/item/recipe/building) → string utk portabilitas (`07` §Save↔Content). (2) `merchant`+`events` (transient, M9) TIDAK disimpan → default saat load; catat. (3) Id konten hilang saat load (konten dihapus) → skip entri (node/factory/stockpile/blueprint/autosell), UnlockReq::Resource yang hilang → None (gerbang terbuka, aman). (4) Migrasi: kerangka `migrate(Value, version)` no-op (hanya v1); VersionTooNew bila save > SAVE_VERSION. (5) Round-trip test demo sudah ada di sini; item 3 (round-trip + offline) menambah cek offline setelah item 2.
- Next: M7 item 2 — sim/offline.rs (time-delta batch, OFFLINE_CAP, efisiensi)

### iter 23 — 2026-06-29 — M6 / travel time + unlock body Sol (M6 SELESAI)
- Changed: `src/game/world.rs` (BARU: loader milky_way.ron → Galaxy, DTO Deserialize + konversi ResourceId, WorldError + 2 test), `src/game/ship.rs` (travel_to + TravelError + 2 test), `src/game/state.rs` (Planet +field distance:f64; Biome derive Deserialize), `src/game/mod.rs` (pub mod world), +distance pada 7 literal Planet (economy/research/actions/app/tick×3)
- Why: M6 item 2 — travel time + unlock body Sol. Status Traveling di UI sudah ada di planet.rs (branch ShipStatus::Traveling). Arrival-unlock sudah ada di tick::advance_travel.
- Verify: conventions ✓, fmt ✓, clippy ✓, test 38 unit + 3 snapshot ✓ → HIJAU
- Tests: world loads_sol_system (24 body, Earth unlocked 6 node/6 slot dist 1.0, Luna WarpTier(1) locked), unknown_resource_rejected; ship travel_starts_and_is_busy (tier1→Luna travel, lalu Busy), travel_gate_and_already_unlocked (tier0→Locked, Earth→AlreadyUnlocked).
- Assumptions: (1) distance jadi field Planet (body property per `09`/`03` §4) — churn 7 literal test/demo (distance default 1.0). (2) Loader Milky Way TIDAK diwire ke App::demo()/run() iterasi ini agar snapshot planet_view/research (Earth handcrafted) tak berubah; wiring galaxy nyata + keybinding travel → ditunda ke UI galaxy_map (M9) atau langkah wiring khusus. (3) travel_to gate via meets_unlock_req; have(res)=total stockpile resource di planet unlocked galaksi aktif (Sol hanya pakai WarpTier/None). (4) stockpile_cap body = BASE_CAP (cargo buff diterapkan saat dibutuhkan; `03` §3b cap = BASE_CAP×(1+...)).
- Next: M7 — save/mod.rs (JSON atomic XDG, id↔string, migrasi) + sim/offline.rs + round-trip test

### iter 22 — 2026-06-29 — M6 / game/ship.rs (warp tier gatekeeping + part buff)
- Changed: `src/game/ship.rs` (BARU: meets_unlock_req, travel_secs, stockpile_cap, yield_mult, event_chance, part_level, upgrade_part, ShipError + 4 test), `src/game/mod.rs` (pub mod ship)
- Why: M6 item 1 — ship sebagai gatekeeper Warp Tier + buff pasif part (engine/cargo/scanner)
- Verify: conventions ✓, fmt ✓, clippy ✓, test 34/34 ✓ → HIJAU
- Tests: warp_tier_gates_access (WarpTier ≥ gate), resource_and_all_req (UnlockReq::Resource + All rekursif), engine_reduces_travel_cargo_raises_cap (ENGINE_FACTOR<1, cap linear, event_chance naik thd scanner), upgrade_part_pays_geometric (engine ×GROWTH, shield NotUpgradable, dana habis Insufficient).
- Assumptions: (1) meets_unlock_req generik atas closure `have(res)` agar bisa dipakai travel (M6 item 2) tanpa mengikat sumber resource (stockpile/inventory ditentukan caller). (2) upgrade_part hanya engine/cargo/scanner (kurva di `08`); shield/cloaking → NotUpgradable sampai balancing menetapkan biaya. (3) Biaya dibayar Credits saja (COST_SHIP_* dari balance, geometrik ×GROWTH^level) — `03` §3b menyebut "resource T2/T3" tapi `08` baseline pakai Credits; catat utk tuning. (4) event_chance pakai konstanta `05`/`08` (BASE_EVENT_CHANCE × dist × scanner); konsumsi roll event → M6 item 2/M9.
- Next: M6 item 2 — travel time + unlock body Sol (loader milky_way.ron, aksi travel, status Traveling di UI)

### iter 21 — 2026-06-29 — M5 / UI research terminal (M5 SELESAI)
- Changed: `src/ui/panels/research.rs` (BARU: render — Data/sec+total, riset aktif progress_bar [####..], AVAILABLE TECHS list cost+time), `src/ui/panels/mod.rs`, `src/ui/mod.rs` (route "research" + draw_research_view sidebar menu+shortcuts), `src/app.rs` (demo: research aktif manu_steel 60%), `tests/snapshots.rs` (test research), golden `research_120x40.txt` (BARU)
- Why: M5 item 3 — UI research terminal (available/current + progress bar)
- Verify: conventions ✓, fmt ✓, clippy ✓, test 30 unit + 3 snapshot ✓ → HIJAU
- Snapshot: research@120x40 → cocok. visual_checks research(Full): header Data/sec+total ✓, AVAILABLE TECHS≥1 (ext_efficiency cost 4,000/600s) ✓, progress bar aktif `[######....] 60% (48s left)` ✓. Umum: STELLAR IDLE ✓, border utuh ✓, sidebar shortcut g/p/r/m ✓, no overflow ✓.
- Assumptions: (1) Progress bar pakai fraksi pembatas = min(data_frac, time_frac) karena completion butuh Data DAN waktu; "Ns left" dari sisa waktu. (2) AVAILABLE techs = is_available && bukan tech aktif; demo manu_steel aktif → list sisakan ext_efficiency. (3) Demo state diberi research.active agar snapshot menampilkan progress bar (tak mengubah golden main_menu/planet_view yang tak membaca research.active). (4) Keybinding j/k/Enter di research view (pilih+mulai riset) belum diwire ke app.handle_key — footer informatif; wiring aksi research → iterasi UI lanjut/M9.
- Next: M6 — game/ship.rs (warp tier gatekeeping, part buff) + travel time/unlock body Sol

### iter 20 — 2026-06-29 — M5 / tech tree efek (recipe/building unlock gating)
- Changed: `src/game/research.rs` (+recipe_locked_by, building_locked_by, recipe_available, building_available), `src/game/actions.rs` (build_factory: gate building via UnlockBuilding + recipe via Recipe tech; ActionError::TechLocked + 2 test)
- Why: M5 item 2 — efek tech tree lengkap. WarpTier+multiplier sudah di iter19; iter ini melengkapi konsumsi Recipe + UnlockBuilding.
- Verify: conventions ✓, fmt ✓, clippy ✓, test 30/30 ✓ → HIJAU
- Tests: tech_locked_building_then_unlocked (deep_core_miner → TechLocked, lalu Insufficient setelah ext_deep_core), tech_locked_recipe_then_unlocked (steel_mill → TechLocked, lalu build sukses setelah manu_steel).
- Assumptions: (1) Recipe/UnlockBuilding di-gate saat **build-time** (actions::build_factory pilih recipe/building), BUKAN di loop refinery per-tick — `02` §3 eksplisit hanya gate `requires_blueprint`; menambah gate tech di loop akan kontradiksi spec. Refinery yang sudah berjalan tak dicek ulang. (2) Recipe tech-locked = yang dirujuk `TechUnlock::Recipe` (steel_mill→manu_steel, alloy_forge→manu_titanium); sisanya bebas. Konsekuensi: steel production butuh research manu_steel dulu (sesuai G3 progression). (3) AccessOuterTier masih hanya tercatat di `completed`; konsumsi akses galaksi → M6/M9.
- Next: M5 item 3 — UI research terminal (available/current + progress bar)

### iter 19 — 2026-06-29 — M5 / game/research.rs (Data/sec, alokasi tech, completion)
- Changed: `src/game/research.rs` (BARU: data_rate, is_available, available_techs, start_research, advance, apply_unlock, tech_mult, ResearchError + 5 test), `src/game/mod.rs` (pub mod research), `src/sim/tick.rs` (extractor pakai research::tech_mult; fase 5b → research::advance; TickReport.research_completed)
- Why: M5 item 1 — Data/sec dari ResearchLab, alokasi Data ke satu tech aktif, completion + efek
- Verify: conventions ✓, fmt ✓, clippy ✓, test 28/28 ✓ → HIJAU
- Tests: data_rate_sums_labs (3×5=15/s), start_respects_prereq_and_single_active (PrereqNotMet + AlreadyActive), completes_when_data_and_time_met (manu_steel 500 Data/120s), warp_tier_applied_on_completion (aero_warp_mk2 → ship.warp_tier=6), tech_mult_from_completed (ext_efficiency +0.25 iron).
- Assumptions: (1) Model pool — Data terkumpul di research.data, tiap tick dialokasikan dari pool ke active.data_invested (cap data_cost); completion = data_invested≥data_cost AND elapsed≥time_secs. (2) apply_unlock: WarpTier langsung ke ship (max); TechMult diturunkan on-the-fly via tech_mult(); Recipe/UnlockBuilding/AccessOuterTier cukup tercatat di completed (gate building/recipe/galaksi dikonsumsi modul M6+). (3) tech_mult menerima &completed terpisah agar bisa dipanggil di dalam loop mutasi galaksi tanpa konflik borrow. (4) Fase 5a tick (akumulasi data_rate inline) dipertahankan; research::data_rate duplikat tersedia utk UI.
- Next: M5 item 2 — tech tree dari data + verifikasi semua varian efek (recipe/multiplier) terpakai; lalu item 3 UI research terminal

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
