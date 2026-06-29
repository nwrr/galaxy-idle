# M25 — Particles & banner polish

- **id:** M25 · **kategori:** F-polish-verify · **status:** todo
- **depends_on:** M12, M16
- **gerbang:** S
- **DoD:** [GOALS](../../GOALS.md) U-H

## Tujuan
Polish efek: particle (engine exhaust, warp trail) halus & tak mengganggu keterbacaan; banner judul
tampil di main menu, responsif.

## File disentuh
- `src/ui/particles.rs`, `src/ui/panels/main_view.rs`, `assets/banner/*`

## Checklist
- [ ] M25.1 Engine exhaust halus (saat Traveling) — (S)
- [ ] M25.2 Warp trail saat warp jump — (S)
- [ ] M25.3 Particle tak menutupi teks penting (z-order/area) — (S)
- [ ] M25.4 Banner judul tampil di main menu — (S)
- [ ] M25.5 Banner responsif (small variant di Compact/Minimal) — (S)
- [ ] M25.6 Ikon/simbol set konsisten (★✦·▲) — (S)
- [ ] M25.7 Loading/empty states konsisten — (S)
- [ ] M25.8 Particle deterministik di snapshot (seed/anim_secs=0) — (S)
- [ ] M25.9 Screenshot main_menu (banner+anim+particle) → enak & terbaca — (S)

## Selesai bila
Particle & banner mempercantik tanpa mengurangi keterbacaan; banner responsif; screenshot disetujui.

## Verifikasi
`scripts/screenshot.sh main_menu 120 40`/`60 24`; `verify.sh` hijau.

## Referensi
`15-particle-effects.md`, `14-galaxy-animation.md`, `06-ui.md`. `assets/banner/`.
