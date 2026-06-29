# M03 — Capture terminal nyata (audit + galaxy pixel)

- **id:** M03 · **kategori:** A-foundation · **status:** todo
- **depends_on:** M02
- **gerbang:** T (capture_term audit)
- **DoD:** [GOALS](../../GOALS.md) U-A

## Tujuan
Sediakan jalur screenshot dari terminal sungguhan (warna/font asli + grafis sixel/kitty yang TIDAK
muncul di rasterizer sel). Dipakai audit periodik & wajib untuk verifikasi **galaxy pixel** (D).

## File disentuh
- `scripts/capture_term.sh` (baru)
- output: `agent/test/screens/term/`

## Checklist
- [ ] M03.1 Deteksi terminal grafis tersedia (kitty / wezterm / tmux + emulator) — (T)
- [ ] M03.2 Jalankan binari TUI asli (`cargo run`) ukuran kolom×baris tetap, headless/offscreen — (T)
- [ ] M03.3 Tangkap screenshot OS frame pertama stabil → `agent/test/screens/term/<view>_<w>x<h>.png` — (T)
- [ ] M03.4 Skip-aman + pesan jelas bila tak ada terminal grafis (JANGAN gagalkan verify.sh) — (T)
- [ ] M03.5 Audit: bandingkan capture_term main_menu vs rasterizer PNG → warna/glyph konsisten — (T)

## Selesai bila
`scripts/capture_term.sh <view> <w> <h>` menghasilkan PNG dari terminal nyata (atau skip rapi),
dan konsisten dgn rasterizer untuk view non-grafis.

## Verifikasi
`scripts/capture_term.sh main_menu 120 40` → Read PNG (bila tersedia). Tak memblok verify.sh saat skip.

## Referensi
`VISUAL_TARGETS.md` §Catatan (rasterizer vs sixel). Dipakai lagi di M17/M20 (galaxy pixel).
