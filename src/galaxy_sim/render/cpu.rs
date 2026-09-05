//! Backend CPU (rayon) — port near-verbatim dari `andromeda-simulation-tui`'s
//! `src/render/cpu.rs`. Matematika fixed-point identik dgn shader WGSL (Phase 4): akumulasi
//! per-thread lalu direduksi, tonemap paralel per piksel.

use glam::Vec3;
use rayon::prelude::*;

use super::SimBackend;
use super::tonemap::tonemap_pixel;
use crate::galaxy_sim::model::{FLAG_ANCHOR_MASK, FLAG_DUST, StarGpu, Uniforms};
use crate::galaxy_sim::orbit;

pub struct CpuBackend {
    stars: Vec<StarGpu>,
    out: Vec<u8>,
    w: u32,
    h: u32,
}

impl CpuBackend {
    pub fn new(stars: Vec<StarGpu>) -> Self {
        Self {
            stars,
            out: Vec::new(),
            w: 0,
            h: 0,
        }
    }
}

/// Mirror `accumulate` di shader WGSL (Phase 4).
#[inline]
fn accumulate_star(s: &StarGpu, u: &Uniforms, anchors: &[Vec3; 3], w: usize, buf: &mut [u32]) {
    let anchor = (s.flags & FLAG_ANCHOR_MASK) as usize;
    let world = orbit::eval(
        s.a,
        s.b,
        s.tilt,
        s.theta0,
        s.omega,
        s.z,
        anchor,
        u.time_myr,
        u.pattern_omega,
        anchors,
    );
    let vy = world.y * u.incl_cs[0] - world.z * u.incl_cs[1];
    let qx = world.x * u.pa_cs[0] - vy * u.pa_cs[1] - u.view_center[0];
    let qy = world.x * u.pa_cs[1] + vy * u.pa_cs[0] - u.view_center[1];
    let px = qx * u.zoom + u.screen[0] * 0.5;
    let py = qy * u.zoom * u.dust_k[3] + u.screen[1] * 0.5;
    if px < 0.0 || py < 0.0 || px >= u.screen[0] || py >= u.screen[1] {
        return;
    }
    let idx = (py as usize * w + px as usize) * 4;
    let weight = s.weight();
    if s.flags & FLAG_DUST != 0 {
        buf[idx + 3] += weight;
    } else {
        buf[idx] += (s.color & 0xff) * weight;
        buf[idx + 1] += ((s.color >> 8) & 0xff) * weight;
        buf[idx + 2] += ((s.color >> 16) & 0xff) * weight;
    }
}

impl SimBackend for CpuBackend {
    fn name(&self) -> String {
        format!("CPU: rayon {} threads", rayon::current_num_threads())
    }

    fn resize(&mut self, w: u32, h: u32) {
        self.w = w;
        self.h = h;
        self.out = vec![0; (w * h * 4) as usize];
    }

    fn set_stars(&mut self, stars: &[StarGpu]) {
        self.stars = stars.to_vec();
    }

    fn render(&mut self, u: &Uniforms) -> Result<&[u8], String> {
        let (w, h) = (self.w as usize, self.h as usize);
        if w == 0 || h == 0 {
            return Err("render dipanggil sblm resize (w/h=0)".into());
        }
        let len = w * h * 4;
        let anchors = [
            Vec3::from_slice(&u.anchors[0][..3]),
            Vec3::from_slice(&u.anchors[1][..3]),
            Vec3::from_slice(&u.anchors[2][..3]),
        ];

        let accum = self
            .stars
            .par_chunks(8192)
            .fold(
                || vec![0u32; len],
                |mut buf, chunk| {
                    for s in chunk {
                        accumulate_star(s, u, &anchors, w, &mut buf);
                    }
                    buf
                },
            )
            .reduce(
                || vec![0u32; len],
                |mut a, b| {
                    a.iter_mut().zip(&b).for_each(|(x, y)| *x += *y);
                    a
                },
            );

        self.out
            .par_chunks_exact_mut(4)
            .enumerate()
            .for_each(|(i, px)| {
                let o = tonemap_pixel(
                    accum[i * 4],
                    accum[i * 4 + 1],
                    accum[i * 4 + 2],
                    accum[i * 4 + 3],
                    u.exposure,
                    u.dust_k,
                    u.tonemap_denom,
                );
                px.copy_from_slice(&o);
            });

        Ok(&self.out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::galaxy_sim::generate::generate;
    use crate::galaxy_sim::spec::spec_for_seed;

    fn test_uniforms(screen: [f32; 2], star_count: u32) -> Uniforms {
        Uniforms {
            time_myr: 0.0,
            pattern_omega: 0.004,
            zoom: 4.0,
            exposure: 1.0,
            view_center: [0.0, 0.0],
            screen,
            incl_cs: [1.0, 0.0],
            pa_cs: [1.0, 0.0],
            dust_k: [0.1, 0.1, 0.1, 1.0],
            anchors: [[0.0; 4]; 3],
            star_count,
            tonemap_denom: 1.0,
            _pad: [0; 2],
        }
    }

    /// `render` sblm `resize` HARUS genuinely Err (bukan panic/index-out-of-bounds) --
    /// dites LANGSUNG, bukan diasumsikan dr guard.
    #[test]
    fn render_before_resize_is_err_not_panic() {
        let mut b = CpuBackend::new(Vec::new());
        let u = test_uniforms([32.0, 16.0], 0);
        assert!(b.render(&u).is_err());
    }

    /// Render HARUS genuinely hasilkan buffer ukuran w*h*4 & TAK all-black (bintang genuinely
    /// muncul) -- pola SAMA reference's `set_stars_switches_galaxy` tp diperluas cek non-black.
    #[test]
    fn render_produces_correctly_sized_non_black_buffer() {
        let spec = spec_for_seed(0, true); // anchor Milky Way, span_kpc=44.
        let model = generate(&spec, 31, 4_000);
        let mut b = CpuBackend::new(model.stars);
        b.resize(64, 32);
        let mut u = test_uniforms([64.0, 32.0], 4_000);
        u.zoom = 64.0 / spec.view.span_kpc;
        u.anchors = model.anchors.map(|a| [a.x, a.y, a.z, 0.0]);
        let out = b.render(&u).unwrap();
        assert_eq!(out.len(), 64 * 32 * 4);
        assert!(
            out.iter().any(|&c| c > 0),
            "buffer HARUS genuinely py piksel non-hitam (bintang muncul)"
        );
    }

    /// Ganti galaksi (`set_stars`) HARUS genuinely ubah hasil render -- pola SAMA reference's
    /// `set_stars_switches_galaxy`.
    #[test]
    fn set_stars_switches_galaxy() {
        let spec_a = spec_for_seed(1, false);
        let spec_b = spec_for_seed(2, false);
        let stars_a = generate(&spec_a, 1, 2_000).stars;
        let stars_b = generate(&spec_b, 2, 2_000).stars;

        let mut b = CpuBackend::new(stars_a);
        b.resize(32, 16);
        let mut u = test_uniforms([32.0, 16.0], 2_000);
        u.zoom = 1.0;
        let f1 = b.render(&u).unwrap().to_vec();
        assert_eq!(f1.len(), 32 * 16 * 4);

        b.set_stars(&stars_b);
        let f2 = b.render(&u).unwrap().to_vec();
        assert_eq!(f2.len(), 32 * 16 * 4);
        assert_ne!(f1, f2, "galaksi beda HARUS genuinely hasilkan gambar beda");
    }
}
