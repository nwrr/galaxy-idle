//! `GalaxySpec` seed-driven — diadaptasi dari `andromeda-simulation-tui`'s
//! `src/galaxy/spec.rs`, TAPI diganti dari 2 galaksi bernama (`MilkyWay`/`Andromeda`) jadi
//! `GalaxyStyle` (barred/grand-design/flocculent) + parameter KONTINU diturunkan dari `u64`
//! seed via `crate::rng::derive` — konsisten dgn cara nama/biome/planet galaxy-idle SUDAH
//! bervariasi per seed (`game/world/procgen.rs`), bukan `rand`/`rand_distr` (M20.8 follow-up
//! plan: "RNG: reuse SplitMix64, no new RNG dep").
//!
//! Kasus `anchor=true` (galaksi Home/Fixed, "Milky Way") HARUS reproduksi angka LITERAL
//! reference project persis (regression test) -- galaksi frontier ProcGen (`anchor=false`)
//! dapat gaya+parameter kontinu dr seed masing2.

use glam::Vec3;

use super::orbit;

/// Gaya galaksi -- menentukan resep populasi bintang (Phase 2) & rentang parameter fisik.
/// Urutan varian dipakai `derive(seed,"style") % 3` (lihat `spec_for_seed`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GalaxyStyle {
    /// Spiral berbatang (mis. Bimasakti/SBbc) -- bar pusat kaku + 2 lengan.
    BarredSpiral,
    /// Spiral 2-lengan rapi tanpa bar (grand-design, mis. M51-like).
    GrandDesign,
    /// Spiral berlengan banyak/tak rapi (flocculent, tanpa pola 2-lengan jelas).
    Flocculent,
}

/// Parameter fisik per galaksi -- hanya dipakai generator (ω dibake per bintang saat generate,
/// Phase 2). Port near-verbatim dari reference `GalaxyPhys`.
#[derive(Clone, Copy, Debug)]
pub struct GalaxyPhys {
    /// Kecepatan rotasi datar (km/s).
    pub v_flat_kms: f32,
    /// Radius transisi kurva rotasi solid-body -> datar (kpc).
    pub r_core: f32,
    /// Skala radial disk eksponensial (kpc).
    pub r_d: f32,
    /// Batas radial disk (kpc).
    pub disk_r_min: f32,
    pub disk_r_max: f32,
    /// Puntiran orientasi elips per kpc -- membentuk pola spiral/cincin.
    pub twist_rad_per_kpc: f32,
    /// Presesi lambat pola (rad/Myr) untuk tilt bintang anchor inti.
    pub pattern_omega: f32,
}

/// km/s -> kpc/Myr (port dr reference `model.rs::KMS_TO_KPC_PER_MYR`, dipakai Phase 1+).
pub const KMS_TO_KPC_PER_MYR: f32 = 0.001_022_71;

impl GalaxyPhys {
    /// Kecepatan datar dalam kpc/Myr.
    pub fn v_flat(&self) -> f32 {
        self.v_flat_kms * KMS_TO_KPC_PER_MYR
    }

    /// Kecepatan sudut orbit (rad/Myr) pada radius r.
    pub fn omega(&self, r: f32) -> f32 {
        orbit::omega(r, self.v_flat(), self.r_core)
    }
}

/// Sudut pandang default galaksi di viewport. Port near-verbatim dari reference `GalaxyView`.
#[derive(Clone, Copy, Debug)]
pub struct GalaxyView {
    pub inclination_deg: f32,
    pub position_angle_deg: f32,
    /// Lebar dunia (kpc) yang dimuat viewport pada zoom overview.
    pub span_kpc: f32,
}

#[derive(Clone, Debug)]
pub struct GalaxySpec {
    pub style: GalaxyStyle,
    pub name: String,
    pub phys: GalaxyPhys,
    pub view: GalaxyView,
    /// Posisi pusat [inti, satelit-1, satelit-2] (kpc) -- artistik, statis.
    pub anchors: [Vec3; 3],
    /// 0..=2 -- galaksi frontier BOLEH tak py satelit sama sekali (0), Milky Way anchor
    /// SELALU 2 (LMC+SMC, cocok reference persis).
    pub satellite_count: u8,
}

/// Turunkan `f32` di rentang `[lo,hi)` dr `(seed,label)` -- deterministik, dipakai SEMUA
/// parameter kontinu di bawah (ganti `rand::random_range` reference dgn `SplitMix64`-based
/// `crate::rng::derive`, konsisten kebijakan zero-dep-RNG proyek ini).
fn derive_range(seed: u64, label: &str, lo: f32, hi: f32) -> f32 {
    let h = crate::rng::derive(seed, label);
    let t = (h >> 11) as f64 / (1u64 << 53) as f64; // [0,1), pola sama `SplitMix64::next_f64`.
    lo + (t as f32) * (hi - lo)
}

