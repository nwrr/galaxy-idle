# M16 — Galaxy backdrop procedural (spiral)

- **id:** M16 · **kategori:** D-galaxy · **status:** todo
- **depends_on:** M13
- **gerbang:** S, U
- **DoD:** [GOALS](../../GOALS.md) U-G

## Tujuan
Upgrade `galaxy_anim` dari "kelap-kelip" jadi **spiral galaksi berwarna** menyerupai `galaxy_2.png`:
lengan spiral, core terang, gradien warna, twinkle, rotasi, resize penuh. (KRUSIAL.)

## File disentuh
- `src/ui/galaxy_anim.rs` (upgrade)

## Checklist
- [ ] M16.1 Model lengan spiral (log-spiral, N arm) — (U)
- [ ] M16.2 Distribusi partikel: padat core → jarang tepi — (U)
- [ ] M16.3 Gradien warna radial (putih-biru core → biru gelap tepi) — (S)
- [ ] M16.4 Half-block density utk kepadatan bintang (sub-sel) — (S)
- [ ] M16.5 Twinkle deterministik dari `anim_secs` — (S)
- [ ] M16.6 Rotasi lambat keseluruhan — (S)
- [ ] M16.7 Depth/parallax (layer brightness) — (S)
- [ ] M16.8 Core glow / bloom — (S)
- [ ] M16.9 Resize penuh: isi area apa pun tanpa distorsi — (S)
- [ ] M16.10 Performa render < budget frame (no lag) — (U)
- [ ] M16.11 Snapshot deterministik (anim_secs=0) golden — (V)
- [ ] M16.12 Screenshot backdrop vs `galaxy_2.png` → mirip (warna/bentuk) — (S)

## Selesai bila
Backdrop tampak seperti galaksi spiral berwarna (mirip `galaxy_2.png`), beranimasi halus, mengisi
area apa pun, deterministik saat anim_secs=0.

## Verifikasi
`scripts/screenshot.sh main_menu 120 40` (MAIN backdrop) → Read; bandingkan `galaxy_2.png`.

## Referensi
`assets/source/references/galaxy_2.png`, `6-galaksi-bima-sakti.jpg`, `14-galaxy-animation.md`.
`src/ui/galaxy_anim.rs`.
