# M10 — ANSI fidelitas ≥99 + varian responsive

- **id:** M10 · **kategori:** B-asset-pipeline · **status:** todo
- **depends_on:** M09
- **gerbang:** A, S
- **DoD:** [GOALS](../../GOALS.md) U-D

## Tujuan
Pastikan konversi ANSI **detail & bagus skor ≥99%** (permintaan user) dan **responsive**: tiap base
punya 9 varian (sm/md/lg × tc/256/16). Skor = re-render `.ans`→RGB lalu SSIM vs sumber-downscale.

## File disentuh
- `scripts/genassets/` (scorer fidelitas + decoder `.ans`→RGB)
- `scripts/gen_assets.py` (emit 9 varian per `convert`)

## Checklist
- [ ] M10.1 Decoder `ans_to_rgb(lines)` (kebalikan half-block) untuk scoring — (U)
- [ ] M10.2 `fidelity(src_img, ans, cols, rows)` = SSIM(re-render, src-downscale) 0–100 — (U)
- [ ] M10.3 Target ≥99 (tc) didokumentasikan + ambang per depth (256/16 lebih longgar) — (A)
- [ ] M10.4 Tuning sampling/dither sampai tc ≥99 di gambar uji — (A)
- [ ] M10.5 `convert` emit 9 varian `sprites/<base>/<size>.<depth>.ans` (sm 20×10, md 40×20, lg 64×32) — (A)
- [ ] M10.6 Skor fidelitas tiap varian dicatat (laporan) — (A)
- [ ] M10.7 Varian sm/md/lg semua terbaca (no overflow box) saat dirender — (S)
- [ ] M10.8 Depth 256/16 fallback tetap terbaca (screenshot/cat) — (S)

## Selesai bila
`convert` menghasilkan 9 varian/base dgn fidelitas tc ≥99; varian kecil tetap terbaca; laporan skor
tersimpan.

## Verifikasi
`gen_assets.py convert <png> <base>` → cek laporan skor; `scripts/screenshot.sh` render sprite (M11).

## Referensi
M09 (converter), `src/ui/sprite.rs` (SpriteSize sm/md/lg, ColorDepth tc/256/16).
