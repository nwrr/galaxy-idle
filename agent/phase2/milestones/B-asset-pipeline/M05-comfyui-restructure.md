# M05 — ComfyUI restructure (prompt ⟂ workflow, `--set`)

- **id:** M05 · **kategori:** B-asset-pipeline · **status:** todo
- **depends_on:** —
- **gerbang:** A (skrip jalan/dry-run)
- **DoD:** [GOALS](../../GOALS.md) U-C

## Tujuan
Pisahkan **prompt** (data, banyak) dari **workflow** (graph ComfyUI), agar mudah generate banyak set
asset. Generalisasi `generate.py` agar terima `--set <name>`.

## File disentuh
- `comfyui/workflows/` (baru; pindahan `workflow_zimage_turbo*.json`)
- `comfyui/prompts/` (multi-set), `comfyui/generate.py`, `comfyui/README.md`

## Checklist
- [ ] M05.1 Buat `comfyui/workflows/` + pindah `workflow_zimage_turbo*.json` ke sana — (A)
- [ ] M05.2 Skema prompt-set seragam `{set, workflow, size, style, negative, items:[{id,seed,subject,ref}]}` — (A)
- [ ] M05.3 `characters.json` dimigrasi ke skema baru (tetap 21 item) — (A)
- [ ] M05.4 `generate.py --set <name>`: baca set → resolusi workflow → post per item — (A)
- [ ] M05.5 Output ke `assets/source/<set>/...` (folder per set) — (A)
- [ ] M05.6 Flag lama tetap jalan lintas-set: `--only/--seed/--steps/--group/--dry-run` — (A)
- [ ] M05.7 `--dry-run` cetak rencana akurat (set, item, seed, path) — (A)
- [ ] M05.8 README `comfyui/` perbarui: pisah prompt/workflow + alur `--set` — (A)

## Selesai bila
`python3 comfyui/generate.py --set characters --dry-run` cetak 21 rencana; struktur `workflows/` ⟂
`prompts/` rapi; tanpa server tetap dry-run sukses.

## Verifikasi
`python3 comfyui/generate.py --set characters --dry-run`. (Generate nyata butuh server ComfyUI —
lihat M08.)

## Referensi
`comfyui/generate.py` (existing), `comfyui/README.md`.
