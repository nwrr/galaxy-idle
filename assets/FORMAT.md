# Asset Format Spec

## Tipe File

| Ext | Isi | Kapan dipakai |
|-----|-----|---------------|
| `.ans` | Art dengan ANSI escape (`\x1b[..m`) tertanam — **berwarna** | Sprite celestial (warna intrinsik biome) & art multi-warna |
| `.txt` | Unicode art monokrom (warna diberi saat render via theme) | Banner (logo ANSI-shadow), frame UI struktural |
| `.ron` | `manifest.ron` saja | Registry asset |

> **Sprite celestial = `.ans` berwarna**: biome (ocean/lava/gas/dst.) harus bisa dibedakan dari
> warna, jadi warna **intrinsik ke asset** — bukan diturunkan theme. Banner/frame tetap `.txt`
> (struktur murni; warna dari theme `ui/theme.rs`).

## Aturan Bentuk

- **Encoding:** UTF-8. Box-drawing (`┌─┐│└┘├┤┬┴┼`) & emoji diizinkan (lihat `06-ui.md`).
- **Lebar = jumlah kolom tampil**, bukan byte. Emoji/CJK = **2 sel**; hitung lebar pakai
  *display width* (mis. crate `unicode-width` saat implementasi), bukan `.len()`.
- **Aspek sel terminal ≈ 2:1** (tinggi:lebar). Sprite yang harus tampak "bulat" (planet) digambar
  lebar ~2× tinggi. Konsisten dengan `ASPECT=2.0` di `14`/`15`.
- **Tanpa trailing spaces** kecuali sengaja untuk padding bentuk; jaga lebar tiap baris konsisten
  (pad dengan spasi agar persegi-panjang rapi).
- **Ukuran maksimum** harus muat di panel target terkecil yang memakainya (cek breakpoint `06-ui.md`:
  Full ≥100, Compact ≥70, Minimal <70). Banner besar → sediakan varian kecil.

## `.ans` (berwarna) — sprite celestial

- **Half-block** `▀`/`▄`: tiap sel = 2 piksel vertikal (fg = piksel atas, bg = piksel bawah) →
  resolusi vertikal 2× + warna penuh. Grid piksel **persegi** (W=cols, H=2·rows) → bola tampak bulat.
- Hanya SGR color codes; fg `\x1b[38;2;r;g;bm` (truecolor) / `\x1b[38;5;Nm` (256) / `\x1b[3x;9xm`
  (16). **Hindari** kontrol kursor/clear. Reset (`\x1b[0m`) di tiap akhir sel/baris (tak "bocor").
- **Fallback color-depth:** tiap base disediakan 3 varian depth — `tc` (24-bit) / `256` / `16` —
  penamaan `sprites/<base>/<size>.<depth>.ans` (1 sub-folder per base, ≤10 file). Runtime deteksi
  kapabilitas terminal → pilih depth.
- **Responsif:** 3 varian ukuran — `sm` 20×10, `md` 40×20, `lg` 64×32 (sel) — runtime pilih per luas
  panel. Render procedural langsung di Rust (math sama dgn `gen_assets.py`) untuk resize mulus (M9);
  `.ans` jadi sumber/preview.
- Loader mem-parse SGR → `ratatui::Style` (crate `ansi-to-tui` atau helper setara).

## `manifest.ron`

```ron
// Vec<AssetEntry>
[
    ( id: "banner_title",  path: "banner/title.txt",              kind: Banner, w: 72, h: 14 ),
    ( id: "planet_terran", path: "sprites/planet_terran/lg.tc.ans", kind: Sprite, w: 64, h: 32 ),
    ( id: "ship",          path: "sprites/ship/lg.tc.ans",          kind: Sprite, w: 64, h: 32 ),
    ( id: "frame_full",    path: "ui/frame_full.txt",              kind: Frame,  w: 0,  h: 0  ),
]
// Sprite path = varian canonical <base>/lg.tc.ans; varian lain di folder yg sama
// `sprites/<base>/<size>.<depth>.ans` (size sm|md|lg, depth tc|256|16) diturunkan runtime.
```

```rust
pub struct AssetEntry { pub id: String, pub path: String, pub kind: AssetKind, pub w: u16, pub h: u16 }
pub enum AssetKind { Banner, Sprite, Frame, Portrait, Misc }
```

### Ukuran Kanonik

| Kategori | Ukuran (w×h sel) | Catatan |
|----------|------------------|---------|
| Banner besar | 72×14 | splash/menu utama (logo ANSI-shadow) |
| Banner kecil | 44×5 | layout Compact/Minimal |
| Sprite celestial `lg` (planet/star/ship) | 64×32 | ANSI half-block berwarna; grid piksel 64×64 persegi → bulat |
| Sprite celestial `md` | 40×20 | panel sedang |
| Sprite celestial `sm` | 20×10 | panel sempit |
| Portrait karakter | **PNG** (area tampil ~44×32 sel) | bust NPC — dirender via **ratatui-image** |
| Frame UI | dinamis (0×0) | full/compact/minimal |

> Tiap base celestial = **9 file**: {sm,md,lg} × {tc,256,16}. Half-block → tiap sel 2 piksel
> vertikal, jadi grid piksel persegi (lg 64×64) bukan 2:1; bola tetap bulat karena sel terminal ~2:1.

### Karakter = gambar (bukan ASCII)

Wajah butuh resolusi yang tak tercapai pada grid sel kecil → **karakter dirender sebagai gambar
asli** via crate **`ratatui-image`** (protokol Sixel/Kitty/iTerm2, fallback Unicode half-block).
`AssetKind::Portrait` ⇒ `path` menunjuk `.png`; `w/h` = **area tampil** widget (sel), bukan resolusi
piksel. Runtime PNG (512×512) di-publish dari `assets/source/characters/` oleh `gen_assets.py`.
Lihat [`../abstraction/design/06-ui.md`](../abstraction/design/06-ui.md) & [`07-architecture.md`](../abstraction/design/07-architecture.md).

### Generasi celestial (numpy → half-block ANSI)

`scripts/gen_assets.py`: celestial **prosedural** (numpy: sphere shading + noise biome → **RGB**
per-biome) → encode `to_halfblock()` jadi `.ans` berwarna, 9 varian per base. Banner = logo
ANSI-shadow tangan (tak di-generate). Portrait **tidak** di-ASCII; hanya disalin+di-downscale jadi
runtime PNG (dirender via ratatui-image).

- `w`/`h` = *display width/height* (sel) varian canonical (lg = 64×32). `0` = dinamis (frame).
- `check_assets.py` strip ANSI escape sebelum ukur; verifikasi tiap base punya **9 varian** lengkap,
  WARN bila underfill <60%, FAIL bila overflow box atau varian hilang.
- Loader memvalidasi: file ada, dimensi cocok (baris ≤ `h`, lebar tampil ≤ `w`) → gagal-cepat saat
  startup (sejajar `content.rs::validate`).

## Versi & Lisensi

- Semua asset buatan sendiri (ANSI/teks prosedural), tanpa konten pihak ketiga → bebas lisensi.
- Bila mengimpor referensi (SVG/PNG) ke `source/`, catat sumber di `source/README.md`.
