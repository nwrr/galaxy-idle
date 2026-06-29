# Sumber Gambar Karakter (untuk portrait)

Taruh gambar sumber di sini. `scripts/gen_assets.py` menyalin + men-downscale jadi runtime PNG
(`assets/characters/<group>/*.png`, 512×512) yang **dirender sebagai gambar** via `ratatui-image`
(bukan ASCII — wajah butuh resolusi). Penamaan **harus** cocok:

```
assets/source/characters/
  male/   char_male_01.png   ... char_male_07.png
  female/ char_female_01.png ... char_female_07.png
  alien/  char_alien_01.png  ... char_alien_07.png
```

## Tips gambar agar hasil bagus
- **Latar gelap solid**, subjek terang & kontras tinggi → tampil rapi di panel TUI gelap dan di
  fallback half-block.
- Wajah/bust **terpusat**, menghadap kamera. Resolusi tinggi bagus (di-downscale ke 512×512).
- Format apa pun yang didukung Pillow (`.png`, `.jpg`, `.webp`).

## Publish ke runtime
```
python3 scripts/gen_assets.py all                 # salin+downscale semua → assets/characters/<group>/*.png
python3 scripts/check_assets.py                   # verifikasi (portrait dicek sbg image)
```

File di folder `source/` ini = **master** (boleh resolusi penuh). Runtime memakai PNG hasil
publish di `assets/characters/<group>/*.png` yang dirender via `ratatui-image`.
