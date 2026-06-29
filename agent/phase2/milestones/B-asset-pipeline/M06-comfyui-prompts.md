# M06 — ComfyUI prompt sets (celestial/star/ship/galaxy/banner)

- **id:** M06 · **kategori:** B-asset-pipeline · **status:** todo
- **depends_on:** M05
- **gerbang:** A
- **DoD:** [GOALS](../../GOALS.md) U-C

## Tujuan
Tulis prompt-set non-karakter untuk semua asset visual yang dipakai game, lengkap dengan peta
`ref` (referensi pembanding kemiripan) ke `assets/source/references/`.

## File disentuh
- `comfyui/prompts/{celestial,ship,galaxy,banner}.json`
- `comfyui/workflows/texture.json` (bila beda dari portrait)

## Checklist
- [ ] M06.1 `celestial.json`: 9 biome planet (terran/ocean/gasgiant/deadworld/crystalworld/iceworld/
      lavaworld/ironworld/asteroidbelt) — prompt khas + warna biome jelas — (A)
- [ ] M06.2 `celestial.json`: `star_sun` — (A)
- [ ] M06.3 Tiap planet `ref` ke gambar acuan biome (atau deskripsi target warna) — (A)
- [ ] M06.4 `ship.json`: hull kapal sci-fi (top/side view utk sprite) — (A)
- [ ] M06.5 `galaxy.json`: galaxy top-down spiral, `ref: galaxy_2.png` — (A)
- [ ] M06.6 `banner.json`: logo/title "STELLAR IDLE" art — (A)
- [ ] M06.7 `texture.json` workflow (square, planet/galaxy) bila param beda dari portrait — (A)
- [ ] M06.8 Semua set lulus `--dry-run` (item/seed/path benar) — (A)

## Selesai bila
Set celestial(10)/ship/galaxy/banner ada, ber-`ref`, dan dry-run benar lintas-set.

## Verifikasi
`python3 comfyui/generate.py --set celestial --dry-run` (dst tiap set).

## Referensi
`assets/source/references/` (galaxy_2.png, dll). Biome di `src/game/state.rs` enum `Biome`.
