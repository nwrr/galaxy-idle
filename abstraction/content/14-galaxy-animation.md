# 14 — Galaxy Animation (Menu Utama)

> Perhitungan render galaksi spiral di MAIN VIEW menu utama, dianimasikan tiap detik. **Murni
> kosmetik** — tidak menyentuh `GameState`; berjalan di loop render ([`07-architecture.md`](../design/07-architecture.md)),
> memakai `render_frame`/waktu nyata, bukan tick simulasi. Melengkapi §animasi di
> [`06-ui.md`](../design/06-ui.md). Render via `ratatui` (buffer cell) atau `Canvas` widget.

## Model: Galaksi Spiral Logaritmik

Bintang ditempatkan dalam koordinat **polar** `(r, θ)` lalu diproyeksikan ke grid terminal. Lengan
spiral mengikuti **spiral logaritmik**:

```
r(θ) = a · e^(b·θ)        ⇔        θ_arm(r) = (1/b) · ln(r/a)
```

- `a` = skala awal, `b` = ketat-longgar lengan (pitch). `b≈0.25` → spiral klasik.
- `ARM_COUNT` (mis. 2 atau 4): lengan ke-`k` digeser fase `θ + k·(2π/ARM_COUNT)`.

## Pembuatan Bintang (sekali, saat masuk menu)

Deterministik dari seed menu (boleh acak; tidak disimpan). Untuk `i ∈ 0..N` (`N ≈ 400–800`):

```
r_i   = R_MAX · sqrt(rng.f64())        // sqrt → densitas seragam per luas (bukan menumpuk di pusat)
arm   = rng.range(0, ARM_COUNT)
θ_arm = (1/b)·ln(r_i/a) + arm·(2π/ARM_COUNT)
θ_i   = θ_arm + gauss(0, SCATTER·(1 - r_i/R_MAX))   // sebar; makin ke tepi makin rapat ke lengan
bright_i = lerp(0.4, 1.0, rng.f64())   // kecerahan dasar
phase_i  = rng.f64()·2π                 // fase twinkle
temp_i   = arm_temperature(arm, r_i)    // warna (lihat §Warna)
```

`gauss(μ,σ)` = Box–Muller dari dua `rng.f64()`. Inti galaksi (`r` kecil) dibuat lebih padat dengan
menambah ~`N·0.15` bintang pada `r_i = R_CORE·rng.f64()` (bulge).

## Rotasi Diferensial (animasi)

Galaksi berputar; bagian dalam lebih cepat (kurva rotasi mendatar):

```
ω(r) = ω0 / (1 + r/r0)        // rad/detik; ω0 kecil agar lambat (mis. 0.05)
θ_i(t) = θ_i0 + ω(r_i) · t     // t = waktu nyata sejak masuk menu (detik)
```

Karena `ω` turun terhadap `r`, lengan **melengkung makin lama** (winding) — efek galaksi berputar
yang terlihat "hidup setiap detik". Untuk mencegah over-winding pada sesi panjang, boleh modulo
`t` atau pakai `ω` konstan-blok per cincin (TBD estetika).

## Proyeksi ke Grid Terminal

Sel terminal **tidak persegi** (≈ tinggi:lebar 2:1). Koreksi aspek `ASPECT = 2.0`.

```
x =  r·cos(θ)
y =  r·sin(θ) / ASPECT          // kompresi vertikal agar lingkaran tampak bulat
col = round( cx + x · scale )    // cx,cy = pusat panel (kolom/baris)
row = round( cy + y · scale )
scale = min(panel_w, panel_h·ASPECT) / (2·R_MAX)
```

Bintang di luar panel → culling (skip). Bila dua bintang jatuh ke sel sama, ambil `bright` terbesar
(atau akumulasi densitas, lihat §Glyph).

## Twinkle (kerlip per detik)

```
b_i(t) = clamp01( bright_i · (1 + TWINKLE_AMP · sin(2π·TWINKLE_FREQ·t + phase_i)) )
```

