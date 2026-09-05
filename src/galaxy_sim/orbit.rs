//! Kurva rotasi + posisi orbit density-wave — diadaptasi near-verbatim dari
//! `andromeda-simulation-tui`'s `src/galaxy/orbit.rs`. `eval` HARUS TETAP SINKRON dgn shader
//! WGSL (Phase 4) -- SATU-SATUNYA rumus posisi, jgn diubah salah satu tanpa yg lain.

use glam::{Vec2, Vec3};

use super::model::OrbitParams;

/// Kurva rotasi (kpc/Myr): naik ~solid-body di inti, mendatar (`tanh`) di luar.
/// `v_flat` dalam kpc/Myr, `r_core` radius transisi (kpc) — lihat `GalaxyPhys`.
pub fn rotation_velocity(r: f32, v_flat: f32, r_core: f32) -> f32 {
    v_flat * (r / r_core).tanh()
}

/// Kecepatan sudut orbit (rad/Myr) pada radius `r`.
pub fn omega(r: f32, v_flat: f32, r_core: f32) -> f32 {
    let r = r.max(0.05);
    rotation_velocity(r, v_flat, r_core) / r
}

/// Bungkus sudut ke [0, 2π) — menjaga presisi f32 utk t besar; rumus persis sama dgn versi
/// WGSL (x - floor(x/τ)·τ) demi paritas GPU/CPU (Phase 4).
#[inline]
pub fn wrap_angle(x: f32) -> f32 {
    use core::f32::consts::TAU;
    x - (x / TAU).floor() * TAU
}

/// Posisi orbit density-wave pada waktu t — SATU-SATUNYA rumus posisi.
///
/// θ = θ0 + ω·t; tilt_eff = tilt + pattern_ω·t (hanya anchor inti);
/// p = Rot2(tilt_eff)·(a·cosθ, b·sinθ); world = anchor + (p.x, p.y, z)
#[inline]
#[allow(clippy::too_many_arguments)] // parameter = field StarGpu apa adanya, mirror shader.
pub fn eval(
    a: f32,
    b: f32,
    tilt: f32,
    theta0: f32,
    omega: f32,
    z: f32,
    anchor_idx: usize,
    t_myr: f32,
    pattern_omega: f32,
    anchors: &[Vec3; 3],
) -> Vec3 {
    let theta = wrap_angle(theta0 + omega * t_myr);
    let tilt_eff = if anchor_idx == 0 {
        wrap_angle(tilt + pattern_omega * t_myr)
    } else {
        tilt
    };
    let (st, ct) = theta.sin_cos();
    let p = Vec2::new(a * ct, b * st);
    let (s, c) = tilt_eff.sin_cos();
    let xy = Vec2::new(p.x * c - p.y * s, p.x * s + p.y * c);
    anchors[anchor_idx] + Vec3::new(xy.x, xy.y, z)
}

pub fn orbit_position(
    o: &OrbitParams,
    t_myr: f32,
    pattern_omega: f32,
    anchors: &[Vec3; 3],
) -> Vec3 {
    eval(
        o.a,
        o.b,
        o.tilt,
        o.theta0,
        o.omega,
        o.z,
        o.anchor as usize,
        t_myr,
        pattern_omega,
        anchors,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Kurva rotasi HARUS genuinely naik lalu mendatar (bukan monoton naik tanpa batas) --
    /// pola SAMA reference's `rotation_curve_rises_then_flattens`.
    #[test]
    fn rotation_curve_rises_then_flattens() {
        let (vf, rc) = (220.0_f32, 2.0_f32);
        assert!(rotation_velocity(rc / 4.0, vf, rc) < 0.4 * vf);
        assert!((rotation_velocity(20.0, vf, rc) - vf).abs() < 0.01 * vf);
        let mut prev = 0.0;
        for i in 1..300 {
            let v = rotation_velocity(i as f32 * 0.1, vf, rc);
            assert!(v >= prev);
            prev = v;
        }
    }

    /// `omega` di r=0 HARUS genuinely aman (tak div-by-zero/NaN) — `r.max(0.05)` guard.
    #[test]
    fn omega_at_zero_radius_is_finite_not_nan() {
        let o = omega(0.0, 220.0, 2.0);
        assert!(o.is_finite() && o > 0.0);
    }

    /// `orbit_position` HARUS genuinely periodik (posisi sama tiap 1 periode penuh, dgn
    /// `pattern_omega=0` biar periodisitas EKSAK) & TETAP FINITE dgn presesi pola aktif --
    /// pola SAMA reference's `orbit_is_periodic_and_finite`.
    #[test]
    fn orbit_is_periodic_and_finite() {
        use crate::galaxy_sim::model::AnchorId;
        use core::f32::consts::TAU;

        let anchors = [
            Vec3::ZERO,
            Vec3::new(10.0, -12.0, -5.0),
            Vec3::new(13.5, -15.0, -7.0),
        ];
        for i in 0..40 {
            let a = 0.01 + i as f32 * 0.75; // 0.01 .. 30
            let o = OrbitParams {
                a,
                b: a * 0.9,
                tilt: a * 0.15,
                theta0: 1.3,
                omega: omega(a, 220.0 * crate::galaxy_sim::spec::KMS_TO_KPC_PER_MYR, 2.0),
                z: 0.2,
                anchor: AnchorId::Core,
            };
            let period = TAU / o.omega;
            let p0 = orbit_position(&o, 0.0, 0.0, &anchors);
            let p1 = orbit_position(&o, period, 0.0, &anchors);
            assert!(p0.is_finite() && p1.is_finite());
            assert!((p0 - p1).length() < 1e-3 * a.max(1.0), "a={a}");
            let p2 = orbit_position(&o, 12_345.0, 0.004, &anchors);
            assert!(p2.is_finite());
        }
    }

    /// Satelit (anchor != Core) HARUS genuinely TAK terpengaruh `pattern_omega` (tilt tetap,
    /// hanya anchor inti yg presesi) — dites LANGSUNG bandingkan `pattern_omega=0` vs besar.
    #[test]
    fn satellite_tilt_unaffected_by_pattern_precession() {
        use crate::galaxy_sim::model::AnchorId;

        let anchors = [
            Vec3::ZERO,
            Vec3::new(10.0, -12.0, -5.0),
            Vec3::new(13.5, -15.0, -7.0),
        ];
        let o = OrbitParams {
            a: 5.0,
            b: 4.0,
            tilt: 0.3,
            theta0: 0.0,
            omega: 0.01,
            z: 0.0,
            anchor: AnchorId::Sat1,
        };
        let p_no_precess = orbit_position(&o, 100.0, 0.0, &anchors);
        let p_with_precess = orbit_position(&o, 100.0, 0.5, &anchors);
        assert_eq!(
            p_no_precess, p_with_precess,
            "satelit HARUS genuinely tak terpengaruh pattern_omega (bukan cuma anchor inti)"
        );
    }
}
