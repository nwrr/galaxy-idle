//! Generator populasi bintang (density-wave) — diadaptasi dari `andromeda-simulation-tui`'s
//! `src/galaxy/{generate.rs,milky_way.rs,andromeda.rs}`, TAPI: (1) `Gen` di-reimplement di
//! atas `crate::rng::SplitMix64` (bukan `rand::StdRng`+`rand_distr`, konsisten kebijakan
//! zero-dep-RNG proyek ini) dgn Box-Muller normal sampler tulisan sendiri; (2) 2 resep bernama
//! (MilkyWay/Andromeda) digeneralisasi jd SATU fungsi `generate()` yg bercabang per
//! `GalaxyStyle` (barred/grand-design/flocculent) shg SETIAP galaxy frontier ProcGen (seed
//! beda) genuinely py tampilan beda, bukan cuma 2 pilihan tetap.
//!
//! Struktur populasi (bulge/disk-tua/muda/knot/debu/halo+globular/satelit) & rasio persentase
//! MENGIKUTI reference (SUDAH tertala visual), style HANYA mengubah: bar ada/tidak, & sebaran
//! populasi muda (barred: cincin+lengan lebar; grand-design: cincin rapi; flocculent: sebar
//! difus tanpa pola lengan).

use glam::Vec3;

use super::model::{AnchorId, GalaxyModel, OrbitParams, PoiOrbitKey, StarGpu};
use super::palette::{HII_PINK, kelvin_to_rgb};
use super::spec::{GalaxySpec, GalaxyStyle};
use crate::rng::SplitMix64;

/// RNG + akumulator bintang — pengganti reference's `Gen` (StdRng-based) di atas `SplitMix64`.
pub(super) struct Gen {
    rng: SplitMix64,
    pub(super) stars: Vec<StarGpu>,
}

impl Gen {
    pub(super) fn new(seed: u64, cap: usize) -> Self {
        Self {
            rng: SplitMix64::new(seed),
            stars: Vec::with_capacity(cap),
        }
    }

    /// Standar normal (mean=0, std=1) via Box-Muller -- pengganti `rand_distr::StandardNormal`.
    pub(super) fn n(&mut self) -> f32 {
        // u1 dijaga >0 (hindari ln(0) -> -inf) via `next_f64()` di [0,1) lalu clamp bawah kecil.
        let u1 = self.rng.next_f64().max(1e-12);
        let u2 = self.rng.next_f64();
        let r = (-2.0 * u1.ln()).sqrt();
        (r * (core::f64::consts::TAU * u2).cos()) as f32
    }

    pub(super) fn uniform(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (self.rng.next_f64() as f32) * (hi - lo)
    }

    pub(super) fn angle(&mut self) -> f32 {
        self.uniform(0.0, core::f32::consts::TAU)
    }

    /// `u32` di `[lo,hi]` inklusif -- pengganti `rng.random_range(lo..=hi)`.
    pub(super) fn range_u32(&mut self, lo: u32, hi: u32) -> u32 {
        lo + self.rng.below(hi - lo + 1)
    }

    pub(super) fn bool_with_prob(&mut self, p: f64) -> bool {
        self.rng.next_f64() < p
    }

