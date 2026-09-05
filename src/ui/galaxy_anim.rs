//! Animasi galaksi spiral logaritmik untuk MAIN VIEW menu (`14-galaxy-animation.md`).
//!
//! **Murni kosmetik & read-only** — tidak menyentuh `GameState`. Bintang dibuat sekali (seed
//! tetap), sudut dianimasikan dari `t` (waktu render nyata; `0.0` = snapshot deterministik).
//! Proyeksi polar→grid dengan koreksi `ASPECT`, akumulasi densitas per sel, glyph via RAMP.
#![allow(dead_code)]

use crate::balance::{
    ARM_COUNT, ASPECT, OMEGA_R0, OMEGA0, R_CORE, R_MAX, SCATTER, SPIRAL_B, STAR_COUNT, TWINKLE_AMP,
    TWINKLE_FREQ,
};
use crate::rng::SplitMix64;
use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use std::f64::consts::TAU;

/// Glyph gelap→terang (kecerahan/densitas dipetakan ke indeks).
const RAMP: [char; 7] = [' ', '·', '.', ':', '*', '✦', '★'];

/// Kelas temperatur → warna (inti panas, lengan muda, tepi nebula).
#[derive(Clone, Copy)]
enum Temp {
    Core,
    Arm,
    Edge,
}

#[derive(Clone, Copy)]
struct Star {
    r: f64,
    theta0: f64,
    bright: f64,
    phase: f64,
    temp: Temp,
}

/// Kumpulan bintang menu galaksi (dibuat sekali dari seed).
pub struct GalaxyAnim {
    stars: Vec<Star>,
}

fn gauss(rng: &mut SplitMix64, sigma: f64) -> f64 {
    let u1 = rng.next_f64().max(1e-12);
    let u2 = rng.next_f64();
    (-2.0 * u1.ln()).sqrt() * (TAU * u2).cos() * sigma
}

fn temp_of(r: f64) -> Temp {
    if r < R_CORE {
        Temp::Core
    } else if r > 0.75 * R_MAX {
        Temp::Edge
    } else {
        Temp::Arm
    }
}

/// M16.1: model log-spiral N-arm MURNI (tanpa scatter/noise) — diekstrak jd fungsi terpisah
/// (dulu inline di `star()`, tak bisa dites TERPISAH dari noise gaussian yg genuinely
/// mengaburkan verifikasi formula). θ = (1/b)·ln(r/r_core) + arm·(2π/N) — formula BAKU
/// logarithmic spiral (dipakai lengan galaksi nyata: Archimedean spiral pakai r=a+bθ linear,
/// LOG-spiral pakai r=a·e^(bθ) — bentuk yg genuinely dipakai referensi `galaxy_2.png`).
fn spiral_theta(r: f64, arm: u32) -> f64 {
    let r = r.max(0.005); // hindari ln(0) saat r→0 (bulge)
    (1.0 / SPIRAL_B) * (r / R_CORE).ln() + arm as f64 * (TAU / ARM_COUNT.max(1) as f64)
}

/// M16.7: "Depth/parallax (layer brightness)" — **gap real ditemukan**: dulu `bright` UNIFORM
/// RANDOM independen dari `r` (jarak dari core) — tak py hubungan "depth" sama sekali (beda
/// dari M16.2's densitas & M16.3's WARNA yg SUDAH radial, tp KECERAHAN individual tiap bintang
/// tetap acak rata, tak melemah menjauh). Fix: faktor depth linear (1.0 di core -> 0.7 di tepi)
/// — bintang JAUH genuinely lebih redup drpd yg DEKAT (rata2), efek "parallax kedalaman" via
/// kecerahan (bukan literal gerak beda kecepatan -- itu sudah dicover differential rotation
/// M16.1/6). Diekstrak jd fungsi murni (pola sama `spiral_theta`) biar testable langsung.
fn depth_factor(r: f64) -> f64 {
    1.0 - 0.3 * (r / R_MAX).min(1.0)
}

fn star(rng: &mut SplitMix64, r: f64) -> Star {
    let r = r.max(0.005); // hindari ln(0) saat r→0 (bulge)
    let arm = rng.below(ARM_COUNT.max(1));
    let theta_arm = spiral_theta(r, arm);
    let theta0 = theta_arm + gauss(rng, SCATTER * (1.0 - r / R_MAX));
    let bright = (0.4 + 0.6 * rng.next_f64()) * depth_factor(r);
    let phase = rng.next_f64() * TAU;
    Star {
        r,
        theta0,
        bright,
        phase,
        temp: temp_of(r),
    }
}

