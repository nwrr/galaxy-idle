# M22 — Merchant view

- **id:** M22 · **kategori:** E-secondary-views · **status:** todo
- **depends_on:** M11, M13
- **gerbang:** V, S, U
- **DoD:** [GOALS](../../GOALS.md) U-B, U-F

## Tujuan
Void Merchant view: portrait pedagang (ratatui-image/fallback), daftar offer + harga + afford, timer,
flavor text. Wire `Merchant` model yang sudah ada.

## File disentuh
- `src/ui/panels/merchant.rs` (baru), `src/ui/mod.rs` (view), `src/app.rs`
- `tests/snapshots.rs`, golden

## Checklist
- [ ] M22.1 Portrait pedagang (via `App.portraits`, fallback half-block) — (S)
- [ ] M22.2 Daftar offer (blueprint/buy/sell) + harga cores/credits — (S)
- [ ] M22.3 Highlight offer terpilih + afford indicator — (S)
- [ ] M22.4 Timer merchant (expires/restock) — (S)
- [ ] M22.5 Dialog/flavor text pedagang — (S)
- [ ] M22.6 Navigasi offer — (U)
- [ ] M22.7 Aksi beli/jual + konfirmasi + feedback — (U)
- [ ] M22.8 Empty state (no merchant aktif) jelas — (S)
- [ ] M22.9 Golden snapshot merchant — (V)
- [ ] M22.10 Screenshot merchant vs target → layak — (S)

## Selesai bila
merchant view menampilkan portrait + offer + timer; beli/jual berfungsi; empty state rapi; screenshot
disetujui.

## Verifikasi
`scripts/screenshot.sh merchant 120 40`; `cargo test`; `verify.sh` hijau.

## Referensi
`05-events-merchant.md`, `06-ui.md`. `src/game/state.rs` (`Merchant`/`MerchantOffer`), `src/ui/portrait.rs`.
