# M02 — Screenshot bin + script + baseline

- **id:** M02 · **kategori:** A-foundation · **status:** todo
- **depends_on:** M01
- **gerbang:** V, S
- **DoD:** [GOALS](../../GOALS.md) U-A

## Tujuan
Bungkus rasterizer (M01) jadi alat pakai: binari headless render view → PNG, plus skrip pembungkus.
Hasil PNG dibaca agent (Claude vision) untuk menilai desain & iterasi.

## File disentuh
- `src/bin/screenshot.rs` (baru)
- `scripts/screenshot.sh` (baru)
- `agent/test/README.md` (dok cara pakai)
- output: `agent/test/screens/`

## Checklist
- [ ] M02.1 `bin/screenshot.rs`: arg `<view> <w> <h> [theme] <out.png>`; render `draw_view` ke
      `TestBackend` ukuran w×h — (V)
- [ ] M02.2 Pakai `buffer_to_rgba` → tulis PNG ke `out.png`; exit-code jelas bila view tak dikenal — (V)
- [ ] M02.3 Dukung arg `theme` (default/high_contrast/mono) → set `App.settings.theme` deterministik — (V)
- [ ] M02.4 `scripts/screenshot.sh <view> <w> <h> [theme]` → `agent/test/screens/<view>_<w>x<h>[_theme].png` — (V)
- [ ] M02.5 Smoke: screenshot `main_menu 120 40` → PNG valid; **Read PNG** terbaca jelas — (S)
- [ ] M02.6 Loop semua view (main_menu/planet_view/research/galaxy_map/warp) → arsip
      `agent/test/screens/baseline/` — (S)
- [ ] M02.7 Dok di `agent/test/README.md`: cara pakai + batas (tak menampilkan sixel) — (V)

## Selesai bila
`scripts/screenshot.sh <view> <w> <h>` menghasilkan PNG berwarna yang bisa di-Read; baseline tiap
view tersimpan; agent bisa menilai (acuan UX_REVIEW di M04).

## Verifikasi
`scripts/screenshot.sh main_menu 120 40` → Read PNG hasil. `verify.sh` hijau (bin ikut clippy/build).

## Referensi
`VISUAL_TARGETS.md`. Pola headless mirip `src/bin/snapshot.rs` + `tests/snapshots.rs`.
