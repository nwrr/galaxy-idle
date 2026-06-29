# M28 — Final sign-off (DoD gate)

- **id:** M28 · **kategori:** F-polish-verify · **status:** todo
- **depends_on:** M27 (+ semua milestone)
- **gerbang:** S, T, A
- **DoD:** [GOALS](../../GOALS.md) U-A..U-J (semua)

## Tujuan
Gerbang akhir Phase 2: screenshot tiap view × 3 breakpoint **disetujui**, semua DoD terpenuhi,
ringkasan rilis ditulis.

## File disentuh
- `agent/test/screens/approved/*`, `agent/phase2/PROGRESS.md`, `state.json`

## Checklist
- [ ] M28.1 `cargo run` manual: TUI padat, asset tampil, quit `q` bersih — (T)
- [ ] M28.2 Screenshot tiap view Full disetujui → `agent/test/screens/approved/` — (S)
- [ ] M28.3 Screenshot tiap view Compact disetujui — (S)
- [ ] M28.4 Screenshot tiap view Minimal disetujui — (S)
- [ ] M28.5 ComfyUI `_verify.json` semua ≥90 + ANSI fidelitas ≥99 + `check_assets.py` PASS — (A)
- [ ] M28.6 Tak ada modul asset nganggur (sprite/portrait/galaxy terpakai) — (A)
- [ ] M28.7 DoD U-A..U-J semua ✅ di `GOALS.md` — (S)
- [ ] M28.8 `PROGRESS.md` ditutup ringkasan rilis Phase 2; sign-off user — (S)

## Selesai bila
Semua DoD U-A..U-J ✅; screenshot tiap view×3 disetujui & terarsip; verify hijau; PROGRESS ditutup;
user sign-off. **→ Phase 2 SELESAI.**

## Verifikasi
`scripts/verify.sh` + `check_assets.py`; review folder `agent/test/screens/approved/`.

## Referensi
`GOALS.md` (DoD), `VISUAL_TARGETS.md`. Stop Condition "Selesai" di `LOOP.md`.
