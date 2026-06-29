# Opsi Kedua — Flux via Provider Berbayar (fal.ai / Replicate)

Alternatif bila tak ingin generate lokal: pakai **Flux** di cloud. Output disimpan ke path yang
sama (`assets/source/characters/<group>/<filename>.png`) lalu lanjut `scripts/gen_assets.py all`.

## Provider & Model

| Provider | Model id | Catatan | Estimasi biaya* |
|----------|----------|---------|-----------------|
| **fal.ai** | `fal-ai/flux/dev` | cepat, murah, kualitas tinggi | ~$0.025 / gambar |
| **fal.ai** | `fal-ai/flux-pro/v1.1` | kualitas terbaik | ~$0.04 / gambar |
| **Replicate** | `black-forest-labs/flux-dev` | mudah, pay-per-second | ~$0.03 / gambar |
| **Replicate** | `black-forest-labs/flux-1.1-pro` | terbaik | ~$0.04 / gambar |

\* Perkiraan; cek harga terbaru di situs provider. 21 gambar ≈ $0.5–$0.9 total.

## Setup

```bash
# fal.ai
pip install fal-client
export FAL_KEY="xxxxxxxx"

# atau Replicate
pip install replicate
export REPLICATE_API_TOKEN="r8_xxxx"
```

API key: fal.ai → dashboard "Keys"; Replicate → Account → API tokens.

## Pakai

**Otomatis (script):**
```bash
python3 comfyui/cloud/generate_cloud.py --provider fal --model fal-ai/flux/dev
python3 comfyui/cloud/generate_cloud.py --provider replicate --model black-forest-labs/flux-dev
python3 comfyui/cloud/generate_cloud.py --provider fal --only char_alien_03   # subset
```
Script membaca `comfyui/prompts/characters.json` (subject + style yang sama dgn lokal) dan menyimpan
PNG ke `assets/source/characters/..`.

**Manual:** salin prompt dari [`prompts_flux.md`](prompts_flux.md) ke playground provider. Setelan
disarankan: rasio **1:1** (atau 4:5 untuk bust lebih tinggi), `num_inference_steps` 28–32 (dev),
`guidance` 3–3.5, output PNG. Simpan dengan nama `char_<group>_NN.png` di folder yang sesuai.

## Konsistensi dgn pipeline ASCII

Prompt sudah menekankan **pure black background + dramatic rim lighting + bust terpusat** agar
hasil sama baik saat dikonversi ke ASCII (latar gelap → spasi, subjek terang → padat). Setelah
gambar ada:
```
python3 scripts/gen_assets.py all && python3 scripts/check_assets.py
```
