# M26 — Responsive pass (3 breakpoint × semua view)

- **id:** M26 · **kategori:** F-polish-verify · **status:** todo
- **depends_on:** M14, M15, M20, M21, M22, M23
- **gerbang:** S, T
- **DoD:** [GOALS](../../GOALS.md) U-F

## Tujuan
Pastikan SETIAP view bersih di Full/Compact/Minimal: tak ada overflow/clipping, boundary tak glitch,
sprite auto-size.

## File disentuh
- `src/ui/mod.rs`, `src/ui/panels/*` (perbaikan responsif), `src/ui/layout.rs`

## Checklist
- [ ] M26.1 main_menu Full/Compact/Minimal bersih — (S)
- [ ] M26.2 planet_view 3 breakpoint bersih — (S)
- [ ] M26.3 research 3 breakpoint bersih — (S)
- [ ] M26.4 galaxy_map 3 breakpoint bersih — (S)
- [ ] M26.5 warp 3 breakpoint bersih — (S)
- [ ] M26.6 merchant 3 breakpoint bersih — (S)
- [ ] M26.7 events 3 breakpoint bersih — (S)
- [ ] M26.8 Tak ada overflow/clip di ukuran ekstrem kecil — (S)
- [ ] M26.9 Boundary tepat 70/100w tak glitch — (S)
- [ ] M26.10 Sprite size auto turun saat sempit — (S)
- [ ] M26.11 Menu kanan auto-ringkas di Compact — (S)
- [ ] M26.12 Footer/status bar tetap muat — (S)
- [ ] M26.13 Resize dinamis (besar↔kecil) tak panic — (T)
- [ ] M26.14 Golden snapshot semua view × 3 ukuran lengkap — (V)
- [ ] M26.15 Screenshot grid semua view × 3 ukuran diarsip — (S)
- [ ] M26.16 `visual_checks.md` responsif lengkap — (S)

## Selesai bila
Semua view bersih di 3 breakpoint (no overflow/glitch); golden lengkap; arsip screenshot grid ada.

## Verifikasi
`scripts/screenshot.sh <view> <w> <h>` tiap kombinasi; resize manual `cargo run`; `verify.sh` hijau.

## Referensi
`06-ui.md` §Responsive. `src/ui/layout.rs` (`layout_mode`).
