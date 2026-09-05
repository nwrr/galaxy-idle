# comfyui/ — Auto-Generate Portrait Karakter

Generate 21 PNG sumber karakter (`assets/source/characters/{male,female,alien}/`) untuk pipeline
ASCII (`scripts/gen_assets.py portrait`). Dua jalur: **lokal (ComfyUI, gratis)** dan **cloud
berbayar (Flux)** di [`cloud/`](cloud/README.md).

## Analisis Hardware & Pilihan Model

| Komponen | Nilai |
|----------|-------|
| GPU | **NVIDIA RTX 3060 12GB** (driver 595) |
| CPU / RAM | Ryzen 5 2600 / 27GB |
| OS / ComfyUI | Arch Linux / ComfyUI v0.20.1 di `~/ComfyUI` |

**Model terbaik yang kamu miliki & cocok 12GB → Z-Image Turbo** (`z_image_turbo_bf16`). Distilled,
cepat (**4 langkah**, cfg 1), kualitas tinggi, dan kamu sudah punya semua komponennya. Graph
diambil persis dari blueprint `Text to Image (Z-Image-Turbo)` milikmu:

| Node | Nilai |
|------|-------|
| `UNETLoader` | `z_image_turbo_bf16.safetensors` |
| `ModelSamplingAuraFlow` | shift 3 |
| `CLIPLoader` | `qwen_3_4b.safetensors`, type **`lumina2`** |
| negatif | `ConditioningZeroOut` (cfg 1 → negatif tak berpengaruh) |
| `VAELoader` | `ae.safetensors` |
| `KSampler` | steps 4, cfg 1, `res_multistep` / `simple`, denoise 1 |

> Kenapa bukan Flux/SDXL? Z-Image Turbo lebih cepat di 3060, sudah terpasang, dan kualitasnya cukup.
> Catatan: portrait kini dirender sebagai **gambar** (ratatui-image), jadi resolusi & detail wajah
> berpengaruh — output 1024² Z-Image sudah bagus; latar hitam + rim light tetap dijaga di prompt.

## File

| File | Fungsi |
|------|--------|
| `workflows/workflow_zimage_turbo.json` | Workflow UI-format — drag ke ComfyUI untuk pakai/tweak manual |
| `workflows/workflow_zimage_turbo_api.json` | API-format — diposting oleh `generate.py` (atau "Load API format") |
| `prompts/characters.json` | 21 karakter: id, group, filename, subject, seed + `style`/`negative` global |
| `prompts/style.md` | Aturan gaya & framing untuk hasil ASCII yang bagus |
| `generate.py` | Driver API: loop 21 prompt → simpan PNG ke `assets/source/characters/..` |
| `cloud/` | Opsi kedua: Flux via fal.ai / Replicate (berbayar) |

## Skema prompt-set (Phase 2, M05)

Tiap file di `prompts/*.json` = satu **set** (grup asset yang di-generate bersama, mis. karakter,
planet per biome, kapal, galaxy, banner). Skema seragam, dipisah dari workflow (graph ComfyUI):

```jsonc
{
  "set": "characters",              // nama set — dipakai `generate.py --set <name>` & folder output
  "workflow": "workflow_zimage_turbo_api", // basename di workflows/ (tanpa .json), graph dipakai
  "size": "1024x1024",              // WxH default (override via --size)
  "style": "...",                   // deskripsi gaya global, digabung ke tiap subject
  "negative": "...",                // negative prompt global
  "items": [
    {
      "id": "char_male_01",         // unik dlm set; dipakai nama file & --only
      "seed": 1010001,              // deterministik; verify.py regen naikkan (seed+1, +2, ...)
      "subject": "...",             // deskripsi spesifik item, digabung dgn style
      "ref": null,                  // path referensi (relatif assets/source/references/) utk
                                     // gerbang verify.py M07 (SSIM+phash ≥90%); null = tak ada
                                     // target kemiripan gambar spesifik (mis. karakter rekaan)
      "group": "male",              // opsional: sub-folder output (assets/source/<set>/<group>/)
      "filename": "char_male_01.png" // opsional: nama file eksplisit (default `<id>.png`)
    }
  ]
}
```