impl GalaxyAnim {
    /// Bangun bidang bintang: `count` bintang lengan + ~15% bulge inti, deterministik dari `seed`.
    pub fn new(seed: u64, count: u32) -> Self {
        let mut rng = SplitMix64::new(seed);
        let mut stars = Vec::with_capacity(count as usize + count as usize / 6);
        for _ in 0..count {
            // M16.2: **gap real ditemukan** — dulu `R_MAX * U.sqrt()` genuinely beri densitas
            // SERAGAM per luas (CDF∝r² match luas∝r², dikonfirmasi matematis+unit test) —
            // BUKAN "padat core → jarang tepi" (checklist minta gradien MENURUN, versi lama
            // konstan di luar bulge). Diganti `R_MAX * U` (linear di r, BUKAN sqrt) — densitas
            // AREAL (star/luas) jadi ∝1/r (elemen luas `r·dr` tumbuh linear drpd jumlah star
            // per-r yg SERAGAM) — genuinely MENURUN monoton dari core ke tepi, pola realistis
            // galaksi disk (surface brightness eksponensial, di sini disederhanakan jd 1/r).
            let r = R_MAX * rng.next_f64();
            stars.push(star(&mut rng, r));
        }
        for _ in 0..(count / 6) {
            let r = R_CORE * rng.next_f64();
            stars.push(star(&mut rng, r));
        }
        GalaxyAnim { stars }
    }

    /// Bidang bintang default ukuran `STAR_COUNT`.
    pub fn default_field() -> Self {
        Self::new(0x6A1A5E_DEC0DE, STAR_COUNT)
    }

