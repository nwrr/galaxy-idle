# Gaya & Aturan Prompt Portrait

Hasil PNG dirender sebagai **gambar** di terminal via `ratatui-image` (bukan ASCII). Karena tampil
sebagai gambar penuh dalam panel, jaga agar terbaca jelas dalam area kecil:

- **Latar belakang hitam pekat / gelap solid** → menyatu rapi dengan panel TUI gelap, fokus ke wajah.
- **Pencahayaan rim/key dramatis, kontras tinggi** → wajah jelas walau panel kecil & fallback half-block.
- **Komposisi bust** (kepala + bahu), **terpusat**, menghadap kamera.
- Subjek menonjol dari latar.

## Style suffix (dipakai semua karakter)

```
sci-fi character portrait, head and shoulders bust, centered, facing forward,
dramatic rim lighting, strong single key light, pure black background,
high contrast, detailed face, painterly concept art, cinematic, sharp focus
```

`generate.py` menggabung: `positive = "<subject>, <style suffix>"`. Field `style` & `negative`
disimpan di [`characters.json`](characters.json) agar mudah di-tweak sekali untuk semua.

## Negative (referensi)

Z-Image Turbo pakai cfg=1 → negatif **tidak berpengaruh** (conditioning di-zero-out). Field
`negative` di `characters.json` disediakan untuk **opsi cloud/SDXL** yang memakai CFG > 1:

```
blurry, low contrast, busy background, cluttered background, bright background, washed out,
watermark, text, logo, frame, multiple people, extra limbs, deformed, mutated, lowres
```

## Catatan konsistensi

- Seed tetap per-karakter (reproducible) — ubah seed di `characters.json` bila ingin variasi.
- Ukuran default 1024×1024 (atau set `--size 832x1216` untuk bust lebih tinggi).
- Untuk gaya seragam antar 21 karakter, jangan ubah style suffix; cukup ubah `subject`.
