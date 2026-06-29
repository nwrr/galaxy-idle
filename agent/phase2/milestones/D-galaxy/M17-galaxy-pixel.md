# M17 — Galaxy pixel (sixel/kitty)

- **id:** M17 · **kategori:** D-galaxy · **status:** todo
- **depends_on:** M03, M08, M16
- **gerbang:** T, U
- **DoD:** [GOALS](../../GOALS.md) U-G

## Tujuan
Saat terminal mendukung grafis (sixel/kitty), tampilkan **galaxy PNG asli** (ComfyUI) via
`ratatui-image`; fallback ke backdrop procedural (M16) bila tak didukung/headless.

## File disentuh
- `src/ui/galaxy_pixel.rs` (baru), `src/ui/mod.rs`/panel galaxy
- `assets/source/galaxy/*.png` (dari M08)

## Checklist
- [ ] M17.1 Deteksi dukungan sixel/kitty via `ratatui-image` Picker — (U)
- [ ] M17.2 Load galaxy PNG → StatefulImage, render di area — (T)
- [ ] M17.3 Fallback ke backdrop procedural bila tak didukung/headless — (U)
- [ ] M17.4 Cache image protocol (no reload tiap frame) — (U)
- [ ] M17.5 Toggle pixel/procedural (key atau auto) — (T)
- [ ] M17.6 `capture_term galaxy` → pixel tampil benar — (T)

## Selesai bila
Galaxy view menampilkan gambar pixel di terminal grafis (terbukti via `capture_term.sh`), dan
fallback procedural mulus saat headless/tak didukung (snapshot tetap jalan).

## Verifikasi
`scripts/capture_term.sh galaxy_map 120 40` (terminal grafis) → Read PNG; headless → fallback ok.

## Referensi
`ratatui-image` (dep ada), `src/ui/portrait.rs` (pola Picker/StatefulImage). `07-architecture.md`.
