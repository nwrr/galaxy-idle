# M08 — Generate + verify semua aset (≥90)

- **id:** M08 · **kategori:** B-asset-pipeline · **status:** todo
- **depends_on:** M06, M07
- **gerbang:** S, A
- **DoD:** [GOALS](../../GOALS.md) U-C

## Tujuan
Hasilkan PNG sumber untuk semua set, lewati gerbang ≥90 (regen seed++ bila kurang), lalu agent
konfirmasi mata (Read PNG vs ref). Butuh server ComfyUI; bila absen → catat blocker & lanjut item lain.

## File disentuh
- `assets/source/{celestial,ship,galaxy,banner,characters}/*.png`
- `comfyui/prompts/*.json` (catat seed final)

## Checklist
- [ ] M08.1 Generate set celestial → PNG; `verify.py --set celestial --regen` ≥90 — (A)
- [ ] M08.2 Agent **Read** tiap PNG planet → biome benar & jelas (konfirmasi vision) — (S)
- [ ] M08.3 Generate+verify `star_sun` ≥90 — (S,A)
- [ ] M08.4 Generate+verify `ship` ≥90 — (S,A)
- [ ] M08.5 Generate+verify `galaxy` vs `galaxy_2.png` ≥90 — (S,A)
- [ ] M08.6 Generate+verify `banner` ≥90 — (S,A)
- [ ] M08.7 (Re)generate+verify 21 portrait karakter ≥90 (regen yang kurang) — (S,A)
- [ ] M08.8 Konsistensi gaya antar-aset (palet/lighting) dicek mata — (S)
- [ ] M08.9 Catat seed final tiap aset di set JSON (reproducible) — (A)

## Selesai bila
Semua set punya PNG sumber lulus ≥90 (atau blocker tercatat bila server tak ada); seed final
tersimpan; agent setuju kualitas.

## Verifikasi
`comfyui/verify.py --set <x>` semua PASS; Read sampel PNG tiap set.

## Referensi
M06 (set), M07 (gate). Server: `comfyui/README.md`. Blocker tanpa server → lihat LOOP Stop Conditions.
