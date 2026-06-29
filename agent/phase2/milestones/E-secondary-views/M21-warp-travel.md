# M21 — Warp / Travel view

- **id:** M21 · **kategori:** E-secondary-views · **status:** todo
- **depends_on:** M13
- **gerbang:** V, S, U
- **DoD:** [GOALS](../../GOALS.md) U-F

## Tujuan
View perjalanan & warp: peta rute, progress travel, status kapal + sprite, particle exhaust saat
Traveling, aksi pilih target & mulai travel.

## File disentuh
- `src/ui/panels/warp.rs`, `src/app.rs`, `tests/snapshots.rs`, golden

## Checklist
- [ ] M21.1 Peta rute origin→target (garis/arc) — (S)
- [ ] M21.2 Progress travel (elapsed/total + bar) — (S)
- [ ] M21.3 Ship status panel (tier/part/cargo) — (S)
- [ ] M21.4 Sprite ship di view — (S)
- [ ] M21.5 Particle exhaust saat Traveling di rute — (S)
- [ ] M21.6 ETA + jarak (AU) — (S)
- [ ] M21.7 Daftar target tersedia (unlock by tier) — (S)
- [ ] M21.8 Warp Tier gate indicator (locked target) — (S)
- [ ] M21.9 Konfirmasi warp jump (prestige) bila relevan — (S)
- [ ] M21.10 Navigasi pilih target — (U)
- [ ] M21.11 Aksi mulai travel + feedback — (U)
- [ ] M21.12 Compact layout — (S)
- [ ] M21.13 Minimal layout — (S)
- [ ] M21.14 Golden snapshot warp — (V)
- [ ] M21.15 Screenshot warp vs target → layak — (S)
- [ ] M21.16 `visual_checks.md` warp diperbarui — (V,S)

## Selesai bila
warp view menampilkan rute+progress+ship+exhaust, target navigable & travel berfungsi, 3 breakpoint,
screenshot disetujui.

## Verifikasi
`scripts/screenshot.sh warp 120 40`/`80 30`/`60 24`; `cargo test`; `verify.sh` hijau.

## Referensi
`04-prestige.md`, `03-progression.md` §4, `06-ui.md`, `15-particle-effects.md`.
`src/ui/panels/warp.rs`, `src/ui/particles.rs`.
