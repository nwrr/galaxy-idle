# Assets — `galaxy-idle`

Asset statis untuk TUI. **Runtime = teks** (Unicode + ANSI berwarna); karakter = PNG via
ratatui-image. Celestial = **ANSI half-block berwarna** (.ans), bukan ASCII mono.

## Keputusan Format (final)

| Pilihan | Verdict | Alasan |
|---------|---------|--------|
| **Unicode text + ANSI berwarna** | ✅ **dipakai** | Portabel lintas terminal, diffable di git, bisa dibaca & diverifikasi agent (snapshot), tanpa dependency protokol grafis |
| Image-to-terminal (sixel/kitty/chafa) | ❌ untuk celestial | Rapuh utk sprite umum; **dipakai khusus portrait** karakter via ratatui-image (fallback half-block) |
| ASCII monokrom | ❌ | Biome **tak bisa dibedakan** tanpa warna → celestial pakai **ANSI half-block berwarna** |

> **Galaxy spiral & particle = prosedural** (dihitung saat render, lihat
> [`abstraction/content/14-galaxy-animation.md`](../abstraction/content/14-galaxy-animation.md) &
> [`15-particle-effects.md`](../abstraction/content/15-particle-effects.md)). Itu **bukan** asset statis dan
> tidak ada di folder ini. Asset di sini = hal statis: banner/logo, sprite planet/ship, frame UI,
> tabel glyph.

> **Karakter = GAMBAR.** Wajah butuh resolusi → portrait karakter dirender sebagai PNG asli via
> **`ratatui-image`** (Sixel/Kitty/iTerm2, fallback half-block). **Celestial = ANSI half-block
> berwarna** (.ans, 9 varian: sm/md/lg × tc/256/16). Banner/frame = teks. Lihat [`FORMAT.md`](FORMAT.md).

## Struktur

```
assets/
  README.md          # dokumen ini
  FORMAT.md          # spec teknis .txt/.ans + skema manifest
  manifest.ron       # registry asset (id → path, kind, dimensi)
  banner/            # logo & splash besar+kecil (.txt)
  sprites/           # celestial: planet per biome, star, ship — ANSI half-block (.ans, 9 varian/base)
  characters/        # portrait NPC: male/ female/ alien/ (.PNG, dirender via ratatui-image)
  ui/                # template frame/border layout full/compact/minimal (.txt)
  icons/             # glyphs.md — tabel icon/emoji → makna + warna
  source/            # SVG/PNG desain (REFERENSI saja, TIDAK di-load runtime)
```

## Cara Pakai (runtime)

1. `manifest.ron` di-load saat startup (loader gaya `content.rs`,
   [`abstraction/design/07-architecture.md`](../abstraction/design/07-architecture.md)).
2. Asset `.txt`/`.ans` di-`include_str!` (single-binary) atau dibaca dari disk (modding).
3. Render: tulis baris asset ke `ratatui::Buffer` pada `Rect` yang sesuai. Hormati lebar/tinggi dari
   manifest (jangan overflow panel).

## Cara Generate Asset (cara utama)

Asset **di-generate**, bukan digambar tangan, lewat `scripts/gen_assets.py`:

```
pip install -r requirements.txt          # numpy + pillow (sekali)
python3 scripts/gen_assets.py all         # regen 99 .ans celestial; publish portrait dari gambar yg ada
python3 scripts/check_assets.py           # verifikasi dimensi + kelengkapan 9 varian vs manifest
```

- **Celestial** (planet/star/ship) = render **prosedural** (numpy): bola ber-shading + **warna RGB
  per-biome** → encode **ANSI half-block** (.ans) berwarna, 9 varian (sm/md/lg × tc/256/16).
- **Banner** = logo ANSI-shadow (di-author tangan; `all` tak menimpanya).
- **Karakter** = konversi gambar (PNG via ratatui-image). Taruh PNG di
  [`source/characters/`](source/characters/README.md) lalu jalankan `all`.

## Cara Menambah Asset Baru

1. Tambahkan job di `scripts/gen_assets.py` (atau drop gambar sumber utk portrait).
2. Daftarkan di `manifest.ron` (`id`, `path`, `kind`, `w`, `h`) — patuhi ukuran di [`FORMAT.md`](FORMAT.md).
3. `gen_assets.py all` → `check_assets.py` (harus PASS, tanpa underfill).
4. Tambah glyph ke [`icons/glyphs.md`](icons/glyphs.md) bila relevan.

## Peran Looping Agent

Agent boleh **membuat/memperbaiki asset** saat loop. Setelah mengubah asset yang tampil di UI, agent
**wajib** render snapshot & cek [`agent/test/visual_checks.md`](../agent/test/visual_checks.md) untuk
memastikan tidak ada overflow/misalignment. SVG/PNG di `source/` jangan pernah di-`include_str!`.