    #[allow(clippy::too_many_arguments)] // satu parameter per field StarGpu, apa adanya.
    pub(super) fn push(
        &mut self,
        a: f32,
        q: f32,
        tilt: f32,
        theta0: f32,
        omega: f32,
        z: f32,
        rgb: [u8; 3],
        weight: u32,
        anchor: AnchorId,
        dust: bool,
    ) {
        let mut flags = anchor as u32;
        if dust {
            flags |= super::model::FLAG_DUST;
        }
        self.stars.push(StarGpu {
            a,
            b: a * q,
            tilt,
            theta0,
            omega,
            z,
            color: StarGpu::pack_color(rgb, weight),
            flags,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn push_disk_star(
        &mut self,
        a: f32,
        q: f32,
        tilt: f32,
        z_sigma: f32,
        kelvin: (f32, f32),
        w: (u32, u32),
        omega: f32,
    ) {
        let t = self.uniform(kelvin.0, kelvin.1);
        let weight = self.range_u32(w.0, w.1);
        let theta0 = self.angle();
        let z = self.n() * z_sigma;
        self.push(
            a,
            q,
            tilt,
            theta0,
            omega,
            z,
            kelvin_to_rgb(t),
            weight,
            AnchorId::Core,
            false,
        );
    }
}

/// Sampel eksponensial (mean=`scale`) via inverse-CDF, tolak-ulang sampai masuk `[lo,hi]`.
/// Pengganti reference's `sample_clipped(g, &Exp::new(1/scale), lo, hi)`.
fn sample_exp_clipped(g: &mut Gen, scale: f32, lo: f32, hi: f32) -> f32 {
    loop {
        let u = (g.uniform(0.0, 1.0)).min(0.999_999); // hindari ln(0).
        let r = -scale * (1.0 - u).ln();
        if (lo..=hi).contains(&r) {
            return r;
        }
    }
}

/// Sampel campuran 2 Normal (peluang `p` memilih yg pertama), tolak-ulang sampai masuk
/// `[lo,hi]`. Pengganti reference's `sample_mix`.
#[allow(clippy::too_many_arguments)]
fn sample_mix_clipped(
    g: &mut Gen,
    p: f64,
    mean1: f32,
    std1: f32,
    mean2: f32,
    std2: f32,
    lo: f32,
    hi: f32,
) -> f32 {
    loop {
        let r = if g.bool_with_prob(p) {
            mean1 + std1 * g.n()
        } else {
            mean2 + std2 * g.n()
        };
        if (lo..=hi).contains(&r) {
            return r;
        }
    }
}

/// Bangun model galaksi deterministik dari `(spec, seed, n_stars)`. Style menentukan bar
/// ada/tidak & pola sebaran populasi muda; sisanya (bulge/disk/knot/debu/halo/satelit)
/// dibagi via persentase SAMA lintas style (sudah tertala visual dr reference).
pub fn generate(spec: &GalaxySpec, seed: u64, n_stars: u32) -> GalaxyModel {
    let n = n_stars.max(200) as usize;
    let phys = &spec.phys;
    let twist = phys.twist_rad_per_kpc;
    let has_bar = spec.style == GalaxyStyle::BarredSpiral;
    let mut g = Gen::new(seed, n);
    let mut poi_orbits = Vec::new();

    let n_bar = if has_bar { n * 12 / 100 } else { 0 };
    let n_bulge = n * 10 / 100;
    let n_young = n * 16 / 100;
    let n_knots = n * 11 / 100;
    let n_dust = n * 8 / 100;
    let n_halo = n * 3 / 100;
    let n_sat1 = if spec.satellite_count >= 1 {
        n * 25 / 1000
    } else {
        0
    };
    let n_sat2 = if spec.satellite_count >= 2 {
        n * 12 / 1000
    } else {
        0
    };
    let n_disk = n
        .saturating_sub(n_bar)
        .saturating_sub(n_bulge)
        .saturating_sub(n_young)
        .saturating_sub(n_knots)
        .saturating_sub(n_dust)
        .saturating_sub(n_halo)
        .saturating_sub(n_sat1)
        .saturating_sub(n_sat2)
        .saturating_sub(2); // -2 utk 2 bintang "notable" (NotableStar1+2, Phase 7's POI).

    // ── 1. Bar (HANYA BarredSpiral): elips lonjong orientasi seragam = bar kaku ──
    const BAR_ANGLE: f32 = 0.45;
    if has_bar {
        for _ in 0..n_bar {
            let a = (g.n().abs() * 1.6 + 0.4).min(4.8);
            let q = g.uniform(0.35, 0.5);
            let tilt = BAR_ANGLE + g.n() * 0.06;
            let theta0 = g.angle();
            let z = g.n() * 0.25 * (0.4 + a / 5.0);
            let t = g.uniform(3800.0, 5200.0);
            let w = g.range_u32(2, 5);
            g.push(
                a,
                q,
                tilt,
                theta0,
                phys.omega(a),
                z,
                kelvin_to_rgb(t),
                w,
                AnchorId::Core,
                false,
            );
        }
    }

    // ── 2. Bulge klasik: sferoid kuning hangat, orientasi acak ──
    for _ in 0..n_bulge {
        let r = sample_exp_clipped(&mut g, 0.9, 0.03, 3.0);
        let q = g.uniform(0.9, 1.0);
        let tilt = g.angle();
        let theta0 = g.angle();
        let z = g.n() * 0.5 * r;
        let t = g.uniform(4200.0, 5400.0);
        let w = g.range_u32(2, 5);
        g.push(
            r,
            q,
            tilt,
            theta0,
            phys.omega(r),
            z,
            kelvin_to_rgb(t),
            w,
            AnchorId::Core,
            false,
        );
    }

    // ── 3. Disk tua: eksponensial R_D, spiral halus ──
    for _ in 0..n_disk {
        let a = sample_exp_clipped(&mut g, phys.r_d, phys.disk_r_min, phys.disk_r_max);
        let q = g.uniform(0.90, 0.97);
        let tilt = a * twist + g.n() * 0.25;
        g.push_disk_star(a, q, tilt, 0.3, (4500.0, 7000.0), (1, 3), phys.omega(a));
    }

    // ── Bintang "notable" tunggal (dicatat utk POI, Phase 7) ──
    let notable_theta0;
    let notable_a = phys.disk_r_min + (phys.disk_r_max - phys.disk_r_min) * 0.35;
    {
        let a = notable_a;
        let o = OrbitParams {
            a,
            b: a * 0.97,
            tilt: a * twist,
            theta0: g.angle(),
            omega: phys.omega(a),
            z: 0.02,
            anchor: AnchorId::Core,
        };
        notable_theta0 = o.theta0;
        g.push(
            o.a,
            0.97,
            o.tilt,
            o.theta0,
            o.omega,
            o.z,
            kelvin_to_rgb(5778.0),
            10,
            AnchorId::Core,
            false,
        );
        poi_orbits.push((PoiOrbitKey::NotableStar1, o));
    }

    // ── Bintang "notable" KEDUA: raksasa biru panas di radius BEDA -- variasi POI (Phase 7),
    // biar katalog py >1 bintang followable (bukan cuma 1 bintang+1 gugus).
    {
        let a = phys.disk_r_min + (phys.disk_r_max - phys.disk_r_min) * 0.62;
        let o = OrbitParams {
            a,
            b: a * 0.95,
            tilt: a * twist + 0.3,
            theta0: g.angle(),
            omega: phys.omega(a),
            z: 0.0,
            anchor: AnchorId::Core,
        };
        g.push(
            o.a,
            0.95,
            o.tilt,
            o.theta0,
            o.omega,
            o.z,
            kelvin_to_rgb(22_000.0),
            9,
            AnchorId::Core,
            false,
        );
        poi_orbits.push((PoiOrbitKey::NotableStar2, o));
    }

    // ── 4. Populasi muda biru: sebaran BEDA per style ──
    let mid = (phys.disk_r_min + phys.disk_r_max) / 2.0;
    for _ in 0..n_young {
        let a = match spec.style {
            // Barred: campuran cincin dalam + lengan lebar (pola SAMA reference MW).
            GalaxyStyle::BarredSpiral => sample_mix_clipped(
                &mut g,
                0.25,
                mid * 0.5,
                mid * 0.1,
                mid * 1.0,
                mid * 0.24,
                phys.disk_r_min,
                phys.disk_r_max,
            ),
            // Grand-design: DOMINAN 1 cincin rapi (pola SAMA reference Andromeda, p tinggi).
            GalaxyStyle::GrandDesign => sample_mix_clipped(
                &mut g,
                0.8,
                mid * 1.1,
                mid * 0.17,
                mid * 0.58,
                mid * 0.09,
                phys.disk_r_min,
                phys.disk_r_max,
            ),
            // Flocculent: SEBAR DIFUS, tanpa pola cincin/lengan jelas -- eksponensial polos.
            GalaxyStyle::Flocculent => {
                sample_exp_clipped(&mut g, phys.r_d * 1.3, phys.disk_r_min, phys.disk_r_max)
            }
        };
        let q = g.uniform(0.78, 0.88);
        let tilt = a * twist + g.n() * 0.10;
        g.push_disk_star(a, q, tilt, 0.12, (9000.0, 18000.0), (4, 8), phys.omega(a));
    }

    // ── 5. Knot: gugus muda + region HII, anggota kaku (ω induk bersama) ──
    {
        let mut remaining = n_knots;

        // Knot pertama: lbh kaya, dicatat utk POI ("notable cluster").
        let notable_members = (n_knots / 50).clamp(20, 400).min(remaining.max(1));
        let a_p = notable_a + 0.1;
        let q_p = 0.9;
        let o = OrbitParams {
            a: a_p,
            b: a_p * q_p,
            tilt: a_p * twist + g.n() * 0.04,
            theta0: notable_theta0 + 0.05,
            omega: phys.omega(a_p),
            z: 0.02,
            anchor: AnchorId::Core,
        };
        for _ in 0..notable_members {
            let a = o.a + g.n() * 0.12;
            let theta0 = o.theta0 + g.n() * 0.015;
            let z = o.z + g.n() * 0.04;
            let (rgb, w) = if g.bool_with_prob(0.3) {
                (
                    kelvin_to_rgb(g.uniform(15_000.0, 30_000.0)),
                    g.range_u32(5, 9),
                )
            } else {
                let j = g.range_u32(0, 19) as i16;
                let rgb = [
                    HII_PINK[0],
                    (HII_PINK[1] as i16 + j - 10).clamp(0, 255) as u8,
                    (HII_PINK[2] as i16 + j - 10).clamp(0, 255) as u8,
                ];
                (rgb, g.range_u32(4, 7))
            };
            g.push(
                a,
                q_p,
                o.tilt,
                theta0,
                o.omega,
                z,
                rgb,
                w,
                AnchorId::Core,
                false,
            );
        }
        poi_orbits.push((PoiOrbitKey::NotableCluster, o));
        remaining -= notable_members;

        while remaining > 0 {
            let a_p = g.uniform(phys.disk_r_min, phys.disk_r_max);
            let q_p = g.uniform(0.78, 0.88);
            let tilt_p = a_p * twist + g.n() * 0.08;
            let theta0_p = g.angle();
            let omega_p = phys.omega(a_p);
            let z_p = g.n() * 0.08;
            let is_blue = g.bool_with_prob(0.75);
            let size = (g.range_u32(50, 150) as usize).min(remaining);
            for _ in 0..size {
                let a = a_p + g.n() * 0.12;
                let theta0 = theta0_p + g.n() * 0.02;
                let z = z_p + g.n() * 0.04;
                let (rgb, w) = if is_blue {
                    (
                        kelvin_to_rgb(g.uniform(12_000.0, 20_000.0)),
                        g.range_u32(4, 10),
                    )
                } else {
                    let j = g.range_u32(0, 19) as i16;
                    let rgb = [
                        HII_PINK[0],
                        (HII_PINK[1] as i16 + j - 10).clamp(0, 255) as u8,
                        (HII_PINK[2] as i16 + j - 10).clamp(0, 255) as u8,
                    ];
                    (rgb, g.range_u32(3, 6))
                };
                g.push(
                    a,
                    q_p,
                    tilt_p,
                    theta0,
                    omega_p,
                    z,
                    rgb,
                    w,
                    AnchorId::Core,
                    false,
                );
            }
            remaining -= size;
        }
    }

    // ── 6. Debu: jalur gelap sepanjang lengan (offset -6 deg) ──
    for _ in 0..n_dust {
        let a = loop {
            let r = if g.bool_with_prob(0.6) {
                g.uniform(mid * 0.6, mid * 1.4)
            } else {
                sample_exp_clipped(&mut g, phys.r_d, 0.5, phys.disk_r_max)
            };
            if (1.0..=phys.disk_r_max.min(20.0)).contains(&r) {
                break r;
            }
        };
        let q = g.uniform(0.85, 0.93);
        let tilt = a * twist - 6.0_f32.to_radians() + g.n() * 0.08;
        let theta0 = g.angle();
        let z = g.n() * 0.1;
        let w = g.range_u32(4, 10);
        g.push(
            a,
            q,
            tilt,
            theta0,
            phys.omega(a),
            z,
            [0, 0, 0],
            w,
            AnchorId::Core,
            true,
        );
    }

    // ── 7. Halo + gugus bola ──
    {
        let mut remaining = n_halo;
        for _gi in 0..12 {
            if remaining == 0 {
                break;
            }
            let members = (g.range_u32(40, 80)).min(remaining as u32) as usize;
            let a_c = g.uniform(4.0, phys.disk_r_max.max(10.0) + 6.0);
            let z_c = g.n() * 0.3 * a_c;
            let theta0_c = g.angle();
            let tilt_c = g.angle();
            let omega_c = phys.omega(a_c);
            for _ in 0..members {
                let a = (a_c + g.n() * 0.05).max(0.1);
                let theta0 = theta0_c + g.n() * (0.06 / a_c);
                let z = z_c + g.n() * 0.05;
                let t = g.uniform(4000.0, 5200.0);
                let w = g.range_u32(2, 4);
                g.push(
                    a,
                    1.0,
                    tilt_c,
                    theta0,
                    omega_c,
                    z,
                    kelvin_to_rgb(t),
                    w,
                    AnchorId::Core,
                    false,
                );
            }
            remaining -= members;
        }
        // Bintang halo tersebar: hukum pangkat r^-3.5 (p(r) ∝ r^-1.5), sferis.
        let (s_min, s_max) = (4.0_f32.powf(-0.5), 26.0_f32.powf(-0.5));
        for _ in 0..remaining {
            let u = g.uniform(0.0, 1.0);
            let r = (s_min - u * (s_min - s_max)).powi(-2);
            let cos_phi = g.uniform(-1.0, 1.0);
            let z = r * cos_phi;
            let a = (r * (1.0 - cos_phi * cos_phi).sqrt()).max(0.3);
            let tilt = g.angle();
            let theta0 = g.angle();
            let t = g.uniform(4000.0, 5000.0);
            let w = g.range_u32(1, 2);
            g.push(
                a,
                1.0,
                tilt,
                theta0,
                phys.omega(a),
                z,
                kelvin_to_rgb(t),
                w,
                AnchorId::Core,
                false,
            );
        }
    }

    // ── 8. Satelit 1 (mis. LMC-like): katai irregular, anchor 1 ──
    if n_sat1 > 0 {
        let sat_tilt = g.angle();
        for _ in 0..n_sat1 {
            let a = g.n().abs() * 1.0 + 0.05;
            let theta0 = g.angle();
            let om = phys.omega(a) * 3.0;
            let z = g.n() * 0.3;
            let (rgb, w) = if g.bool_with_prob(0.32) {
                (
                    kelvin_to_rgb(g.uniform(9_000.0, 15_000.0)),
                    g.range_u32(3, 6),
                )
            } else {
                (
                    kelvin_to_rgb(g.uniform(4_500.0, 5_500.0)),
                    g.range_u32(1, 3),
                )
            };
            g.push(
                a,
                0.9,
                sat_tilt,
                theta0,
                om,
                z,
                rgb,
                w,
                AnchorId::Sat1,
                false,
            );
        }
    }

    // ── 9. Satelit 2 (mis. SMC-like): katai kecil memanjang, lebih redup, anchor 2 ──
    if n_sat2 > 0 {
        let sat_tilt = g.angle();
        for _ in 0..n_sat2 {
            let a = g.n().abs() * 0.6 + 0.02;
            let theta0 = g.angle();
            let om = phys.omega(a) * 3.0;
            let z = g.n() * 0.4;
            let (rgb, w) = if g.bool_with_prob(0.3) {
                (
                    kelvin_to_rgb(g.uniform(8_000.0, 14_000.0)),
                    g.range_u32(2, 4),
                )
            } else {
                (
                    kelvin_to_rgb(g.uniform(4_500.0, 5_600.0)),
                    g.range_u32(1, 2),
                )
            };
            g.push(
                a,
                0.6,
                sat_tilt,
                theta0,
                om,
                z,
                rgb,
                w,
                AnchorId::Sat2,
                false,
            );
        }
    }

    // Genapkan ke jumlah persis (pembulatan pembagian) dengan disk tua.
    while g.stars.len() < n {
        let a = sample_exp_clipped(&mut g, phys.r_d, phys.disk_r_min, phys.disk_r_max);
        let q = g.uniform(0.90, 0.97);
        let tilt = a * twist + g.n() * 0.25;
        g.push_disk_star(a, q, tilt, 0.3, (4500.0, 6500.0), (1, 3), phys.omega(a));
    }
    g.stars.truncate(n);

    GalaxyModel {
        stars: g.stars,
        anchors: spec.anchors,
        poi_orbits,
    }
}

/// Anchor world-space (kpc) dr [`GalaxySpec::anchors`], dipakai render/POI (Phase 5+).
pub fn anchors_of(spec: &GalaxySpec) -> [Vec3; 3] {
    spec.anchors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::galaxy_sim::spec::spec_for_seed;

    /// Generate DETERMINISTIK & jumlah bintang PERSIS -- pola SAMA reference's
    /// `deterministic_and_exact_count_all_galaxies`.
    #[test]
    fn deterministic_and_exact_count_all_styles() {
        for seed in [1u64, 2, 3, 4, 5, 100, 200] {
            let spec = spec_for_seed(seed, false);
            let m1 = generate(&spec, seed, 2_000);
            let m2 = generate(&spec, seed, 2_000);
            assert_eq!(m1.stars.len(), 2_000, "seed={seed}");
            assert_eq!(
                m1.stars.len(),
                m2.stars.len(),
                "jumlah bintang HARUS genuinely sama"
            );
            for (a, b) in m1.stars.iter().zip(&m2.stars) {
                assert_eq!(a.a, b.a);
                assert_eq!(a.b, b.b);
                assert_eq!(a.color, b.color);
                assert_eq!(a.flags, b.flags);
            }
        }
    }

    /// Anchor case (Milky Way) HARUS genuinely reproduksi jumlah bintang PERSIS jg.
    #[test]
    fn anchor_case_generates_exact_count() {
        let spec = spec_for_seed(0, true);
        let m = generate(&spec, 0, 5_000);
        assert_eq!(m.stars.len(), 5_000);
    }

    /// Weight tiap bintang HARUS genuinely dlm budget overflow (1..=MAX_WEIGHT) -- pola SAMA
    /// reference's `weights_within_overflow_budget`.
    #[test]
    fn weights_within_overflow_budget() {
        for seed in 0..10u64 {
            let spec = spec_for_seed(seed, false);
            let m = generate(&spec, seed, 3_000);
            assert!(
                m.stars
                    .iter()
                    .all(|s| (1..=super::super::model::MAX_WEIGHT).contains(&s.weight())),
                "seed={seed}"
            );
        }
    }

    /// Radius/z HARUS genuinely finite & positif/masuk akal -- pola SAMA reference's
    /// `radii_finite_and_positive`.
    #[test]
    fn radii_finite_and_positive() {
        for seed in 0..10u64 {
            let spec = spec_for_seed(seed, false);
            let m = generate(&spec, seed, 3_000);
            for s in &m.stars {
                assert!(
                    s.a.is_finite() && s.a > 0.0 && s.a < 60.0,
                    "seed={seed} a={}",
                    s.a
                );
                assert!(s.b.is_finite() && s.b > 0.0, "seed={seed} b={}", s.b);
                assert!(s.z.is_finite() && s.z.abs() < 40.0, "seed={seed} z={}", s.z);
            }
        }
    }

    /// POI orbit ("notable star"+"notable cluster") HARUS genuinely tercatat -- tanpa ini
    /// Phase 7 (POI catalog) tak py apa2 utk direferensikan.
    #[test]
    fn poi_orbits_recorded() {
        let spec = spec_for_seed(42, false);
        let m = generate(&spec, 42, 3_000);
        assert!(m.poi_orbit(PoiOrbitKey::NotableStar1).is_some());
        assert!(m.poi_orbit(PoiOrbitKey::NotableStar2).is_some());
        assert!(m.poi_orbit(PoiOrbitKey::NotableCluster).is_some());
    }

    /// `BarredSpiral` HARUS genuinely py bintang bar (elips lonjong q<0.55 di anchor inti);
    /// style LAIN TAK BOLEH (dites LANGSUNG, bukan diasumsikan dr cabang `if has_bar`).
    #[test]
    fn only_barred_spiral_has_bar_stars() {
        // Cari seed yg genuinely hasilkan tiap style (deterministik, style dr `spec_for_seed`).
        let mut found = [false; 3];
        for seed in 0..30u64 {
            let spec = spec_for_seed(seed, false);
            let idx = match spec.style {
                GalaxyStyle::BarredSpiral => 0,
                GalaxyStyle::GrandDesign => 1,
                GalaxyStyle::Flocculent => 2,
            };
            if found[idx] {
                continue;
            }
            found[idx] = true;
            let m = generate(&spec, seed, 5_000);
            let bar_like = m
                .stars
                .iter()
                .filter(|s| {
                    s.flags & super::super::model::FLAG_ANCHOR_MASK == 0
                        && s.a > 0.3
                        && s.b / s.a < 0.55
                })
                .count();
            if spec.style == GalaxyStyle::BarredSpiral {
                assert!(bar_like > 100, "seed={seed} bar_like={bar_like}");
            }
        }
        assert!(
            found.iter().all(|&f| f),
            "SEMUA 3 gaya HARUS genuinely ditemukan dr 30 seed"
        );
    }

    /// Satelit HARUS genuinely nol bintang saat `satellite_count=0` -- dites LANGSUNG dgn
    /// spec buatan tangan (bukan seed acak, biar pasti kena cabang 0).
    #[test]
    fn zero_satellites_produces_no_satellite_stars() {
        let mut spec = spec_for_seed(7, false);
        spec.satellite_count = 0;
        let m = generate(&spec, 7, 3_000);
        let sat_stars = m
            .stars
            .iter()
            .filter(|s| s.flags & super::super::model::FLAG_ANCHOR_MASK != 0)
            .count();
        assert_eq!(
            sat_stars, 0,
            "satellite_count=0 HARUS genuinely nol bintang satelit"
        );
    }
}
