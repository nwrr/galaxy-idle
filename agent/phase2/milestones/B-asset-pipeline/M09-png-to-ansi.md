# M09 — PNG → ANSI converter

- **id:** M09 · **kategori:** B-asset-pipeline · **status:** todo
- **depends_on:** —  (paralel dgn M05–M08; butuh PNG sumber utk uji nyata dari M08)
- **gerbang:** U, A
- **DoD:** [GOALS](../../GOALS.md) U-D

## Tujuan
Konversi gambar PNG arbitrer → ANSI half-block berwarna detail, di ukuran sel target, dengan depth
tc/256/16. Reuse encoder `genassets/ansi.py`.

## File disentuh
- `scripts/genassets/` (modul `imgconv.py` baru atau perluas `ansi.py`/`build.py`)
- `scripts/gen_assets.py` (subcmd `convert`)

## Checklist
- [ ] M09.1 `png_to_ansi(img, cols, rows, depth)` skeleton — (U)
- [ ] M09.2 Downscale area-average ke (cols × 2·rows) px (half-block 2 px vertikal/sel) — (U)
- [ ] M09.3 Aspect lock + padding agar muat box tanpa distorsi — (U)
- [ ] M09.4 Map RGB→tc (reuse `ansi.py._fg/_bg`) — (U)
- [ ] M09.5 Map RGB→256 (reuse `rgb_to_256`) — (U)
- [ ] M09.6 Map RGB→16 (reuse `rgb_to_16`) — (U)
- [ ] M09.7 Dithering Floyd–Steinberg opsi (256/16) — (U)
- [ ] M09.8 Mask alpha/near-black bg → sel kosong (transparansi) — (U)
- [ ] M09.9 `gen_assets.py convert <src.png> <base> [--sizes ...]` → tulis `.ans` — (A)
- [ ] M09.10 Unit test konverter: gambar uji kecil → ANSI dimensi/escape benar — (U)

## Selesai bila
`gen_assets.py convert <png> <base>` menulis `.ans` half-block di tiap depth, dimensi tepat box,
transparansi benar.

## Verifikasi
`python3 scripts/gen_assets.py convert <sample.png> testbase`; inspeksi `.ans` (M10 skor fidelitas).

## Referensi
`scripts/genassets/ansi.py` (encoder existing), `scripts/genassets/celestial.py` (pola render).
