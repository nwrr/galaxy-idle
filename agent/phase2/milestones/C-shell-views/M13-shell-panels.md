# M13 — Shell panels padat + responsif + golden

- **id:** M13 · **kategori:** C-shell-views · **status:** todo
- **depends_on:** M12
- **gerbang:** V, S
- **DoD:** [GOALS](../../GOALS.md) U-E

## Tujuan
Isi panel persisten shell dengan data padat ala Riftborne, terapkan responsif 3 breakpoint, dan
perbarui golden snapshot.

## File disentuh
- `src/ui/panels/{resources,menu,shortcuts}.rs` (+ panel baru building/assets)
- `src/ui/mod.rs` (compact/minimal), `tests/snapshots.rs`, golden

## Checklist
- [ ] M13.1 RESOURCES tabel (Now/Cap/▲per-h/ETA) rata-kanan + warna tier — (S)
- [ ] M13.2 Panel Building/Training (antrian + timer) — (S)
- [ ] M13.3 Panel Assets (Buildings/Ships ringkas berlevel) — (S)
- [ ] M13.4 OPTIONS menu berkelompok (COLONY/EMPIRE/MILITARY/INTEL/SYSTEM) — (S)
- [ ] M13.5 Footer MARKET: komoditas + harga ringkas — (S)
- [ ] M13.6 Highlight item menu/view aktif (reverse/bg) — (S)
- [ ] M13.7 Format angka konsisten (k/M, ribuan) + alignment — (V,S)
- [ ] M13.8 Compact (≥70w): kolom kanan ringkas/sembunyi — (S)
- [ ] M13.9 Minimal (<70w): tab bar + main + footer — (S)
- [ ] M13.10 Boundary 70/100w tak glitch — (S)
- [ ] M13.11 Golden snapshot shell 3 ukuran di-regen sadar; `verify.sh` hijau — (V)

## Selesai bila
Panel persisten padat-informasi & terbaca di 3 breakpoint; kepadatan menyerupai Riftborne; golden
hijau.

## Verifikasi
`scripts/screenshot.sh main_menu 120 40` / `80 30` / `60 24` → Read; `verify.sh` hijau.

## Referensi
`riftborne/42UN3F.png`, `06-ui.md`. Panel existing: `src/ui/panels/`.
