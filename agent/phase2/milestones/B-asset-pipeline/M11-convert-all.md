# M11 — Konversi semua aset + check_assets + manifest

- **id:** M11 · **kategori:** B-asset-pipeline · **status:** todo
- **depends_on:** M08, M10
- **gerbang:** A, S
- **DoD:** [GOALS](../../GOALS.md) U-D

## Tujuan
Konversi semua PNG sumber (M08) → ANSI 9 varian (M10), validasi lengkap, daftarkan di manifest, dan
konfirmasi sprite tampil tajam via screenshot.

## File disentuh
- `assets/sprites/<base>/*.ans` (9 biome, star, ship, galaxy-fallback)
- `assets/banner/*`, `assets/manifest.ron`
- `scripts/check_assets.py`, `assets/README.md`, `assets/FORMAT.md`

## Checklist
- [ ] M11.1 Konversi 9 planet biome → ANSI 9 varian masing-masing — (A)
- [ ] M11.2 Konversi `star_sun` + `ship` — (A)
- [ ] M11.3 Konversi `galaxy` (utk fallback non-sixel) — (A)
- [ ] M11.4 Banner → ANSI/teks — (A)
- [ ] M11.5 `check_assets.py`: ANSI-aware (strip escape) + tiap base 9 varian lengkap & dalam box — (A)
- [ ] M11.6 `check_assets.py`: laporan skor fidelitas per aset — (A)
- [ ] M11.7 `manifest.ron`: entri canonical lg/tc tiap base + komentar varian — (A)
- [ ] M11.8 Render tiap base via `ui::sprite::Sprites` → screenshot → biome jelas beda, tajam — (S)

## Selesai bila
Semua base punya 9 varian lulus `check_assets.py`; manifest lengkap; screenshot sprite menunjukkan
biome berbeda & tajam.

## Verifikasi
`python3 scripts/check_assets.py` PASS; `scripts/screenshot.sh planet_view 120 40` → sprite tampil.

## Referensi
M09/M10. `assets/manifest.ron`, `scripts/check_assets.py`, `src/ui/sprite.rs`.
