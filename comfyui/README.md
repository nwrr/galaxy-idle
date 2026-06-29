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
| `workflow_zimage_turbo.json` | Workflow UI-format — drag ke ComfyUI untuk pakai/tweak manual |
| `workflow_zimage_turbo_api.json` | API-format — diposting oleh `generate.py` (atau "Load API format") |
| `prompts/characters.json` | 21 karakter: id, group, filename, subject, seed + `style`/`negative` global |
| `prompts/style.md` | Aturan gaya & framing untuk hasil ASCII yang bagus |
| `generate.py` | Driver API: loop 21 prompt → simpan PNG ke `assets/source/characters/..` |
| `cloud/` | Opsi kedua: Flux via fal.ai / Replicate (berbayar) |

## Cara Pakai (lokal)

1. **Jalankan ComfyUI** (API di `:8188`):
   ```
   cd ~/ComfyUI && python main.py
   # jika OOM 12GB:  python main.py --lowvram
   ```
2. **Generate** (dari root repo ini):
   ```
   python3 comfyui/generate.py --dry-run         # cek rencana dulu
   python3 comfyui/generate.py                    # generate 21 → assets/source/characters/
   # subset:
   python3 comfyui/generate.py --only char_alien_03
   python3 comfyui/generate.py --group female --size 832x1216
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
