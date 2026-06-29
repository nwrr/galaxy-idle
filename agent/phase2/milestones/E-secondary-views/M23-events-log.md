# M23 — Events / Log view

- **id:** M23 · **kategori:** E-secondary-views · **status:** todo
- **depends_on:** M13
- **gerbang:** V, S, U
- **DoD:** [GOALS](../../GOALS.md) U-F

## Tujuan
Panel/View LOG/EVENTS: baca `events` queue, tampilkan log berwarna per jenis, scroll, resolusi event,
badge unread di menu.

## File disentuh
- `src/ui/panels/events.rs` (baru), `src/ui/mod.rs`, `src/app.rs`
- `tests/snapshots.rs`, golden

## Checklist
- [ ] M23.1 Panel Events/Log baca `events` queue — (S)
- [ ] M23.2 Entri log terbaru (waktu/jenis/teks) — (S)
- [ ] M23.3 Warna per jenis event — (S)
- [ ] M23.4 Scroll log — (U)
- [ ] M23.5 Resolusi event terpilih (aksi) — (U)
- [ ] M23.6 Badge unread di menu OPTIONS — (S)
- [ ] M23.7 Log ringkas terintegrasi di shell (panel bawah) — (S)
- [ ] M23.8 Empty state (no event) jelas — (S)
- [ ] M23.9 Golden snapshot events — (V)
- [ ] M23.10 Screenshot events vs target → layak — (S)

## Selesai bila
events view menampilkan log berwarna + scroll + resolusi; badge unread tampil; screenshot disetujui.

## Verifikasi
`scripts/screenshot.sh events 120 40`; `cargo test`; `verify.sh` hijau.

## Referensi
`05-events-merchant.md`, `04-prestige.md`, `06-ui.md`. `src/game/events.rs`, `src/game/state.rs`.
