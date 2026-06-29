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

fn star(rng: &mut SplitMix64, r: f64) -> Star {
    let r = r.max(0.005); // hindari ln(0) saat r→0 (bulge)
    let arm = rng.below(ARM_COUNT.max(1));
    let theta_arm =
        (1.0 / SPIRAL_B) * (r / R_CORE).ln() + arm as f64 * (TAU / ARM_COUNT.max(1) as f64);
    let theta0 = theta_arm + gauss(rng, SCATTER * (1.0 - r / R_MAX));
    let bright = 0.4 + 0.6 * rng.next_f64();
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
            let r = R_MAX * rng.next_f64().sqrt(); // sqrt → densitas seragam per luas
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
    pub fn render(&self, f: &mut Frame, area: Rect, t: f64, th: &Theme) {
        if area.width < 3 || area.height < 3 {
            return;
        }
        let (w, h) = (area.width as f64, area.height as f64);
        let scale = (w.min(h * ASPECT)) / (2.0 * R_MAX);
        let cx = area.x as f64 + w / 2.0;
        let cy = area.y as f64 + h / 2.0;
        let cells = (area.width as usize) * (area.height as usize);
        let mut dens = vec![0.0f64; cells]; // kecerahan terbaik per sel
        let mut temp = vec![Temp::Arm; cells];

        for s in &self.stars {
            let omega = OMEGA0 / (1.0 + s.r / OMEGA_R0);
            let theta = s.theta0 + omega * t;
            let col = (cx + s.r * theta.cos() * scale).round();
            let row = (cy + s.r * theta.sin() / ASPECT * scale).round();
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
            if b > dens[ix] {
                dens[ix] = b;
                temp[ix] = s.temp;
            }
        }

        let buf = f.buffer_mut();
        for j in 0..area.height {
            for i in 0..area.width {
                let ix = j as usize * area.width as usize + i as usize;
                let d = dens[ix];
                if d <= 0.0 {
                    continue;
                }
                let g = RAMP[((d * (RAMP.len() - 1) as f64) as usize).min(RAMP.len() - 1)];
                let color = match temp[ix] {
                    Temp::Core => th.focus,
                    Temp::Arm => th.advanced,
                    Temp::Edge => th.rare,
                };
                let cell = &mut buf[(area.x + i, area.y + j)];
                cell.set_char(g);
                cell.set_style(Style::default().fg(color));
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
            assert!((0.4..=1.0).contains(&s.bright));
        }
    }
}