**`items`** menggantikan nama field lama `characters` (spesifik-karakter) — netral lintas jenis
set. **`ref`** ditambah utk M07 (verify.py similarity gate), 3 bentuk valid:
- `null` — tak ada target kemiripan (mis. desain alien rekaan tanpa foto acuan).
- path string — relatif `assets/source/references/...`, dibanding via SSIM+phash (mis. galaxy →
  `galaxy_2.png`).
- `"colors:#hex1,#hex2,..."` — dipakai `celestial.json` (M06.3): tak ada foto biome planet nyata,
  jadi target berupa **2–3 warna dominan** (bukan gambar). Gerbang M07 utk bentuk ini = cek warna
  dominan hasil generate cukup dekat (jarak warna, bukan SSIM piksel-demi-piksel).

`group`/`filename` tetap ada khusus utk kompatibilitas struktur folder `characters/<group>/` yang
sudah ada; set baru bebas tak memakainya.

> `prompts/characters.json` sudah dimigrasi ke skema ini (M05.3); `generate.py --set <name>`
> membaca set apapun di `prompts/<name>.json` (M05.4). Set lain (planet/star/ship/galaxy/banner)
> menyusul M06.

## Cara Pakai (lokal)

1. **Jalankan ComfyUI** (API di `:8188`):
   ```
   cd ~/ComfyUI && python main.py
   # jika OOM 12GB:  python main.py --lowvram
   ```
2. **Generate** (dari root repo ini):
   ```
   python3 comfyui/generate.py --dry-run         # cek rencana dulu (default --set characters)
   python3 comfyui/generate.py                    # generate 21 → assets/source/characters/
   # subset:
   python3 comfyui/generate.py --only char_alien_03
   python3 comfyui/generate.py --group female --size 832x1216
   # set lain (M06+, mis. planet per biome) — sama pola begitu prompts/<set>.json ada:
   python3 comfyui/generate.py --set <nama-set> --dry-run
   ```
3. **Publish ke runtime** (karakter = **gambar**, dirender via `ratatui-image`, BUKAN ASCII):
   ```
   python3 scripts/gen_assets.py all              # salin+downscale PNG → assets/characters/<group>/*.png
   python3 scripts/check_assets.py                # 21 portrait PASS sbg image
   ```
   Portrait **tidak dikonversi ke ASCII** (wajah butuh resolusi). `gen_assets.py` hanya menyalin +
   downscale PNG sumber jadi runtime PNG 512×512. Sprite/banner tetap ASCII.

`generate.py` mengunduh hasil via API (`/view`) langsung ke path repo — tidak bergantung folder
`ComfyUI/output`. Hanya butuh Python stdlib (`urllib`).

## Tips Kualitas (untuk ASCII)

- **Latar hitam + rim light** sudah ada di style prompt → background jadi spasi saat di-ASCII.
- Ubah `subject`/`seed` di `characters.json` bila ingin variasi; jangan ubah `style` agar 21
  karakter konsisten.
- Bust 1:1 default 1024×1024; untuk framing lebih tinggi pakai `--size 832x1216`.

## Troubleshooting

| Gejala | Solusi |
|--------|--------|
| `Connection refused` | ComfyUI belum jalan / port beda → `--server http://127.0.0.1:PORT` |
| OOM / CUDA out of memory | start ComfyUI `--lowvram` (atau `--novram`), atau `--size 768x960` |
| Node/model "not found" | nama model di workflow harus sama persis dgn yang ada di `~/ComfyUI/models` |
| Hasil terlalu "ramai" di ASCII | naikkan kontras prompt, pastikan `pure black background` tetap ada |
