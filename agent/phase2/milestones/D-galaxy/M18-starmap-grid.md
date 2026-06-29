# M18 — Starmap grid + navigasi

- **id:** M18 · **kategori:** D-galaxy · **status:** todo
- **depends_on:** M13
- **gerbang:** S, U
- **DoD:** [GOALS](../../GOALS.md) U-G

## Tujuan
Star map **playable** ala Riftborne (`owOev1.png`): grid tile sistem berwarna, kursor yang bisa
digerakkan, scroll viewport. Pondasi "se-playable mungkin".

## File disentuh
- `src/ui/panels/galaxy_map.rs` (upgrade), `src/app.rs` (galaxy_map_keys, sudah ada)

## Checklist
- [ ] M18.1 Grid tile sistem (huruf/simbol) di viewport — (S)
- [ ] M18.2 Warna tile per faksi/biome/status — (S)
- [ ] M18.3 Kursor sel (highlight) + tampil koordinat — (S)
- [ ] M18.4 Navigasi kursor h/j/k/l + panah — (U)
- [ ] M18.5 Scroll viewport saat kursor di tepi — (U)
- [ ] M18.6 Grid deterministik dari seed procgen (sama tiap render) — (U)
- [ ] M18.7 Density grid mirip Riftborne (padat tapi terbaca) — (S)
- [ ] M18.8 Marker home/ship/target — (S)
- [ ] M18.9 Clamp kursor & viewport (no out-of-bounds/panic) — (U)
- [ ] M18.10 Screenshot starmap vs `owOev1.png` → struktur grid setara — (S)

## Selesai bila
Grid starmap berwarna tampil; kursor bergerak h/j/k/l dgn scroll; deterministik dari seed; mirip
Riftborne; tak panic.

## Verifikasi
`scripts/screenshot.sh galaxy_map 120 40` → Read; `cargo test` navigasi.

## Referensi
`assets/source/references/riftborne/owOev1.png`, `16-procgen.md`, `06-ui.md`.
`src/ui/panels/galaxy_map.rs`, `src/game/world/procgen.rs`.
