# VISUAL_TARGETS — Peta View → Referensi + Kriteria "Layak"

Dipakai gerbang **(S)** di [`CHECKLIST.md`](CHECKLIST.md): agent `scripts/screenshot.sh <view> <w> <h>`
→ **Read PNG** → bandingkan ke referensi & kriteria di sini → iterasi sampai layak. Referensi di
[`../../assets/source/references/`](../../assets/source/references/).

## Referensi Utama

| Ref | Apa | Dipakai untuk |
|-----|-----|---------------|
| `riftborne/42UN3F.png` | Colony view (3 kolom, ANSI city art, RESOURCES/OPTIONS/MARKET) | Shell U3, planet U4, merchant U8 |
| `riftborne/owOev1.png` | Star Map (grid tile berwarna, kursor, SELECTED TILE) | Galaxy starmap U6 |
| `galaxy_2.png` | Particle galaxy spiral (core glow, lengan, gradien biru) | Galaxy backdrop U6, galaxy pixel |
| `6-galaksi-bima-sakti.jpg` | Foto galaksi nyata | Acuan warna/bentuk backdrop |
| `riftborne/*` (lain) | View Riftborne tambahan | Kepadatan info, research, log |

## Kriteria "Layak" per View

Skala nilai agent tiap screenshot (catat di ITERATION_LOG): **Layout** (struktur/proporsi),
**Density** (info padat tapi terbaca), **Color** (theme konsisten, kontras cukup), **Asset** (sprite/
portrait/galaxy tampil benar), **Polish** (alignment, spacing, no overflow). Item lulus bila semua
≥ "baik" dan tak ada cacat blocker (panel kosong, teks terpotong, warna hancur).

### Shell global (semua view)
- 3 kolom persisten: kiri RESOURCES+status, tengah MAIN, kanan OPTIONS; top status bar; footer MARKET.
- Kepadatan setara `riftborne/42UN3F.png`: banyak data, tetap rapi & terbaca.
- Border/sekat berwarna konsisten; item menu aktif ter-highlight.

### Planet / Colony (U4) — ref `42UN3F.png`
- Sprite biome planet tampil di MAIN (warna biome jelas beda).
- List RESOURCE NODES + FACTORIES padat, angka rata-kanan, afford indicator berwarna.
- Aksi kontekstual di status bar; slot/node terpilih ter-highlight.

### Research (U5)
- Tree tech tervisual (node status locked/available/active/done berwarna), progress bar riset aktif,
  detail tech terpilih, connector dependency.

### Galaxy (U6) — KRUSIAL — ref `galaxy_2.png` + `owOev1.png`
- **Backdrop:** spiral partikel berwarna, core terang putih-biru → tepi biru gelap, lengan terlihat,
  twinkle + rotasi halus, mengisi area apa pun. Mirip `galaxy_2.png`.
- **Starmap:** grid tile sistem berwarna (faksi/biome), kursor jelas + koordinat, panel SELECTED TILE,
  legend, marker home/ship/target, minimap. Playable: kursor gerak, select, scan, kirim ship.
- **Pixel (sixel/kitty):** galaxy PNG tajam saat didukung (audit `capture_term.sh`), fallback procedural.

### Warp/Travel (U7)
- Rute origin→target, progress bar, ship status + sprite, particle exhaust saat Traveling, ETA/jarak.

### Merchant + Events/Log (U8) — ref `42UN3F.png` (kolom)
- Portrait pedagang tampil; daftar offer + harga + afford; timer; flavor text. Log: entri berwarna,
  terbaru, scroll, badge unread.

### Theme (U9)
- Default/HighContrast/Mono ketiganya terbaca; depth tc/256/16 sprite tetap layak.

## Catatan
- Rasterizer (Buffer→PNG) **tak** menampilkan sixel/kitty → galaxy pixel dinilai via
  `scripts/capture_term.sh`, bukan `screenshot.sh`.
- "Layak" = penilaian agent + selera user; bila ragu/major-redesign → tanya user (Stop Condition LOOP).
