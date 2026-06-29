# M20 — Galaxy integrasi + responsif + sign-off

- **id:** M20 · **kategori:** D-galaxy · **status:** todo
- **depends_on:** M16, M17, M19
- **gerbang:** V, S, T
- **DoD:** [GOALS](../../GOALS.md) U-G

## Tujuan
Satukan backdrop (M16) + pixel (M17) + starmap (M18/19) dalam galaxy view yang konsisten dengan shell,
responsif, dan **disetujui** (galaxy = bagian paling krusial).

## File disentuh
- `src/ui/mod.rs` / panel galaxy, `tests/snapshots.rs`, golden

## Checklist
- [ ] M20.1 Mode backdrop vs starmap (tab/key) dalam galaxy view — (S)
- [ ] M20.2 Shell 3-kolom tetap di galaxy view — (S)
- [ ] M20.3 Compact: viewport mengecil, panel ringkas — (S)
- [ ] M20.4 Minimal: starmap fullscreen + footer — (S)
- [ ] M20.5 Navigasi galaxy bebas-panic (unit + smoke) — (V)
- [ ] M20.6 Golden snapshot galaxy_map 3 ukuran — (V)
- [ ] M20.7 capture_term galaxy pixel audit final — (T)
- [ ] M20.8 Review akhir: agent + user setuju galaxy "krusial" tercapai (backdrop≈galaxy_2, starmap≈Riftborne) — (S)

## Selesai bila
Galaxy view menyatukan backdrop+pixel+starmap, responsif 3 breakpoint, golden hijau, dan
screenshot/sign-off menyatakan kualitas tercapai.

## Verifikasi
`scripts/screenshot.sh galaxy_map 120 40`/`80 30`/`60 24`; `scripts/capture_term.sh galaxy_map 120 40`;
`verify.sh` hijau.

## Referensi
`galaxy_2.png`, `riftborne/owOev1.png`, `VISUAL_TARGETS.md` §Galaxy.
