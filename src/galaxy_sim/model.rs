//! Model bintang/uniform GPU-compatible — diadaptasi near-verbatim dari
//! `andromeda-simulation-tui`'s `src/galaxy/model.rs`. `StarGpu`/`Uniforms` HARUS `#[repr(C)]`+
//! `Pod` — layout identik dgn shader WGSL (Phase 4), jgn diubah tanpa sinkron kedua sisi.

use glam::Vec3;

/// Konversi km/s -> kpc/Myr.
pub const KMS_TO_KPC_PER_MYR: f32 = 0.001_022_71;
/// Batas atas weight per bintang; menjaga budget overflow akumulasi u32:
/// weight(16) * warna(255) * ~N bintang/piksel < 2^32.
pub const MAX_WEIGHT: u32 = 16;
/// Kalibrasi tonemap: akumulasi (ternormalisasi) yang dianggap "putih penuh".
pub const L_REF: f32 = 200.0;
/// Normalisasi fixed-point akumulasi: 1.0 = satu bintang weight-16 warna-255.
pub const FIXED_NORM: f32 = 1.0 / (255.0 * 16.0);
/// Koefisien ekstingsi debu relatif (memerah: biru paling teredam).
pub const DUST_BASE: [f32; 3] = [0.6, 1.0, 1.6];
/// Kekuatan ekstingsi debu pada zoom overview (skala thd `N_REF` bintang).
pub const DUST_STRENGTH: f32 = 0.065;
/// Jumlah bintang acuan untuk normalisasi exposure/debu (viewport TUI kecil -> jauh lbh
/// sedikit drpd reference project's 400k; ditala Phase 5 via screenshot, bukan ditebak).
pub const N_REF: f32 = 400_000.0;

pub const FLAG_ANCHOR_MASK: u32 = 0b11;
pub const FLAG_DUST: u32 = 1 << 2;

/// Satu bintang/partikel. 32 byte, layout identik dgn `Star` di shader WGSL (Phase 4).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StarGpu {
    /// Sumbu semi-mayor elips (kpc). Untuk anggota satelit: radius lokal.
    pub a: f32,
    /// Sumbu semi-minor elips (kpc).
    pub b: f32,
    /// Orientasi elips φ(a) (rad) — puntiran radial membentuk spiral.
    pub tilt: f32,
    /// Fase awal orbit (rad).
    pub theta0: f32,
    /// Kecepatan sudut (rad/Myr), dari kurva rotasi.
    pub omega: f32,
    /// Offset vertikal tetap dari bidang anchor (kpc).
    pub z: f32,
    /// Packed: R | G<<8 | B<<16 | weight<<24 (weight 1..=MAX_WEIGHT).
    pub color: u32,
    /// bit 0..1: indeks anchor (0=inti, 1=satelit-1, 2=satelit-2); bit 2: debu.
    pub flags: u32,
}

impl StarGpu {
    pub fn pack_color(rgb: [u8; 3], weight: u32) -> u32 {
        let w = weight.clamp(1, MAX_WEIGHT);
        (rgb[0] as u32) | ((rgb[1] as u32) << 8) | ((rgb[2] as u32) << 16) | (w << 24)
    }

    pub fn weight(&self) -> u32 {
        (self.color >> 24) & 0xff
    }
}

/// Indeks anchor per bintang; arti satelit bergantung galaksi.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorId {
    Core = 0,
    Sat1 = 1,
    Sat2 = 2,
}

/// Parameter orbit density-wave; posisi = fungsi murni waktu (lihat `orbit::eval`).
#[derive(Clone, Copy, Debug)]
pub struct OrbitParams {
    pub a: f32,
    pub b: f32,
    pub tilt: f32,
    pub theta0: f32,
    pub omega: f32,
    pub z: f32,
    pub anchor: AnchorId,
}

/// Objek yang parameter orbitnya dicatat saat generate agar POI (Phase 7) bisa mengikutinya.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PoiOrbitKey {
    NotableStar1,
    NotableStar2,
    NotableCluster,
}

pub struct GalaxyModel {
    pub stars: Vec<StarGpu>,
    /// Posisi pusat: [inti, satelit-1, satelit-2] (kpc, statis).
    pub anchors: [Vec3; 3],
    pub poi_orbits: Vec<(PoiOrbitKey, OrbitParams)>,
}

impl GalaxyModel {
    pub fn poi_orbit(&self, key: PoiOrbitKey) -> Option<OrbitParams> {
        self.poi_orbits
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, o)| *o)
    }
}

/// Uniform per frame — layout HARUS identik dgn struct `Uniforms` di shader WGSL (Phase 4).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub time_myr: f32,
    pub pattern_omega: f32,
    pub zoom: f32,
    /// Exposure FINAL (sudah dikompensasi zoom & jumlah bintang).
    pub exposure: f32,

    pub view_center: [f32; 2],
    pub screen: [f32; 2],

    pub incl_cs: [f32; 2],
    pub pa_cs: [f32; 2],

    /// xyz: koefisien ekstingsi debu FINAL; w: cell_aspect (dipakai proyeksi).
    pub dust_k: [f32; 4],

    pub anchors: [[f32; 4]; 3],

    pub star_count: u32,
    /// Penyebut tonemap: asinh(exposure * L_REF), dihitung di CPU.
    pub tonemap_denom: f32,
    pub _pad: [u32; 2],
}

const _: () = assert!(size_of::<Uniforms>() == 128);
const _: () = assert!(size_of::<StarGpu>() == 32);

#[cfg(test)]
mod tests {
    use super::*;

    /// `pack_color`/`weight` HARUS genuinely round-trip (bukan cuma satu arah) -- warna+weight
    /// dites lintas beberapa kombinasi, bukan satu nilai tunggal.
    #[test]
    fn pack_color_round_trips_weight() {
        for w in [1u32, 8, 16, 99] {
            // 99 clamp ke MAX_WEIGHT.
            let color = StarGpu::pack_color([10, 20, 30], w);
            let s = StarGpu {
                a: 0.0,
                b: 0.0,
                tilt: 0.0,
                theta0: 0.0,
                omega: 0.0,
                z: 0.0,
                color,
                flags: 0,
            };
            assert_eq!(s.weight(), w.clamp(1, MAX_WEIGHT));
        }
    }

    /// `StarGpu`/`Uniforms` HARUS genuinely ukuran byte TETAP (32/128) -- dijamin `const
    /// assert` di atas SUDAH gagal compile kalau salah, test ini cuma dokumentasi eksplisit.
    #[test]
    fn struct_sizes_match_shader_layout() {
        assert_eq!(size_of::<StarGpu>(), 32);
        assert_eq!(size_of::<Uniforms>(), 128);
    }
}