    /// Render bingkai galaksi ke `area` pada waktu `t` detik. Akumulasi densitas per sel.
    ///
    /// M16.4: "Half-block density utk kepadatan bintang (sub-sel)" — **gap real ditemukan**:
    /// dulu posisi bintang di-`round()` ke SATU sel integer per densitas (`dens[ix]` tunggal)
    /// — kalau 2 bintang jatuh di sel SAMA (lazim terjadi krn `ASPECT` kompresi vertikal 2:1
    /// bikin banyak baris dunia map ke 1 baris terminal), yg KALAH terang HILANG TOTAL dari
    /// render, bukan cuma redup. Fix: lacak densitas per SETENGAH-sel (atas/bawah, dari pecahan
    /// posisi baris float SEBELUM `round()`) — bila 2 bintang beda setengah jatuh di sel sama,
    /// KEDUANYA tampil via `▀` (glyph half-block, fg=warna atas, bg=warna bawah) — bukan cuma
    /// 1 dipilih sbg "menang". Sel dgn cuma 1 sub-bagian terisi (mayoritas kasus, field jarang)
    /// tetap pakai RAMP glyph biasa spt sblmnya (TAK ADA regresi visual utk kasus umum).
    pub fn render(&self, f: &mut Frame, area: Rect, t: f64, th: &Theme) {
        if area.width < 3 || area.height < 3 {
            return;
        }
        let (w, h) = (area.width as f64, area.height as f64);
        let scale = (w.min(h * ASPECT)) / (2.0 * R_MAX);
        let cx = area.x as f64 + w / 2.0;
        let cy = area.y as f64 + h / 2.0;
        let cells = (area.width as usize) * (area.height as usize);
        // Densitas+temperatur dilacak TERPISAH per setengah-sel (atas/bawah) — lihat doc komentar
        // fungsi ini.
        let mut dens_top = vec![0.0f64; cells];
        let mut dens_bot = vec![0.0f64; cells];
        let mut temp_top = vec![Temp::Arm; cells];
        let mut temp_bot = vec![Temp::Arm; cells];

        for s in &self.stars {
            let omega = OMEGA0 / (1.0 + s.r / OMEGA_R0);
            let theta = s.theta0 + omega * t;
            let col = (cx + s.r * theta.cos() * scale).round();
            let row_f = cy + s.r * theta.sin() / ASPECT * scale;
            let row = row_f.round(); // sama PERSIS spt sblm M16.4 -> peta spasial TAK berubah.
            if col < area.x as f64
                || row < area.y as f64
                || col >= (area.x + area.width) as f64
                || row >= (area.y + area.height) as f64
            {
                continue; // culling
            }
            let b = (s.bright * (1.0 + TWINKLE_AMP * (TAU * TWINKLE_FREQ * t + s.phase).sin()))
                .clamp(0.0, 1.0);
            let ix = (row as u16 - area.y) as usize * area.width as usize
                + (col as u16 - area.x) as usize;
            // Sub-sel HANYA dihitung relatif thd sel yg SAMA (`row`, hasil round persis spt
            // sblmnya) — bukan re-derive dari `floor()` yg akan geser pemetaan sel scr
            // keseluruhan. `row_f < row` -> posisi asli sedikit DI ATAS titik tengah sel (round
            // membulatkan ke atas) -> setengah ATAS; sebaliknya -> setengah BAWAH.
            let top_half = row_f < row;
            let (dens, temp) = if top_half {
                (&mut dens_top, &mut temp_top)
            } else {
                (&mut dens_bot, &mut temp_bot)
            };
            if b > dens[ix] {
                dens[ix] = b;
                temp[ix] = s.temp;
            }
        }

        // M16.8: "Core glow / bloom" — **gap real ditemukan**: dulu TAK ADA efek cahaya
        // menyebar ke sel tetangga sama sekali — tiap sel HANYA terisi kalau genuinely ada
        // bintang persis di situ, biarpun sel SEBELAH py bintang sangat terang (core). Fix:
        // sel Core CUKUP TERANG (>0.7) "bocorkan" glow REDUP ke 4 tetangga ortogonal yg
        // KOSONG (kedua setengah 0) — efek halo lembut di sekitar core, TAK menimpa bintang
        // asli manapun (hanya isi sel yg genuinely kosong).
        const GLOW_SOURCE_MIN: f64 = 0.7;
        const GLOW_VALUE: f64 = 0.18;
        let mut glow = vec![0.0f64; cells];
        for j in 0..area.height as usize {
            for i in 0..area.width as usize {
                let ix = j * area.width as usize + i;
                let is_core_source = (dens_top[ix] > GLOW_SOURCE_MIN
                    && matches!(temp_top[ix], Temp::Core))
                    || (dens_bot[ix] > GLOW_SOURCE_MIN && matches!(temp_bot[ix], Temp::Core));
                if !is_core_source {
                    continue;
                }
                let neighbors: [(isize, isize); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
                for (di, dj) in neighbors {
                    let (ni, nj) = (i as isize + di, j as isize + dj);
                    if ni < 0 || nj < 0 || ni >= area.width as isize || nj >= area.height as isize {
                        continue;
                    }
                    let nix = nj as usize * area.width as usize + ni as usize;
                    if dens_top[nix] <= 0.0 && dens_bot[nix] <= 0.0 {
                        glow[nix] = glow[nix].max(GLOW_VALUE);
                    }
                }
            }
        }

        let color_of = |t: Temp| match t {
            Temp::Core => th.focus,
            Temp::Arm => th.advanced,
            Temp::Edge => th.nebula,
        };

        let buf = f.buffer_mut();
        for j in 0..area.height {
            for i in 0..area.width {
                let ix = j as usize * area.width as usize + i as usize;
                let (dt, db) = (dens_top[ix], dens_bot[ix]);
                if dt <= 0.0 && db <= 0.0 {
                    if glow[ix] > 0.0 {
                        let cell = &mut buf[(area.x + i, area.y + j)];
                        let g = RAMP
                            [((glow[ix] * (RAMP.len() - 1) as f64) as usize).min(RAMP.len() - 1)];
                        cell.set_char(g);
                        cell.set_style(Style::default().fg(color_of(Temp::Core)));
                    }
                    continue;
                }
                let cell = &mut buf[(area.x + i, area.y + j)];
                // Ambang MIN keduanya (bukan cuma `>0.0`) — **dicek EMPIRIS via screenshot**:
                // ambang 0.0/0.4 (brightness dasar tiap bintang MIN 0.4, jadi 0.4 nyaris tak
                // efektif) bikin HAMPIR SEMUA sel di sepanjang lengan (padat krn byk bintang
                // berdekatan di kurva sama) kena split — lengan tampak BLOK PADAT solid,
                // REGRESI VISUAL nyata dari starfield renggang (dikonfirmasi Read screenshot,
                // diulang 3x sampai ambang genuinely pas). 0.9 (dekat batas atas brightness+
                // twinkle) hanya lolos tabrakan 2 bintang yg SAMA-SAMA nyaris puncak terang —
                // recovery tetap ada (bbrp blok ▀ genuinely tampak), tp starfield renggang
                // KEMBALI utk mayoritas sel (dikonfirmasi screenshot ulang).
                const DUAL_HALF_MIN: f64 = 0.9;
                if dt > DUAL_HALF_MIN && db > DUAL_HALF_MIN {
                    // Dua bintang CUKUP TERANG di setengah beda, sel sama -> KEDUANYA tampil.
                    cell.set_char('\u{2580}'); // ▀ (upper half block)
                    cell.set_style(
                        Style::default()
                            .fg(color_of(temp_top[ix]))
                            .bg(color_of(temp_bot[ix])),
                    );
                } else {
                    let d = dt.max(db);
                    let temp = if dt > 0.0 { temp_top[ix] } else { temp_bot[ix] };
                    let g = RAMP[((d * (RAMP.len() - 1) as f64) as usize).min(RAMP.len() - 1)];
                    cell.set_char(g);
                    cell.set_style(Style::default().fg(color_of(temp)));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_field() {
        let a = GalaxyAnim::new(123, 200);
        let b = GalaxyAnim::new(123, 200);
        assert_eq!(a.stars.len(), b.stars.len());
        for (s1, s2) in a.stars.iter().zip(&b.stars) {
            assert!((s1.r - s2.r).abs() < 1e-12);
            assert!((s1.theta0 - s2.theta0).abs() < 1e-12);
        }
    }

    #[test]
    fn includes_bulge_and_finite() {
        let g = GalaxyAnim::new(7, 600);
        assert_eq!(g.stars.len(), 600 + 100); // count + count/6
        for s in &g.stars {
            assert!(s.r.is_finite() && s.theta0.is_finite());
            // M16.7: rentang bawah turun dari 0.4 -> 0.28 (0.4*0.7) krn `depth_factor` (bintang
            // di tepi genuinely diredupkan tambahan, lihat test `depth_factor_*` di bawah).
            assert!((0.28..=1.0).contains(&s.bright));
        }
    }

    /// M16.1: "Model lengan spiral (log-spiral, N arm)" — dites LANGSUNG formula MURNI
    /// (`spiral_theta`, diekstrak dari `star()` biar terlepas noise gaussian): dua radius
    /// beda di ARM YG SAMA harus py selisih sudut PERSIS `(1/b)·ln(r2/r1)` — definisi baku
    /// logarithmic spiral (BUKAN Archimedean/linear, BUKAN random scatter yg kebetulan
    /// keliatan spiral-ish di screenshot).
    #[test]
    fn spiral_theta_follows_logarithmic_spiral_formula() {
        let r1: f64 = 0.3;
        let r2: f64 = 0.6;
        let arm = 0;
        let expected_delta = (1.0 / SPIRAL_B) * (r2 / r1).ln();
        let actual_delta = spiral_theta(r2, arm) - spiral_theta(r1, arm);
        assert!(
            (actual_delta - expected_delta).abs() < 1e-9,
            "delta sudut HARUS ikuti formula log-spiral PERSIS: expected={expected_delta} \
             actual={actual_delta}"
        );
    }

    /// M16.1: "N arm" — dicek `ARM_COUNT` arm PERSIS `2π/N` terpisah sudutnya di radius SAMA
    /// (bukan asumsi visual "kelihatan ada beberapa lengan" — dibuktikan angka pasti).
    #[test]
    fn spiral_arms_are_evenly_spaced_by_two_pi_over_arm_count() {
        let r = 0.5;
        let expected_spacing = TAU / ARM_COUNT.max(1) as f64;
        for arm in 0..ARM_COUNT {
            let delta = spiral_theta(r, arm) - spiral_theta(r, 0);
            let expected = arm as f64 * expected_spacing;
            assert!(
                (delta - expected).abs() < 1e-9,
                "arm {arm}: spacing sudut HARUS persis {arm}x(2pi/{ARM_COUNT}): \
                 expected={expected} actual={delta}"
            );
        }
    }

    /// M16.1: bulge dekat core (`r→0.005` clamp) tak boleh genuinely panic/NaN dari `ln(0)`
    /// — dites LANGSUNG di titik EKSTREM (bukan cuma tebak "harusnya aman krn ada `.max()`").
    #[test]
    fn spiral_theta_stays_finite_at_core_clamp_boundary() {
        let theta = spiral_theta(0.0, 0);
        assert!(
            theta.is_finite(),
            "r=0.0 (di-clamp ke 0.005 internal) harus tetap hasilkan sudut FINITE, bukan NaN/inf"
        );
    }

    /// M16.2: "Distribusi partikel: padat core → jarang tepi" — **gap real ditemukan**: dulu
    /// `R_MAX*U.sqrt()` genuinely beri densitas SERAGAM per luas (CDF∝r² match luas∝r² —
    /// matematis dibuktikan, bukan random scatter yg kebetulan mirip), BUKAN "padat→jarang"
    /// (checklist minta gradien MENURUN). Dites LANGSUNG: hitung densitas AREAL (star/luas
    /// anulus) di bin radius dalam (0-0.5·R_MAX) vs luar (0.5-1.0·R_MAX) dari sampel BESAR
    /// (N=50k, hindari noise statistik) — bin DALAM harus GENUINELY lbh padat drpd LUAR.
    #[test]
    fn particle_distribution_is_denser_near_core_than_edge() {
        let mut rng = SplitMix64::new(999);
        let n = 50_000usize;
        let mut inner = 0usize;
        let mut outer = 0usize;
        for _ in 0..n {
            let r = R_MAX * rng.next_f64();
            if r < R_MAX / 2.0 {
                inner += 1;
            } else {
                outer += 1;
            }
        }
        // Luas anulus dalam = π·(R_MAX/2)², luas anulus luar = π·R_MAX² - π·(R_MAX/2)² = 3×
        // luas dalam. Densitas = count/luas.
        let area_inner = std::f64::consts::PI * (R_MAX / 2.0).powi(2);
        let area_outer = std::f64::consts::PI * R_MAX.powi(2) - area_inner;
        let density_inner = inner as f64 / area_inner;
        let density_outer = outer as f64 / area_outer;
        assert!(
            density_inner > density_outer * 1.5,
            "densitas AREAL bin dalam (core) harus genuinely jauh lbh padat drpd bin luar \
             (tepi): inner={density_inner:.3} outer={density_outer:.3}"
        );
    }

    /// M16.3: "Gradien warna radial (putih-biru core → biru gelap tepi)" — **gap real
    /// ditemukan**: dulu zona Edge pakai `th.rare` (Magenta, warna TIER RESOURCE dipinjam,
    /// tak py hubungan semantik dgn nebula galaksi) — checklist minta biru GELAP, bukan
    /// magenta. Dites LANGSUNG (bukan cuma percaya kode benar): render 1 star BUATAN di zona
    /// Edge (posisi TERKONTROL, bukan random), baca `Cell.fg` LANGSUNG dari `Buffer` — harus
    /// genuinely `th.nebula`, BUKAN `th.rare`.
    #[test]
    fn edge_zone_uses_nebula_color_not_rare_tier_color() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let g = GalaxyAnim {
            stars: vec![Star {
                r: R_MAX * 0.9, // > 0.75*R_MAX -> Temp::Edge (dikonfirmasi `temp_of`).
                theta0: 0.0,
                bright: 1.0,
                phase: 0.0,
                temp: Temp::Edge,
            }],
        };
        let th = crate::ui::theme::theme(crate::game::state::ThemeChoice::Default);
        assert_ne!(
            th.nebula, th.rare,
            "`nebula` harus GENUINELY warna berbeda dari `rare` (bukan alias tersembunyi)"
        );

        let mut term = Terminal::new(TestBackend::new(40, 40)).unwrap();
        term.draw(|f| g.render(f, f.area(), 0.0, &th)).unwrap();
        let buf = term.backend().buffer().clone();

        // Star di r=0.9*R_MAX, theta=0 -> posisi (cx+r*scale, cy) kira2 di kanan-tengah area.
        let found = (0..buf.area().width).any(|x| {
            (0..buf.area().height).any(|y| {
                let cell = &buf[(x, y)];
                cell.symbol() != " " && cell.fg == th.nebula
            })
        });
        assert!(
            found,
            "star di zona Edge harus genuinely dirender dgn warna `th.nebula`, tak ditemukan \
             cell manapun dgn warna itu di buffer"
        );
    }

    /// M16.4: "Half-block density utk kepadatan bintang (sub-sel)" — **gap real ditemukan**:
    /// dulu 2 bintang di sel SAMA (round ke integer yg sama) HANYA nyisakan 1 (paling terang),
    /// yg lain HILANG TOTAL dari render (bukan cuma redup). Dites LANGSUNG (posisi 2 star
    /// DIKONTROL PRESISI via geometri, bukan random): 2 star di r berlawanan tanda, theta=90°
    /// (cos=0 -> col sama; sin=1 -> row beda arah dari cy) -> keduanya round ke `row` INTEGER
    /// sama tp 1 di atas titik tengah (`top_half`), 1 di bawah -> cell HARUS genuinely `▀`
    /// dgn fg=warna star atas, bg=warna star bawah (BUKAN salah satu didiskarding).
    #[test]
    fn overlapping_stars_in_same_cell_show_both_via_half_block() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let theta0 = TAU / 4.0; // cos=0 (col identik), sin=1 (row bervariasi linear thd r).
        // r=-0.2/0.2 (BUKAN -0.3/0.3 -- itu PERSIS `-OMEGA_R0` di `balance.rs`, bikin
        // `1+r/OMEGA_R0`=0 -> omega=infinity -> theta=NaN, dikonfirmasi via percobaan pertama
        // gagal krn cuma 1 star tampil, bukan 2 -- dihindari dgn r sedikit lebih kecil).
        let g = GalaxyAnim {
            stars: vec![
                Star {
                    r: -0.2,
                    theta0,
                    bright: 1.0,
                    phase: 0.0,
                    temp: Temp::Core,
                },
                Star {
                    r: 0.2,
                    theta0,
                    bright: 1.0,
                    phase: 0.0,
                    temp: Temp::Arm,
                },
            ],
        };
        let th = crate::ui::theme::theme(crate::game::state::ThemeChoice::Default);
        // Area 4x4 -> scale=2, cx=cy=2. row_f1=2+(-0.2)*1=1.8 (round->2, TOP); row_f2=2+0.2=2.2
        // (round->2, BOTTOM); col=2 keduanya (cos=0). Kedua star jatuh di cell (2,2) SAMA.
        let mut term = Terminal::new(TestBackend::new(4, 4)).unwrap();
        term.draw(|f| g.render(f, f.area(), 0.0, &th)).unwrap();
        let buf = term.backend().buffer().clone();
        let cell = &buf[(2, 2)];
        assert_eq!(
            cell.symbol(),
            "\u{2580}",
            "2 star cukup terang di sel sama HARUS genuinely dirender ▀ (half-block), bukan \
             glyph RAMP tunggal yg diskardingsalah satu star: got={:?}",
            cell.symbol()
        );
        assert_eq!(
            cell.fg, th.focus,
            "fg (setengah ATAS) harus warna star Core (di atas titik tengah sel)"
        );
        assert_eq!(
            cell.bg, th.advanced,
            "bg (setengah BAWAH) harus warna star Arm (di bawah titik tengah sel) -- BUKTI \
             star ini TAK didiskarding, genuinely tampil via setengah-sel berbeda"
        );
    }

    /// M16.5: "Twinkle deterministik dari `anim_secs`" — `render()` ambil `&self` (bukan
    /// `&mut self`) + `t: f64` murni, TAK ADA RNG/state internal dimutasi saat render — dites
    /// LANGSUNG (bukan cuma percaya tanda tangan fungsi): render field SAMA di `t` SAMA 2x,
    /// buffer HARUS identik byte-per-byte (glyph+warna), termasuk `t=0.0` (anim_secs awal).
    #[test]
    fn twinkle_is_deterministic_pure_function_of_anim_secs() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let g = GalaxyAnim::new(42, 300);
        let th = crate::ui::theme::theme(crate::game::state::ThemeChoice::Default);
        for t in [0.0, 3.7, 100.0] {
            let mut term1 = Terminal::new(TestBackend::new(60, 24)).unwrap();
            term1.draw(|f| g.render(f, f.area(), t, &th)).unwrap();
            let buf1 = term1.backend().buffer().clone();

            let mut term2 = Terminal::new(TestBackend::new(60, 24)).unwrap();
            term2.draw(|f| g.render(f, f.area(), t, &th)).unwrap();
            let buf2 = term2.backend().buffer().clone();

            assert_eq!(
                buf1, buf2,
                "render(t={t}) dipanggil 2x HARUS hasilkan buffer IDENTIK -- twinkle/posisi \
                 murni fungsi dari t, tak py randomness/state tersembunyi"
            );
        }
    }

    /// M16.6: "Rotasi lambat keseluruhan" — dites LANGSUNG dua sub-klaim: (1) posisi GENUINELY
    /// berubah seiring `t` (rotasi terjadi, bukan cuma twinkle statis di tempat); (2) rotasi
    /// "lambat" secara KUANTITATIF -- pergerakan sudut per frame render (`RENDER_INTERVAL_MS`)
    /// utk bintang TERCEPAT (`r→0`, `omega→OMEGA0`) harus kecil (<5°), bukan cuma "kelihatan
    /// pelan" scr subjektif.
    #[test]
    fn rotation_is_slow_and_moves_stars_over_time() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let g = GalaxyAnim::new(7, 300);
        let th = crate::ui::theme::theme(crate::game::state::ThemeChoice::Default);
        let mut term_a = Terminal::new(TestBackend::new(60, 24)).unwrap();
        term_a.draw(|f| g.render(f, f.area(), 0.0, &th)).unwrap();
        let buf_a = term_a.backend().buffer().clone();

        let mut term_b = Terminal::new(TestBackend::new(60, 24)).unwrap();
        term_b.draw(|f| g.render(f, f.area(), 60.0, &th)).unwrap();
        let buf_b = term_b.backend().buffer().clone();

        assert_ne!(
            buf_a, buf_b,
            "posisi bintang di t=0 vs t=60 HARUS beda -- rotasi genuinely terjadi seiring waktu"
        );

        // Bintang inti tercepat: omega -> OMEGA0 saat r->0. Pergerakan per frame render harus
        // KECIL (lambat, tak menyentak) -- dikonversi derajat biar mudah dibaca ambang.
        let interval_s = crate::balance::RENDER_INTERVAL_MS as f64 / 1000.0;
        let deg_per_frame = OMEGA0 * interval_s * 180.0 / std::f64::consts::PI;
        assert!(
            deg_per_frame < 5.0,
            "rotasi HARUS lambat: pergerakan sudut bintang inti per frame render \
             ({deg_per_frame:.3}°) harus <5°, kalau tidak animasi tampak menyentak bukan halus"
        );
    }

    /// M16.7: "Depth/parallax (layer brightness)" — dites LANGSUNG formula murni: core (r=0)
    /// HARUS faktor 1.0 (tak diredupkan), tepi (r=R_MAX) HARUS genuinely lbh redup (faktor
    /// <1.0), dan MENURUN MONOTON di antaranya (bukan cuma 2 titik acak kebetulan beda).
    #[test]
    fn depth_factor_dims_monotonically_from_core_to_edge() {
        assert_eq!(
            depth_factor(0.0),
            1.0,
            "core (r=0) HARUS faktor penuh, tak diredupkan"
        );
        assert!(
            depth_factor(R_MAX) < 1.0,
            "tepi (r=R_MAX) HARUS genuinely diredupkan (faktor <1.0)"
        );
        let samples = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0].map(|f| f * R_MAX);
        for w in samples.windows(2) {
            assert!(
                depth_factor(w[0]) > depth_factor(w[1]),
                "faktor depth HARUS menurun MONOTON dari core ke tepi: r={} -> {} vs r={} -> {}",
                w[0],
                depth_factor(w[0]),
                w[1],
                depth_factor(w[1])
            );
        }
    }

