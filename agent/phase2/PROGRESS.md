# PROGRESS — Phase 2: UI/UX Design

> Ringkasan terbaca-manusia. Agent meng-update tiap iterasi. State mesin: [`state.json`](state.json);
> log detail: [`ITERATION_LOG.md`](ITERATION_LOG.md). Phase 1 (build) selesai → [`../phase1/PROGRESS.md`](../phase1/PROGRESS.md).

## Status Saat Ini — 🟦 MULAI (belum ada iterasi Phase 2)

- **Milestone:** M01 — Rasterizer Buffer→PNG (kat A Foundation)
- **Iterasi:** 0 (loop belum dijalankan; kode warisan Phase 1, verify HIJAU)
- **Item berikut:** M01.1 — tambah dep `ab_glyph` + `cargo build` ok
- **Verify terakhir:** HIJAU (Phase 1 iter 37)
- **Blocker:** tidak ada
- **DoD U-A..U-J:** semua ⬜ (lihat `GOALS.md`). **28 milestone** dalam 6 kategori, 1 file/milestone
  di [`milestones/`](milestones/) (~280 item); indeks status di `CHECKLIST.md`.

## Plan Disetujui

Arsitektur Phase 2 terkunci (4 keputusan user) di `GOALS.md` §Keputusan Arsitektur + plan file
`~/.claude/plans/playful-wondering-castle.md`: Hybrid TUI+pixel-galaxy · screenshot Buffer→PNG +
capture terminal · ComfyUI gate ≥90 (verify.py) · ANSI ≥99 · galaxy hybrid (procedural+starmap+pixel).

## Konteks Awal (backlog UI/UX dari Phase 1)

Phase 1 menutup mekanik tapi menyisakan UI/UX (lihat `../phase1/PROGRESS.md` §Backlog):
- `ui::sprite` & `ui::portrait` = modul standalone + tested, **belum diwire** ke view.
- Panel `LOG / EVENTS` + Merchant view belum ada (model `events`/`merchant` sudah siap).
- Sprite celestial `.ans` berwarna sudah digenerate (11 base × 9 varian) di `assets/sprites/`.
- Responsif 3 breakpoint sudah jalan secara struktur; kualitas visual perlu pass.

## Alat Phase 2

- **Generator asset:** `comfyui/` (portrait PNG, lokal Z-Image Turbo / cloud Flux) + `scripts/gen_assets.py`.
- **ANSI converter:** `scripts/genassets/ansi.py` (half-block truecolor + fallback 256/16).
- **Cek asset:** `scripts/check_assets.py`. **Cek visual UI:** `scripts/snapshot.sh` + `../test/visual_checks.md`.

## Ringkasan per Milestone

| Kategori | Milestone | Status |
|----------|-----------|--------|
| A Foundation | M01–M04 (rasterizer, screenshot, capture, wire) | ⬜ belum |
| B Asset Pipeline | M05–M11 (comfyui, verify ≥90, png→ansi ≥99) | ⬜ belum |
| C Shell & Core | M12–M15 (shell, planet, research) | ⬜ belum |
| D Galaxy ★ | M16–M20 (backdrop, pixel, starmap) | ⬜ belum |
| E Secondary | M21–M23 (warp, merchant, events) | ⬜ belum |
| F Polish/Verify | M24–M28 (theme, particle, responsive, sign-off) | ⬜ belum |

Detail per milestone: [`milestones/README.md`](milestones/README.md).

Legenda: ⬜ belum · 🟦 sedang · ✅ selesai · ⛔ blocker

## Catatan / Asumsi Terbuka

- Snapshot `buffer_to_text` **membuang warna** → verifikasi warna sprite/theme tak bisa lewat golden
  teks saja. Strategi: golden untuk **layout/struktur**, plus cek `.ans` langsung (`cat` di terminal
  truecolor) + assertion deskriptif di `visual_checks.md` (lihat `LOOP.md` §Verify).

## Blocker Aktif (jika ada)

_(kosong)_
