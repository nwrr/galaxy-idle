# PROGRESS

> Ringkasan state terbaca-manusia. **Agent meng-update file ini tiap iterasi.** Untuk state
> machine-readable lihat [`state.json`](state.json); log detail di [`ITERATION_LOG.md`](ITERATION_LOG.md).

## Status Saat Ini — ✅ SELESAI (semua DoD tercapai)

- **Milestone:** DONE — semua M0–M9 ✅
- **Iterasi:** 37 (loop dihentikan atas permintaan user setelah Goal Gate)
- **Verify terakhir:** HIJAU (conventions+fmt+clippy+82 unit+7 snapshot+2 smoke) — iter 37
- **Blocker:** tidak ada
- **DoD G1–G10:** semua ✅ (lihat `GOALS.md`).

### Ringkasan Rilis

Galaxy-Idle = game idle TUI bertema galaksi (no-combat) di ratatui, dibangun lewat 37 iterasi loop:
- **Sim & ekonomi (M2):** tick 9-fase (produksi→craft→upkeep→auto-sell→research→travel/event→anchor→
  merchant), extractor/refinery/market.
- **Konten data-driven (M1):** `data/*.ron` (resource/item/recipe/building/tech) → `Content` + validasi.
- **Progресi (M5–M6):** research tree → Warp Tier → unlock body Sol; ship travel + buff part.
- **Save/offline (M7):** JSON atomic XDG, dto id↔string, offline batch round-trip.
- **Prestige (M8):** Warp Jump (reset + Warp Core + anchor permanen).
- **ProcGen + polish (M9):** galaksi luar deterministik dari seed, Galaxy Map, animasi spiral,
  particle, portrait (`ratatui-image`), sprite celestial `.ans` berwarna, responsif 3 breakpoint,
  Void Merchant + event (queue/resolusi).
- **UI:** `draw_view` responsif (Full/Compact/Minimal); 5 view + 7 golden snapshot; visual_checks lulus.

### Backlog pasca-DoD (opsional, di luar scope loop)

- Panel `[ LOG / EVENTS ]` + Merchant view (model events/merchant sudah siap, belum diwire ke view).
- Integrasi `ui::sprite` & `ui::portrait` ke view planet/merchant (modul standalone+tested).
- `EventEffect::TempProductionBuff` (butuh field buff transient di `GameState` + fase 9 buff-expiry).
- Tuning balancing halus (baseline `08` dipakai apa adanya).

## Ringkasan per Milestone

| Milestone | Status |
|-----------|--------|
| M0 Scaffold | ✅ selesai (deps, skeleton, rng, balance) |
| M1 Content Loader | ✅ selesai (data/*.ron + content.rs + validasi) |
| M2 Core Sim | ✅ selesai (state, economy, tick 9-fase) |
| M3 TUI Shell | ✅ selesai (loop, layout, theme, panels, snapshot) |
| M4 Planet & Build | ✅ selesai (panel, snapshot, aksi, keybinding) |
| M5 Research | ✅ selesai (research.rs + tech efek + UI research terminal) |
| M6 Ship & Travel | ✅ selesai (ship.rs gatekeep+buff, world loader Sol, travel_to) |
| M7 Save/Offline | ✅ selesai (save dto id↔string, offline batch, round-trip) |
| M8 Prestige | ✅ selesai (warp jump, anchor feed, permanent upgrade, UI warp) |
| M9 ProcGen + Polish | ✅ item fitur selesai (procgen + Galaxy Map + galaxy_anim + particles + portrait + sprite + responsive + Void Merchant/event ✓); sisa Goal Gate (G1 smoke, G10 audit) |

Legenda: ⬜ belum · 🟦 sedang · ✅ selesai · ⛔ blocker

## Catatan / Asumsi Terbuka

- **`game/` >10 file vs CONVENTIONS §1:** RESOLVED (iter 29). `world.rs` (loader Sol) dipindah ke
  sub-folder `game/world/mod.rs`, `procgen.rs` masuk `game/world/` → `game/` = 9 file datar + 1
  sub-folder. Bila nanti perlu file ke-10 lagi, kelompokkan tema berikut (mis. `game/econ/`).

## Blocker Aktif (jika ada)

_(kosong — bila loop berhenti karena blocker, jelaskan di sini: apa, akar masalah, opsi)_
