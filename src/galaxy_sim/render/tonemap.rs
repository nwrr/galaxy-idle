//! Tonemap asinh hue-preserving + ekstingsi debu — port near-verbatim dari
//! `andromeda-simulation-tui`'s `src/render/tonemap.rs`. KEEP IN SYNC dgn `render/gpu.rs`
//! (Phase 4, WGSL compute mirror) bila salah satu diubah.

use crate::galaxy_sim::model::FIXED_NORM;

/// Tonemap asinh hue-preserving + ekstingsi debu, dr akumulasi fixed-point.
#[inline]
pub fn tonemap_pixel(
    r: u32,
    g: u32,
    b: u32,
    d: u32,
    exposure: f32,
    dust_k: [f32; 4],
    denom: f32,
) -> [u8; 4] {
    let dn = d as f32 / 16.0;
    let c = [
        r as f32 * FIXED_NORM * (-dust_k[0] * dn).exp(),
        g as f32 * FIXED_NORM * (-dust_k[1] * dn).exp(),
        b as f32 * FIXED_NORM * (-dust_k[2] * dn).exp(),
    ];
    let l = c[0].max(c[1]).max(c[2]);
    if l <= 0.0 {
        return [0, 0, 0, 255];
    }
    let lt = ((exposure * l).asinh() / denom).clamp(0.0, 1.0);
    let k = lt / l;
    [
        ((c[0] * k).clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        ((c[1] * k).clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        ((c[2] * k).clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        255,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::galaxy_sim::model::L_REF;

    const DK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

    fn denom(exposure: f32) -> f32 {
        (exposure * L_REF).asinh()
    }

    /// Input nol HARUS genuinely hitam (bukan NaN, `l<=0.0` early-return guard).
    #[test]
    fn zero_input_is_black_not_nan() {
        assert_eq!(
            tonemap_pixel(0, 0, 0, 0, 1.0, DK, denom(1.0)),
            [0, 0, 0, 255]
        );
    }

    /// Akumulasi lbh besar HARUS genuinely hasilkan channel TAK LBH GELAP (monoton) & TETAP
    /// dlm batas [0,255] -- dites lintas rentang skala besar (bukan 2 titik saja).
    #[test]
    fn monotone_and_bounded() {
        let mut prev = 0u8;
        for raw in [1u32, 10, 100, 1_000, 10_000, 100_000, 10_000_000] {
            let px = tonemap_pixel(raw, raw, raw, 0, 1.0, DK, denom(1.0));
            assert!(px[0] >= prev, "raw={raw} px[0]={} prev={prev}", px[0]);
            prev = px[0];
        }
    }

    /// Ekstingsi debu HARUS genuinely REDAM warna (channel lbh kecil) dibanding tanpa debu --
    /// dites LANGSUNG bandingkan d=0 vs d besar, bukan diasumsikan dr formula.
    #[test]
    fn dust_extinction_dims_color() {
        let dk = [0.5, 0.5, 0.5, 1.0];
        let no_dust = tonemap_pixel(50_000, 50_000, 50_000, 0, 1.0, dk, denom(1.0));
        let with_dust = tonemap_pixel(50_000, 50_000, 50_000, 200, 1.0, dk, denom(1.0));
        assert!(with_dust[0] < no_dust[0]);
    }
}
