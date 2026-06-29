# M27 — Visual checks + golden final

- **id:** M27 · **kategori:** F-polish-verify · **status:** todo
- **depends_on:** M26
- **gerbang:** V, S
- **DoD:** [GOALS](../../GOALS.md) U-I

## Tujuan
Perluas `visual_checks.md` dengan assertion warna/asset/overflow yang terukur, dan pastikan semua
golden snapshot mutakhir & hijau.

## File disentuh
- `agent/test/visual_checks.md`, `agent/test/snapshots/*`, `tests/snapshots.rs`

## Checklist
- [ ] M27.1 Assertion sprite tampil (biome beda) per view relevan — (S)
- [ ] M27.2 Assertion portrait tampil/fallback (merchant/judul) — (S)
- [ ] M27.3 Assertion shell 3-kolom + status bar + market tiap view — (S)
- [ ] M27.4 Assertion no overflow/clip 3 breakpoint — (S)
- [ ] M27.5 Assertion galaxy backdrop+starmap (deskriptif) — (S)
- [ ] M27.6 Assertion theme Default/HC/Mono terbaca — (S)
- [ ] M27.7 Semua golden snapshot di-regen sadar & cocok — (V)
- [ ] M27.8 Tambah snapshot view baru (merchant/events) ke `tests/snapshots.rs` — (V)
- [ ] M27.9 `tests/snapshots.rs` semua hijau — (V)
- [ ] M27.10 Cross-check tiap assertion vs screenshot terbaru — (S)
- [ ] M27.11 Hapus assertion usang/duplikat — (V)
- [ ] M27.12 `verify.sh` hijau penuh — (V)

## Selesai bila
`visual_checks.md` lengkap & lulus; semua golden mutakhir; `verify.sh` hijau penuh.

## Verifikasi
`scripts/verify.sh`; cross-check `agent/test/visual_checks.md` vs screenshot.

## Referensi
`agent/test/visual_checks.md`, `agent/test/README.md`, `tests/snapshots.rs`.
