# M04 — Wire asset ke App + audit baseline

- **id:** M04 · **kategori:** A-foundation · **status:** todo
- **depends_on:** M02
- **gerbang:** V, U, S
- **DoD:** [GOALS](../../GOALS.md) U-A, U-B

## Tujuan
Buka jalan agar semua view bisa memakai asset: `App` memegang `Sprites` + `Portraits` (sudah ada,
tested, tapi nganggur). Plus tulis VISUAL_TARGETS & audit UX baseline (PNG tiap view) sebagai acuan
perbaikan kategori C–F.

## File disentuh
- `src/app.rs` (field `sprites`, `portraits`, helper)
- `src/ui/sprite.rs`, `src/ui/portrait.rs` (akses runtime; init headless-safe)
- `agent/phase2/VISUAL_TARGETS.md` (sudah ada — verifikasi/lengkapi)
- `agent/phase2/UX_REVIEW.md` (baru)

## Checklist
- [ ] M04.1 `App` simpan `sprites: ui::sprite::Sprites` (lazy; depth detect; `with_depth` utk test) — (V,U)
- [ ] M04.2 `App` simpan `portraits` (Picker/Portraits) init aman tanpa panic headless — (V,U)
- [ ] M04.3 Mode deterministik asset utk snapshot (depth tetap, no I/O acak) — (U)
- [ ] M04.4 `App::sprite_for_biome(Biome)->&str` (peta biome→base sprite) — (U)
- [ ] M04.5 Helper id portrait per konteks (captain/merchant) deterministik — (U)
- [ ] M04.6 Render uji sprite di area dummy (planet_view) → tampil di PNG — (S)
- [ ] M04.7 Lengkapi `VISUAL_TARGETS.md` (peta view→referensi + kriteria layak) — (V)
- [ ] M04.8 `UX_REVIEW.md`: audit tiap view dari baseline PNG (M02) → daftar defisiensi + target — (S)
- [ ] M04.9 `verify.sh` hijau penuh dgn field asset baru (fmt/clippy/test/snapshot) — (V)

## Selesai bila
`App` memegang sprites+portraits siap dipakai panel; snapshot tetap deterministik & hijau; UX_REVIEW
mencatat defisiensi tiap view sebagai backlog kategori C–F.

## Verifikasi
`cargo test`; `scripts/screenshot.sh planet_view 120 40` → sprite uji tampil; baca UX_REVIEW.

## Referensi
`src/ui/sprite.rs` (`Sprites`), `src/ui/portrait.rs`. `06-ui.md`.
