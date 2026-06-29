# M15 — Research view (tree + progress)

- **id:** M15 · **kategori:** C-shell-views · **status:** todo
- **depends_on:** M13
- **gerbang:** V, S
- **DoD:** [GOALS](../../GOALS.md) U-F

## Tujuan
Terminal riset: pohon tech tervisual dengan status node, progress bar riset aktif, detail tech, dan
garis dependency.

## File disentuh
- `src/ui/panels/research.rs`, `tests/snapshots.rs`, golden

## Checklist
- [ ] M15.1 Tree tech per cabang/branch tervisual — (S)
- [ ] M15.2 Node status locked/available/active/done (warna/ikon beda) — (S)
- [ ] M15.3 Progress bar riset aktif (Data invested + waktu) — (S)
- [ ] M15.4 Detail tech terpilih (efek, biaya, depends_on) — (S)
- [ ] M15.5 Data/s total tampil — (S)
- [ ] M15.6 Garis dependency antar node (ASCII connector) — (S)
- [ ] M15.7 Warp Tier saat ini + next unlock — (S)
- [ ] M15.8 Navigasi antar node (j/k atau panah) — (U)
- [ ] M15.9 Aksi mulai riset terpilih + feedback — (S)
- [ ] M15.10 Highlight node terpilih — (S)
- [ ] M15.11 Scroll bila tree > area — (S)
- [ ] M15.12 Compact: tree ringkas — (S)
- [ ] M15.13 Minimal: list tech (no grafis tree) — (S)
- [ ] M15.14 Golden snapshot research — (V)
- [ ] M15.15 Screenshot research vs target → layak — (S)
- [ ] M15.16 `visual_checks.md` research diperbarui — (V,S)

## Selesai bila
research view menampilkan tree berstatus + progress + detail, navigable, terbaca 3 breakpoint,
screenshot disetujui.

## Verifikasi
`scripts/screenshot.sh research 120 40`/`80 30`/`60 24`; `verify.sh` hijau.

## Referensi
`06-ui.md`, `03-progression.md`, `data/tech_tree.ron`. `src/ui/panels/research.rs`.