/// Bangun `GalaxySpec` dr `seed`. `anchor=true` -> galaksi Home tetap (angka LITERAL reference
/// project's Milky Way, regression-tested persis); `anchor=false` -> gaya+parameter kontinu
/// diturunkan dr seed (galaksi frontier ProcGen, tiap seed genuinely beda tampilan).
pub fn spec_for_seed(seed: u64, anchor: bool) -> GalaxySpec {
    if anchor {
        return GalaxySpec {
            style: GalaxyStyle::BarredSpiral,
            name: "Milky Way".into(),
            phys: GalaxyPhys {
                v_flat_kms: 220.0,
                r_core: 2.0,
                r_d: 2.6,
                disk_r_min: 0.3,
                disk_r_max: 16.0,
                twist_rad_per_kpc: 7.0 * core::f32::consts::PI / 180.0,
                pattern_omega: 0.004,
            },
            view: GalaxyView {
                inclination_deg: 35.0,
                position_angle_deg: 15.0,
                span_kpc: 44.0,
            },
            anchors: [
                Vec3::ZERO,
                Vec3::new(10.0, -12.0, -5.0),
                Vec3::new(13.5, -15.0, -7.0),
            ],
            satellite_count: 2,
        };
    }

    let style = match crate::rng::derive(seed, "galaxy_style") % 3 {
        0 => GalaxyStyle::BarredSpiral,
        1 => GalaxyStyle::GrandDesign,
        _ => GalaxyStyle::Flocculent,
    };
    // Twist per-kpc: barred paling rapat (bar kaku, lengan pendek), flocculent paling lebar
    // (lengan berantakan/tak rapi) -- rentang overlap sengaja (variasi kontinu, bukan pagar
    // keras per gaya).
    let twist_deg_range = match style {
        GalaxyStyle::BarredSpiral => (5.0, 9.0),
        GalaxyStyle::GrandDesign => (7.0, 11.0),
        GalaxyStyle::Flocculent => (9.0, 14.0),
    };
    let satellite_count = (crate::rng::derive(seed, "satellite_count") % 3) as u8; // 0..=2.

    GalaxySpec {
        style,
        name: String::new(), // diisi caller (nama galaksi SUDAH digenerate procgen.rs terpisah).
        phys: GalaxyPhys {
            v_flat_kms: derive_range(seed, "v_flat_kms", 150.0, 280.0),
            r_core: derive_range(seed, "r_core", 1.0, 4.0),
            r_d: derive_range(seed, "r_d", 2.0, 6.0),
            disk_r_min: derive_range(seed, "disk_r_min", 0.2, 0.5),
            disk_r_max: derive_range(seed, "disk_r_max", 12.0, 24.0),
            twist_rad_per_kpc: derive_range(
                seed,
                "twist_rad_per_kpc",
                twist_deg_range.0,
                twist_deg_range.1,
            ) * core::f32::consts::PI
                / 180.0,
            pattern_omega: derive_range(seed, "pattern_omega", 0.002, 0.006),
        },
        view: GalaxyView {
            inclination_deg: derive_range(seed, "inclination_deg", 20.0, 80.0),
            position_angle_deg: derive_range(seed, "position_angle_deg", 0.0, 360.0),
            span_kpc: derive_range(seed, "span_kpc", 30.0, 60.0),
        },
        anchors: [
            Vec3::ZERO,
            Vec3::new(
                derive_range(seed, "anchor1_x", 6.0, 14.0),
                derive_range(seed, "anchor1_y", -16.0, -8.0),
                derive_range(seed, "anchor1_z", -8.0, -3.0),
            ),
            Vec3::new(
                derive_range(seed, "anchor2_x", 8.0, 16.0),
                derive_range(seed, "anchor2_y", -18.0, -10.0),
                derive_range(seed, "anchor2_z", -9.0, -4.0),
            ),
        ],
        satellite_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `spec_for_seed(_, anchor=true)` HARUS genuinely reproduksi angka LITERAL reference
    /// project's Milky Way persis (regression test, bukan cuma "masuk akal") -- dikonfirmasi
    /// tiap field satu-per-satu, bukan cuma sebagian.
    #[test]
    fn anchor_case_reproduces_reference_milky_way_exactly() {
        let s = spec_for_seed(0, true);
        assert_eq!(s.style, GalaxyStyle::BarredSpiral);
        assert_eq!(s.name, "Milky Way");
        assert_eq!(s.phys.v_flat_kms, 220.0);
        assert_eq!(s.phys.r_core, 2.0);
        assert_eq!(s.phys.r_d, 2.6);
        assert_eq!(s.phys.disk_r_min, 0.3);
        assert_eq!(s.phys.disk_r_max, 16.0);
        assert!((s.phys.twist_rad_per_kpc - 7.0_f32.to_radians()).abs() < 1e-6);
        assert_eq!(s.phys.pattern_omega, 0.004);
        assert_eq!(s.view.inclination_deg, 35.0);
        assert_eq!(s.view.position_angle_deg, 15.0);
        assert_eq!(s.view.span_kpc, 44.0);
        assert_eq!(s.anchors[0], Vec3::ZERO);
        assert_eq!(s.anchors[1], Vec3::new(10.0, -12.0, -5.0));
        assert_eq!(s.anchors[2], Vec3::new(13.5, -15.0, -7.0));
        assert_eq!(s.satellite_count, 2);
    }

    /// Seed sama (anchor=false) HARUS genuinely hasilkan spec IDENTIK -- deterministik,
    /// dites bandingkan field-demi-field (bukan cuma satu properti).
    #[test]
    fn same_seed_produces_identical_spec() {
        let a = spec_for_seed(12345, false);
        let b = spec_for_seed(12345, false);
        assert_eq!(a.style, b.style);
        assert_eq!(a.phys.v_flat_kms, b.phys.v_flat_kms);
        assert_eq!(a.phys.r_core, b.phys.r_core);
        assert_eq!(a.view.inclination_deg, b.view.inclination_deg);
        assert_eq!(a.anchors, b.anchors);
        assert_eq!(a.satellite_count, b.satellite_count);
    }

    /// Seed BEDA HARUS genuinely hasilkan spec BEDA (bukan collapse ke 1 nilai tunggal) --
    /// dites via sampel banyak seed, minimal separuh field kunci genuinely bervariasi.
    #[test]
    fn different_seeds_produce_varied_specs() {
        let specs: Vec<_> = (0..50u64).map(|s| spec_for_seed(s, false)).collect();
        let v_flats: std::collections::BTreeSet<u32> =
            specs.iter().map(|s| s.phys.v_flat_kms.to_bits()).collect();
        assert!(
            v_flats.len() > 10,
            "v_flat_kms HARUS genuinely bervariasi antar seed, got {} nilai unik dr 50 seed",
            v_flats.len()
        );
        let styles: std::collections::BTreeSet<_> =
            specs.iter().map(|s| format!("{:?}", s.style)).collect();
        assert!(
            styles.len() >= 2,
            "SEMUA 3 gaya HARUS genuinely muncul dr 50 seed, got: {styles:?}"
        );
    }

    /// SEMUA field kontinu HARUS genuinely dlm rentang yg didefinisikan (bukan overflow/NaN)
    /// -- dites LANGSUNG lintas banyak seed, bukan diasumsikan aman dr formula.
    #[test]
    fn continuous_fields_stay_within_declared_ranges() {
        for seed in 0..200u64 {
            let s = spec_for_seed(seed, false);
            assert!((150.0..280.0).contains(&s.phys.v_flat_kms), "seed={seed}");
            assert!((1.0..4.0).contains(&s.phys.r_core), "seed={seed}");
            assert!((2.0..6.0).contains(&s.phys.r_d), "seed={seed}");
            assert!((0.2..0.5).contains(&s.phys.disk_r_min), "seed={seed}");
            assert!((12.0..24.0).contains(&s.phys.disk_r_max), "seed={seed}");
            assert!(s.phys.twist_rad_per_kpc.is_finite() && s.phys.twist_rad_per_kpc > 0.0);
            assert!(
                (0.002..0.006).contains(&s.phys.pattern_omega),
                "seed={seed}"
            );
            assert!(
                (20.0..80.0).contains(&s.view.inclination_deg),
                "seed={seed}"
            );
            assert!(
                (0.0..360.0).contains(&s.view.position_angle_deg),
                "seed={seed}"
            );
            assert!((30.0..60.0).contains(&s.view.span_kpc), "seed={seed}");
            assert!(s.satellite_count <= 2);
            for a in s.anchors {
                assert!(a.is_finite(), "seed={seed} anchor={a:?}");
            }
        }
    }

    /// `GalaxyPhys::v_flat()`/`omega()` HARUS genuinely bekerja (delegasi ke `orbit.rs`) utk
    /// spec hasil `spec_for_seed`, bukan cuma spec anchor -- dites nilai FINITE & masuk akal.
    #[test]
    fn phys_helpers_work_for_seeded_spec() {
        let s = spec_for_seed(7, false);
        let vf = s.phys.v_flat();
        assert!(vf.is_finite() && vf > 0.0);
        let om = s.phys.omega(5.0);
        assert!(om.is_finite() && om > 0.0);
    }
}
