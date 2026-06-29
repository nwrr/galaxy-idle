# M14 — Planet / Colony view (sprite body wired)

- **id:** M14 · **kategori:** C-shell-views · **status:** todo
- **depends_on:** M11, M13
- **gerbang:** V, S, U
- **DoD:** [GOALS](../../GOALS.md) U-B, U-F

## Tujuan
View planet utama: sprite biome tampil di MAIN + daftar node/factory padat + aksi build/upgrade
dengan indikator afford. Inilah "bisa melakukan sesuatu" yang user rasa hilang.

## File disentuh
- `src/ui/panels/planet.rs`, `src/app.rs` (planet_keys, sudah ada)
- `tests/snapshots.rs`, golden

## Checklist
- [ ] M14.1 Sprite biome planet aktif di MAIN (size per area) via `App.sprites` — (S)
- [ ] M14.2 Header planet: nama/sektor/tier/biome/distance — (S)
- [ ] M14.3 Panel RESOURCE NODES: richness/level + selektor — (S)
- [ ] M14.4 Panel FACTORIES: kind/level/enabled + selektor — (S)
- [ ] M14.5 Panel SHIP ringkas (tier/engine/cargo/status) — (S)
- [ ] M14.6 Panel RESEARCH ringkas (Data/s, aktif) — (S)
- [ ] M14.7 Status bar aksi kontekstual `[1]Node [2]Upgrade [3]Build [4]Lab` — (S)
- [ ] M14.8 Highlight slot/node terpilih jelas — (S)
- [ ] M14.9 Biaya upgrade + afford indicator (hijau/merah) — (S)
- [ ] M14.10 Net income/s per resource — (S)
- [ ] M14.11 Bar kapasitas stockpile (fill) — (S)
- [ ] M14.12 Empty slot tampil `[+ Build]` — (S)
- [ ] M14.13 Sprite + list tak tabrakan (split rapi) — (S)
- [ ] M14.14 Navigasi j/k antar node & slot konsisten — (U)
- [ ] M14.15 Aksi upgrade feedback (level naik terlihat) — (S)
- [ ] M14.16 Detail item terpilih di panel bawah — (S)
- [ ] M14.17 Warna tier resource konsisten dgn RESOURCES — (S)
- [ ] M14.18 Compact: sprite mengecil (md/sm), list tetap — (S)
- [ ] M14.19 Minimal: sprite sm/sembunyi, list prioritas — (S)
- [ ] M14.20 Golden snapshot planet_view 3 ukuran — (V)
- [ ] M14.21 Screenshot planet_view vs target → layak — (S)
- [ ] M14.22 `visual_checks.md` planet_view diperbarui — (V,S)

## Selesai bila
planet_view menampilkan sprite biome + info padat + aksi berfungsi (build/upgrade), terbaca di 3
breakpoint, screenshot disetujui.

## Verifikasi
`scripts/screenshot.sh planet_view 120 40`/`80 30`/`60 24`; `cargo test`; `verify.sh` hijau.

## Referensi
`riftborne/42UN3F.png` (COLONY), `06-ui.md`. `src/ui/panels/planet.rs`, `src/game/actions.rs`.
