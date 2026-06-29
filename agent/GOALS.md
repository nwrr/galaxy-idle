# GOALS — Idea → Goal

## Ide (satu kalimat)

Game **idle TUI bertema galaksi, tanpa combat**: bangun ekonomi produksi yang scaling, eksplorasi
galaksi prosedural, riset, dan prestige "warp jump" — semua di terminal dengan UI teks + ANSI
berwarna responsif (sprite celestial berwarna, portrait karakter via ratatui-image).

## Goal Akhir (Definition of Done — project dianggap SELESAI bila SEMUA terpenuhi)

- [x] **G1** `cargo run` membuka TUI ratatui yang stabil (tidak panic), keluar bersih dengan `q`.
- [x] **G2** Economy loop jalan: extractor → refinery → market/auto-sell, net income real-time.
- [x] **G3** Progression: research tech tree menaikkan Warp Tier → membuka body Sol system.
- [x] **G4** Prestige: satu Warp Jump penuh berfungsi (reset galaksi aktif, dapat Warp Core, anchor tetap).
- [x] **G5** ProcGen: minimal 1 galaksi luar ter-generate deterministik dari seed (save kecil).
- [x] **G6** Save/Load + offline progress (JSON atomic, XDG) round-trip benar.
- [x] **G7** UI responsif 3 breakpoint (Full/Compact/Minimal) + animasi galaxy + particle dasar.
- [x] **G7b** Portrait karakter dirender sebagai gambar via `ratatui-image` (fallback half-block).
- [x] **G7c** Sprite celestial = ANSI half-block **berwarna** (`.ans`), biome jelas beda; runtime
      pilih ukuran (sm/md/lg) per panel & depth (tc/256/16) per kapabilitas terminal.
- [x] **G8** Konten data-driven dari `data/*.ron` (resource/item/recipe/building) ter-load & tervalidasi.
- [x] **G9** `scripts/verify.sh` hijau: `fmt` + `clippy -D warnings` + semua test + snapshot cocok.
- [x] **G10** Semua `agent/test/visual_checks.md` lulus pada snapshot golden.

## Milestone (urutan eksekusi loop)

| ID | Milestone | Selesai bila |
|----|-----------|--------------|
| **M0** | Scaffold | `Cargo.toml` deps (ratatui, crossterm, serde, serde_json, ron), `cargo build` ok, modul kosong sesuai `07` |
| **M1** | Content loader | `data/*.ron` (dari katalog `09`–`13`) ter-load ke `Content`, `validate()` lulus, test loader hijau |
| **M2** | Core sim tick | `sim::tick::step` (9 fase) + economy (extractor/refinery/auto-sell) lulus unit test |
| **M3** | TUI shell | App loop + render frame full (banner, panel RESOURCES/MENU/MAIN/SHORTCUTS), quit `q`, snapshot main_menu cocok |
| **M4** | Planet & build | Panel planet, build/upgrade factory & node, navigasi keybinding; snapshot planet_view cocok |
| **M5** | Research | ResearchLab → Data → tech node → Warp Tier; UI research |
| **M6** | Ship & travel | Travel time, unlock body Sol system via Warp Tier |
| **M7** | Save/offline | Save JSON atomic + load + offline progress; round-trip test |
| **M8** | Prestige | Warp Jump penuh + anchor + Warp Core |
| **M9** | ProcGen + polish | Galaksi luar deterministik, animasi galaxy + particle, responsive 3 breakpoint |

DoD G-list terpetakan: G1/G7→M3,M9 · G2→M2 · G3→M5,M6 · G4→M8 · G5→M9 · G6→M7 · G8→M1 · G9/G10→semua.

## Non-Goals (jangan dikerjakan loop tanpa permintaan)

- Multiplayer / networking.
- Combat / fleet battle (game ini no-combat by design).
- Image-to-terminal (sixel/kitty) — runtime tetap teks (lihat `assets/README.md`).
- Balancing "sempurna" — pakai baseline `08-balancing.md`; tuning halus = pasca-DoD.
