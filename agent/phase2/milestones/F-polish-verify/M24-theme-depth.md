# M24 — Theme & color depth

- **id:** M24 · **kategori:** F-polish-verify · **status:** todo
- **depends_on:** M13
- **gerbang:** S, U
- **DoD:** [GOALS](../../GOALS.md) U-H

## Tujuan
Theme konsisten & terbaca di ketiga palet (Default/HighContrast/Mono) dan ketiga depth (tc/256/16),
dengan switch live.

## File disentuh
- `src/ui/theme.rs`, `src/ui/mod.rs`, `src/app.rs` (key switch), `src/ui/raster.rs` (hormati bg)

## Checklist
- [ ] M24.1 Palet Default disempurnakan (header/border/tier/dim/focus) — (S)
- [ ] M24.2 HighContrast: semua elemen terbaca — (S)
- [ ] M24.3 Mono: terbaca tanpa warna — (S)
- [ ] M24.4 Theme switch live (key) tanpa restart — (U,S)
- [ ] M24.5 Rasterizer hormati theme bg (screenshot per theme benar) — (S)
- [ ] M24.6 Depth tc/256/16: sprite tetap layak tiap depth — (S)
- [ ] M24.7 Konsistensi warna lintas-view — (S)
- [ ] M24.8 Cursor/selection style seragam — (S)
- [ ] M24.9 Golden + screenshot 3 theme (main_menu) → semua layak — (V,S)

## Selesai bila
Ketiga theme & depth terbaca dan konsisten lintas-view; switch live; screenshot per theme disetujui.

## Verifikasi
`scripts/screenshot.sh main_menu 120 40 high_contrast` / `mono`; `verify.sh` hijau.

## Referensi
`06-ui.md` §Theme. `src/ui/theme.rs`, `src/game/state.rs` (`ThemeChoice`).
