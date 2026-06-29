# PROGRESS

> Ringkasan state terbaca-manusia. **Agent meng-update file ini tiap iterasi.** Untuk state
> machine-readable lihat [`state.json`](state.json); log detail di [`ITERATION_LOG.md`](ITERATION_LOG.md).

## Status Saat Ini

- **Milestone aktif:** M5 — Research (🟦 mulai)
- **Iterasi:** 18
- **Item berikutnya:** `game/research.rs` Data/sec + alokasi tech + completion (M5 item 1)
- **Verify terakhir:** HIJAU (conventions+fmt+clippy+25 test) — iter 18 (M4 SELESAI)
- **Blocker:** tidak ada

## Ringkasan per Milestone

| Milestone | Status |
|-----------|--------|
| M0 Scaffold | ✅ selesai (deps, skeleton, rng, balance) |
| M1 Content Loader | ✅ selesai (data/*.ron + content.rs + validasi) |
| M2 Core Sim | ✅ selesai (state, economy, tick 9-fase) |
| M3 TUI Shell | ✅ selesai (loop, layout, theme, panels, snapshot) |
| M4 Planet & Build | ✅ selesai (panel, snapshot, aksi, keybinding) |
| M5 Research | ⬜ belum |
| M6 Ship & Travel | ⬜ belum |
| M7 Save/Offline | ⬜ belum |
| M8 Prestige | ⬜ belum |
| M9 ProcGen + Polish | ⬜ belum |

Legenda: ⬜ belum · 🟦 sedang · ✅ selesai · ⛔ blocker

## Catatan / Asumsi Terbuka

- **`game/` >10 file vs CONVENTIONS §1:** layout `07` punya 11 file datar di `game/`. Saat
  dipopulasi akan dikelompokkan ke sub-folder tematik (mis. `game/world/`, `game/econ/`) agar
  ≤10/folder. Diputuskan saat modul terkait diimplementasi (M2+), bukan sekarang.

## Blocker Aktif (jika ada)

_(kosong — bila loop berhenti karena blocker, jelaskan di sini: apa, akar masalah, opsi)_
