# M12 — Shell skeleton 3-kolom (Riftborne)

- **id:** M12 · **kategori:** C-shell-views · **status:** todo
- **depends_on:** M04
- **gerbang:** V, S
- **DoD:** [GOALS](../../GOALS.md) U-E

## Tujuan
Ganti shell lama dengan layout **Riftborne**: top status bar + 3 kolom persisten (kiri RESOURCES/
status · tengah MAIN · kanan OPTIONS) + footer MARKET. Kerangka dulu (slot kosong), isi panel di M13.

## File disentuh
- `src/ui/mod.rs` (shell_full/compact/minimal baru)

## Checklist
- [ ] M12.1 Top status bar: judul/lokasi/waktu/status build-train (baris atas) — (S)
- [ ] M12.2 Kolom kiri (lebar tetap ~22): slot RESOURCES + Building/Training + Assets — (S)
- [ ] M12.3 Kolom tengah MAIN: slot konten per view (Min(0)) — (S)
- [ ] M12.4 Kolom kanan (lebar ~18): slot OPTIONS menu — (S)
- [ ] M12.5 Footer MARKET strip (baris bawah) — (S)
- [ ] M12.6 Border/sekat warna konsisten (theme.header/dim) — (S)
- [ ] M12.7 `outer_block` "STELLAR IDLE" tetap, isi diisi 3 kolom — (S)
- [ ] M12.8 Proporsi enak di 120×40 (tak sesak) — (S)
- [ ] M12.9 Tak panic di ukuran kecil (guard lebar minimum) — (V)
- [ ] M12.10 Screenshot shell kosong vs `riftborne/42UN3F.png` → struktur setara — (S)
- [ ] M12.11 `verify.sh` hijau (snapshot lama di-regen sadar di M13) — (V)

## Selesai bila
Shell 3-kolom + status bar + market footer tampil di semua view (slot bisa kosong); struktur menyerupai
Riftborne; tak panic.

## Verifikasi
`scripts/screenshot.sh main_menu 120 40` → Read PNG; bandingkan `VISUAL_TARGETS.md` §Shell.

## Referensi
`assets/source/references/riftborne/42UN3F.png`, `06-ui.md`. Shell lama: `src/ui/mod.rs` `shell_full`.
