//! Warna bintang: aproksimasi blackbody + warna region HII. Port near-verbatim dari
//! `andromeda-simulation-tui`'s `src/galaxy/palette.rs`.

/// Aproksimasi warna blackbody (Tanner Helland) untuk suhu 1.000-40.000 K.
pub fn kelvin_to_rgb(kelvin: f32) -> [u8; 3] {
    let t = (kelvin / 100.0).clamp(10.0, 400.0);

    let r = if t <= 66.0 {
        255.0
    } else {
        329.698_73 * (t - 60.0).powf(-0.133_204_76)
    };

    let g = if t <= 66.0 {
        99.470_8 * t.ln() - 161.119_57
    } else {
        288.122_16 * (t - 60.0).powf(-0.075_514_846)
    };

    let b = if t >= 66.0 {
        255.0
    } else if t <= 19.0 {
        0.0
    } else {
        138.517_73 * (t - 10.0).ln() - 305.044_8
    };

    [
        r.clamp(0.0, 255.0) as u8,
        g.clamp(0.0, 255.0) as u8,
        b.clamp(0.0, 255.0) as u8,
    ]
}

/// Warna khas region HII (emisi Hα merah muda).
pub const HII_PINK: [u8; 3] = [255, 110, 150];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blackbody_extremes_look_right() {
        let cool = kelvin_to_rgb(3000.0); // oranye: R > G > B
        assert!(cool[0] > cool[1] && cool[1] > cool[2]);
        let hot = kelvin_to_rgb(15000.0); // biru-putih: B >= R
        assert!(hot[2] >= hot[0]);
        let mid = kelvin_to_rgb(6600.0); // ~putih
        assert!(mid.iter().all(|&c| c > 200));
    }
}