`TWINKLE_FREQ ≈ 0.5–1.5 Hz`, `TWINKLE_AMP ≈ 0.3`. Tiap bintang berkelip beda fase → bidang bintang
berkilau tiap detik tanpa terlihat seragam.

## Mapping Glyph (kecerahan → karakter)

```
RAMP = [' ', '·', '.', ':', '*', '✦', '★']   // gelap → terang
idx  = floor( b_i(t) · (RAMP.len()-1) )
glyph = RAMP[idx]
```

Jika memakai **akumulasi densitas** (banyak bintang per sel), petakan jumlah → ramp untuk efek
"awan bintang" di inti.

## Warna (per lengan / temperatur)

```
temp → warna ANSI:
  inti (r kecil)      → Kuning/Putih (panas, padat)
  lengan              → Cyan / Biru muda (bintang muda)
  tepi & sebaran      → Magenta redup (nebula)
```

Konsisten dengan color scheme [`06-ui.md`](../design/06-ui.md). Tiap lengan boleh diberi hue beda agar
struktur spiral terbaca.

> Render **berwarna penuh** (truecolor bila didukung; turun ke 256/16 sesuai kapabilitas terminal,
> sama seperti sprite celestial `.ans`). Bukan glyph monokrom — temperatur dipetakan ke warna RGB,
> lalu di-downscale ke depth terminal. Lihat [`07-architecture.md`](../design/07-architecture.md) §Sprite.

## Loop Render (pseudocode)

```rust
struct GalaxyAnim { stars: Vec<Star>, t0: Instant, params: GalaxyParams }

fn update(&mut self) { /* tidak perlu mutasi; t dihitung saat render */ }

fn render(&self, buf: &mut Buffer, area: Rect) {
    let t = self.t0.elapsed().as_secs_f64();
    let (cx, cy, scale) = projection(area);
    let mut cells = vec![DensityCell::default(); area.area()];
    for s in &self.stars {
        let theta = s.theta0 + omega(s.r) * t;
        let (col, row) = project(s.r, theta, cx, cy, scale);
        if !area.contains(col, row) { continue; }              // culling
        let b = brightness(s, t);                              // twinkle
        cells[idx(col,row)].add(b, s.temp);                    // akumulasi densitas
    }
    for (pos, cell) in cells.iter().enumerate() {
        let glyph = RAMP[ramp_index(cell.density)];
        buf.set(pos, glyph, color_for(cell.temp, cell.density));
    }
    // overlay: kapal 🚀 bergerak di route bila ShipStatus::Traveling (posisi = elapsed/total)
}
```

Dipanggil tiap `RENDER_INTERVAL` (~75 ms, [`08-balancing.md`](../design/08-balancing.md)) → ~13 FPS.

## Konstanta (ke `08-balancing.md`)

| Const | Nilai | Catatan |
|-------|-------|---------|
| `STAR_COUNT` | 600 | bintang di menu galaxy |
| `ARM_COUNT` | 2 | jumlah lengan spiral |
| `SPIRAL_B` | 0.25 | pitch lengan |
| `R_MAX` | 1.0 | radius ternormalisasi |
| `R_CORE` | 0.2 | radius bulge inti |
| `OMEGA0` | 0.05 rad/s | kecepatan rotasi inti |
| `OMEGA_R0` | 0.3 | skala penurunan ω(r) |
| `SCATTER` | 0.3 | sebaran sudut dari lengan |
| `ASPECT` | 2.0 | rasio sel terminal |
| `TWINKLE_FREQ` | 1.0 Hz | frekuensi kerlip |
| `TWINKLE_AMP` | 0.3 | amplitudo kerlip |

## Perf

- `O(STAR_COUNT)` per frame; 600 bintang × 13 FPS = ~8k op/dtk → murah.
- Buffer densitas dialokasi sekali (reuse), bukan per frame.
- Bila panel kecil (`Minimal`, [`06-ui.md`](../design/06-ui.md)), turunkan `STAR_COUNT` proporsional luas
  panel agar tidak over-draw.
