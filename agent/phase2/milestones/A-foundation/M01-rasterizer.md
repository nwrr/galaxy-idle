# M01 — Rasterizer Buffer→PNG

- **id:** M01 · **kategori:** A-foundation · **status:** todo
- **depends_on:** — (paling awal Phase 2)
- **gerbang:** V (verify.sh), U (unit test)
- **DoD:** [GOALS](../../GOALS.md) U-A

## Tujuan
Bangun inti rasterizer: ubah `ratatui::buffer::Buffer` → gambar PNG berwarna. Tiap sel digambar
sebagai glyph (font monospace embedded) dengan warna fg/bg sebenarnya, termasuk box-drawing &
half-block `▀▄█`. Ini fondasi "screenshot langsung" (ganti `.txt` yang buang warna).

## File disentuh
- `Cargo.toml` (dep `ab_glyph`; `image` sudah ada)
- `assets/fonts/<mono>.ttf` (+ catatan lisensi)
- `src/ui/raster.rs` (baru), daftar di `src/ui/mod.rs`

## Checklist
- [ ] M01.1 Tambah dep `ab_glyph` di `Cargo.toml`; `cargo build` ok — (V)
- [ ] M01.2 Taruh font monospace TTF (coverage box/block, mis. JetBrains/Cascadia/Unifont) di
      `assets/fonts/` + lisensi — (V)
- [ ] M01.3 `raster.rs` skeleton + konstanta `CELL_W/CELL_H` (mis. 10×20 px) + modul terdaftar — (V,U)
- [ ] M01.4 `color_to_rgb(ratatui::Color, &Theme) -> [u8;3]`: tangani Named16/Indexed256/Rgb/Reset
      (Reset fg→palet teks, Reset bg→palet bg) — (U)
- [ ] M01.5 Load font via `ab_glyph::FontRef` (embed `include_bytes!`) — (U)
- [ ] M01.6 `draw_glyph(img, ch, col, row, fg, bg)`: isi bg sel lalu raster glyph fg (anti-alias) — (U)
- [ ] M01.7 Half-block `▀▄█` digambar programatik (isi setengah/penuh sel, pixel-perfect) — (U)
- [ ] M01.8 Box-drawing `│─┌┐└┘├┤┬┴┼╴╶` programatik bila font tak punya glyph — (U)
- [ ] M01.9 `buffer_to_rgba(&Buffer,&Theme)->image::RgbaImage`: loop sel → draw_glyph — (V,U)
- [ ] M01.10 Karakter lebar-ganda/unknown aman (fallback 1 sel, tak overflow tetangga) — (U)
- [ ] M01.11 Test: render Buffer kecil berisi teks+box+half-block → RgbaImage dimensi & warna benar — (U)

## Selesai bila
`buffer_to_rgba` menghasilkan `RgbaImage` berukuran `w*CELL_W × h*CELL_H` dengan warna fg/bg sesuai
sel; box & half-block tampil tegas; `verify.sh` hijau; ≥3 unit test rasterizer lulus.

## Verifikasi
`cargo test raster`; inspeksi manual via M02 (belum ada output PNG di M01).

## Referensi
`abstraction/design/06-ui.md` (warna/elemen), `07-architecture.md`. Half-block sama spt
`scripts/genassets/ansi.py`.