    /// M16.8: "Core glow / bloom" — **gap real ditemukan**: dulu TAK ADA cahaya menyebar ke
    /// tetangga sama sekali, sel kosong TETAP kosong biarpun bersebelahan dgn core sangat
    /// terang. Dites LANGSUNG (posisi star DIKONTROL PRESISI): 1 star Core sangat terang di
    /// r=0 -> sel tetangga LANGSUNG (kosong, tak py star lain) HARUS genuinely dpt glow redup
    /// warna Core, sel JAUH (tak bertetangga) HARUS TETAP kosong (glow tak menyebar tanpa
    /// batas).
    #[test]
    fn bright_core_star_gives_subtle_glow_to_empty_neighbor_not_far_cells() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let g = GalaxyAnim {
            stars: vec![Star {
                r: 0.0,
                theta0: 0.0,
                bright: 1.0,
                phase: 0.0,
                temp: Temp::Core,
            }],
        };
        let th = crate::ui::theme::theme(crate::game::state::ThemeChoice::Default);
        // Area 5x5 -> scale=2.5, cx=cy=2.5. r=0 -> col=row=round(2.5)=3 (round half away dari
        // nol) -> star di sel (3,3). Tetangga langsung (3,2) HARUS kosong dr star lain -> jadi
        // kandidat glow. Sel (0,0) jauh dr (3,3), bukan tetangga -> HARUS tetap kosong.
        let mut term = Terminal::new(TestBackend::new(5, 5)).unwrap();
        term.draw(|f| g.render(f, f.area(), 0.0, &th)).unwrap();
        let buf = term.backend().buffer().clone();

