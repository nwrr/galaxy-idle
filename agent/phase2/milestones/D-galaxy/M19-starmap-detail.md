# M19 — Starmap detail + aksi

- **id:** M19 · **kategori:** D-galaxy · **status:** todo
- **depends_on:** M18
- **gerbang:** S, U
- **DoD:** [GOALS](../../GOALS.md) U-G

## Tujuan
Lengkapi starmap jadi benar-benar playable: panel detail tile, legend, minimap, dan aksi (scan,
masuk sistem, kirim ship). Sprite body ANSI sebagai preview tile.

## File disentuh
- `src/ui/panels/galaxy_map.rs`, `src/app.rs` (aksi)

## Checklist
- [ ] M19.1 Panel SELECTED TILE: koordinat/tipe/richness/owner — (S)
- [ ] M19.2 Legend simbol/warna — (S)
- [ ] M19.3 Minimap (peta kecil posisi viewport) — (S)
- [ ] M19.4 Body sprite ANSI preview tile terpilih — (S)
- [ ] M19.5 Aksi `s` scan/materialize tile (sudah ada → integrasi UI) — (U)
- [ ] M19.6 Aksi `Enter` masuk planet/sistem terpilih — (U)
- [ ] M19.7 Aksi kirim ship ke tile (set Traveling) — (U)
- [ ] M19.8 Highlight rute ship (kursor→target) — (S)
- [ ] M19.9 Filter (faksi/tile) toggle — (S)
- [ ] M19.10 Status incoming/outgoing bila relevan — (S)

## Selesai bila
Tile terpilih menampilkan detail + preview sprite; legend & minimap ada; scan/enter/kirim-ship
berfungsi & terlihat di UI.

## Verifikasi
`scripts/screenshot.sh galaxy_map 120 40` → Read; `cargo test` aksi; uji manual nav+aksi.

## Referensi
`riftborne/owOev1.png` (SELECTED TILE), `16-procgen.md`. `src/game/world/procgen.rs`, `src/ui/sprite.rs`.
