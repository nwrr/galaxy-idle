# M07 — verify.py similarity gate (≥90 + regen)

- **id:** M07 · **kategori:** B-asset-pipeline · **status:** todo
- **depends_on:** M05
- **gerbang:** A
- **DoD:** [GOALS](../../GOALS.md) U-C

## Tujuan
Gerbang otomatis kemiripan hasil ComfyUI ke referensi: skor 0–100; `<90` → regen seed++. Agent
mengonfirmasi final via vision (Read PNG). Memenuhi permintaan "agent cek hasil ≥90% else regenerate".

## File disentuh
- `comfyui/verify.py` (baru)
- output: `assets/source/<set>/_verify.json`

## Checklist
- [ ] M07.1 `verify.py --set <name>`: load tiap PNG hasil + `ref`-nya — (A)
- [ ] M07.2 Skor SSIM (grayscale + warna) via Pillow+numpy — (A)
- [ ] M07.3 Skor perceptual-hash (phash) Hamming → normalisasi — (A)
- [ ] M07.4 Skor gabungan 0–100 (bobot SSIM+phash); ambang `--gate` default 90 — (A)
- [ ] M07.5 CLIP cosine **opsional** (skip rapi bila lib absen) — (A)
- [ ] M07.6 Tulis `_verify.json` (per item: skor, komponen, PASS/REGEN) — (A)
- [ ] M07.7 `--regen`: panggil `generate.py --only <id> --seed <next>` loop ≤N, simpan skor terbaik — (A)
- [ ] M07.8 Ringkasan stdout (lulus/total, item perlu regen) — (A)

## Selesai bila
`verify.py --set <x>` menghasilkan `_verify.json` + ringkasan; `--regen` menaikkan seed sampai ≥90
atau menyerah dengan catatan (≤N). Tanpa `torch` tetap jalan (SSIM+phash).

## Verifikasi
`python3 comfyui/verify.py --set characters` (pakai PNG yang ada). Cek `_verify.json`.

## Referensi
Plan §B. Ambang ≥90 = permintaan user. Dipakai M08/M11.