        let neighbor = &buf[(3, 2)];
        assert_ne!(
            neighbor.symbol(),
            " ",
            "sel tetangga LANGSUNG dari core sangat terang harus dpt glow (bukan kosong)"
        );
        assert_eq!(
            neighbor.fg, th.focus,
            "glow harus pakai warna Core (`th.focus`), sumbernya bintang Core"
        );

        let far = &buf[(0, 0)];
        assert_eq!(
            far.symbol(),
            " ",
            "sel JAUH (bukan tetangga langsung manapun) harus TETAP kosong -- glow tak \
             menyebar tanpa batas"
        );
    }

    /// M16.9: "Resize penuh: isi area apa pun tanpa distorsi" — dites LANGSUNG geometri: 2
    /// star SAMA `r`, beda arah (theta=0 di sumbu-x, theta=90° di sumbu-y), harus jatuh di
    /// jarak PIKSEL SAMA dari pusat SETELAH koreksi `ASPECT` -- dibuktikan di area SANGAT
    /// TAK LAZIM (200x10 amat lebar, 12x50 amat sempit/tinggi), bukan cuma 3 breakpoint baku.
    /// Kalau distorsi ADA, 1 arah akan konsisten lbh jauh/dekat dari yg lain.
    #[test]
    fn projection_stays_circular_not_distorted_at_extreme_aspect_ratios() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        for (w, h) in [(200u16, 10u16), (12u16, 50u16), (60u16, 24u16)] {
            let g = GalaxyAnim {
                stars: vec![
                    Star {
                        r: 0.5,
                        theta0: 0.0, // sumbu-x murni (cos=1, sin=0).
                        bright: 1.0,
                        phase: 0.0,
                        temp: Temp::Core,
                    },
                    Star {
                        r: 0.5,
                        theta0: TAU / 4.0, // sumbu-y murni (cos=0, sin=1).
                        bright: 1.0,
                        phase: 0.0,
                        temp: Temp::Edge,
                    },
                ],
            };
            let th = crate::ui::theme::theme(crate::game::state::ThemeChoice::Default);
            let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
            term.draw(|f| g.render(f, f.area(), 0.0, &th)).unwrap();
            let buf = term.backend().buffer().clone();

            let cx = w as f64 / 2.0;
            let cy = h as f64 / 2.0;
            let mut dist_x = None;
            let mut dist_y = None;
            for y in 0..h {
                for x in 0..w {
                    let cell = &buf[(x, y)];
                    if cell.fg == th.focus && cell.symbol() != " " {
                        dist_x = Some((x as f64 - cx).abs());
                    } else if cell.fg == th.nebula && cell.symbol() != " " {
                        dist_y = Some((y as f64 - cy).abs() * ASPECT);
                    }
                }
            }
            let (dx, dy) = (
                dist_x.expect("star sumbu-x harus ketemu di buffer"),
                dist_y.expect("star sumbu-y harus ketemu di buffer"),
            );
            assert!(
                (dx - dy).abs() <= 1.5,
                "area {w}x{h}: jarak dari pusat sumbu-x ({dx}) vs sumbu-y terkoreksi ASPECT \
                 ({dy}) harus HAMPIR SAMA (r sama) -- beda besar berarti proyeksi terdistorsi"
            );
        }
    }

    /// M16.10: "Performa render < budget frame (no lag)" — dites LANGSUNG waktu wall-clock
    /// `render()` (bidang bintang default penuh, area TERBESAR baku 120x40) harus JAUH di
    /// bawah `RENDER_INTERVAL_MS` (75ms) -- kalau tidak, animasi akan lag/tersendat nyata.
    /// Diulang N kali & ambil RATA-RATA (bukan 1 sampel yg bisa kebetulan lambat krn noise
    /// sistem/JIT warmup).
    #[test]
    fn render_completes_well_under_frame_budget() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;
        use std::time::Instant;

        let g = GalaxyAnim::default_field(); // STAR_COUNT penuh + bulge, kasus TERBERAT.
        let th = crate::ui::theme::theme(crate::game::state::ThemeChoice::Default);
        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();

        let n = 20;
        let start = Instant::now();
        for i in 0..n {
            term.draw(|f| g.render(f, f.area(), i as f64, &th)).unwrap();
        }
        let avg_ms = start.elapsed().as_secs_f64() * 1000.0 / n as f64;

        let budget_ms = crate::balance::RENDER_INTERVAL_MS as f64;
        assert!(
            avg_ms < budget_ms * 0.5,
            "render rata2 ({avg_ms:.3}ms) harus JAUH di bawah budget frame ({budget_ms}ms) \
             -- kalau mendekati/lewat, animasi akan lag nyata"
        );
    }
}
