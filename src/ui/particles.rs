//! Sistem partikel kosmetik (`15-particle-effects.md`): exhaust, warp trail, nebula, sparkle.
//!
//! **Read-only terhadap `GameState`** — di-`update(dt)` di loop render (waktu nyata), lalu
//! `render` ke buffer. Snapshot deterministik: demo tak emit (ship Idle) → golden tak berubah.
//!
//! Asumsi (deviasi aman dari spec): koordinat partikel **dinormalisasi** `[0,1]` (bukan sel
//! mentah) agar `update` lepas dari ukuran panel; dipetakan ke `area` saat render (koreksi ASPECT).
#![allow(dead_code)]

use crate::balance::{ASPECT, PARTICLE_BUDGET};
use crate::rng::SplitMix64;
use crate::ui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use std::f32::consts::TAU;

/// Glyph densitas rendah→tinggi (banyak partikel per sel → makin padat).
const RAMP_DENSITY: [char; 9] = [' ', '.', ':', '-', '=', '+', '*', '#', '@'];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParticleKind {
    Exhaust,
    WarpTrail,
    Nebula,
    Sparkle,
}

#[derive(Clone, Copy)]
struct Particle {
    pos: (f32, f32),
    vel: (f32, f32),
    accel: (f32, f32),
    life: f32,
    max_life: f32,
    kind: ParticleKind,
}

/// Sumber partikel kontinu (mis. exhaust saat Traveling).
#[derive(Clone, Copy)]
pub struct Emitter {
    pub origin: (f32, f32),
    pub dir: f32,    // arah dasar (rad)
    pub rate: f32,   // partikel/detik
    accum: f32,      // akumulator spawn fractional
    pub spread: f32, // sudut sebar (rad)
    pub speed: (f32, f32),
    pub life: (f32, f32),
    pub accel: (f32, f32),
    pub kind: ParticleKind,
    pub enabled: bool,
}

impl Emitter {
    /// Preset Engine Exhaust (`15` §Contoh Efek): 40/s, mundur (dir=π), spread 0.3,
    /// speed 0.2–0.4 (sel/s ternormalisasi), life 0.4–0.8s. Aktif default.
    pub fn exhaust(origin: (f32, f32)) -> Self {
        Emitter {
            origin,
            dir: std::f32::consts::PI,
            rate: 40.0,
            accum: 0.0,
            spread: 0.3,
            speed: (0.2, 0.4),
            life: (0.4, 0.8),
            accel: (0.0, 0.0),
            kind: ParticleKind::Exhaust,
            enabled: true,
        }
    }

    /// Preset Nebula Drift (`15`): latar lambat, sebar penuh, umur panjang. Untuk Galaxy Map.
    pub fn nebula(origin: (f32, f32)) -> Self {
        Emitter {
            origin,
            dir: 0.0,
            rate: 5.0,
            accum: 0.0,
            spread: TAU,
            speed: (0.02, 0.06),
            life: (4.0, 8.0),
            accel: (0.0, 0.0),
            kind: ParticleKind::Nebula,
            enabled: true,
        }
    }
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
    emitters: Vec<Emitter>,
    rng: SplitMix64,
}

fn f32_01(rng: &mut SplitMix64) -> f32 {
    rng.next_f64() as f32
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

impl ParticleSystem {
    pub fn new(seed: u64) -> Self {
        ParticleSystem {
            particles: Vec::with_capacity(PARTICLE_BUDGET as usize),
            emitters: Vec::new(),
            rng: SplitMix64::new(seed),
        }
    }

    pub fn len(&self) -> usize {
        self.particles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }

    pub fn add_emitter(&mut self, e: Emitter) -> usize {
        self.emitters.push(e);
        self.emitters.len() - 1
    }

    /// Aktif/non-aktifkan emitter idx + set origin baru (mis. exhaust mengikuti ship).
    pub fn set_emitter(&mut self, idx: usize, enabled: bool, origin: (f32, f32)) {
        if let Some(e) = self.emitters.get_mut(idx) {
            e.enabled = enabled;
            e.origin = origin;
        }
    }

    /// Spawn `n` partikel sekaligus (efek burst: warp trail, sparkle loot).
    pub fn burst(&mut self, kind: ParticleKind, origin: (f32, f32), n: u32, speed: (f32, f32)) {
        for _ in 0..n {
            if self.particles.len() >= PARTICLE_BUDGET as usize {
                break;
            }
            let angle = f32_01(&mut self.rng) * TAU;
            let sp = lerp(speed.0, speed.1, f32_01(&mut self.rng));
            let life = lerp(0.3, 0.6, f32_01(&mut self.rng));
            self.particles.push(Particle {
                pos: origin,
                vel: (angle.cos() * sp, angle.sin() * sp / ASPECT as f32),
                accel: (0.0, 0.0),
                life,
                max_life: life,
                kind,
            });
        }
    }

    fn spawn_from(&mut self, ei: usize) {
        let e = self.emitters[ei];
        let angle = e.dir + (f32_01(&mut self.rng) - 0.5) * e.spread;
        let sp = lerp(e.speed.0, e.speed.1, f32_01(&mut self.rng));
        let life = lerp(e.life.0, e.life.1, f32_01(&mut self.rng));
        self.particles.push(Particle {
            pos: e.origin,
            vel: (angle.cos() * sp, angle.sin() * sp / ASPECT as f32),
            accel: e.accel,
            life,
            max_life: life,
            kind: e.kind,
        });
    }

    /// Integrasi Euler satu frame: spawn → gerak/decay → cull (mati/keluar `[0,1]²`).
    pub fn update(&mut self, dt: f32) {
        let n = self.emitters.len();
        for ei in 0..n {
            if !self.emitters[ei].enabled {
                continue;
            }
            self.emitters[ei].accum += self.emitters[ei].rate * dt;
            while self.emitters[ei].accum >= 1.0 && self.particles.len() < PARTICLE_BUDGET as usize
            {
                self.emitters[ei].accum -= 1.0;
                self.spawn_from(ei);
            }
        }
        for p in &mut self.particles {
            p.vel.0 += p.accel.0 * dt;
            p.vel.1 += p.accel.1 * dt;
            p.pos.0 += p.vel.0 * dt;
            p.pos.1 += p.vel.1 * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| {
            p.life > 0.0 && p.pos.0 >= 0.0 && p.pos.0 <= 1.0 && p.pos.1 >= 0.0 && p.pos.1 <= 1.0
        });
    }

    /// Warna per kind, di-fade dengan `age_frac` (1.0 muda → 0.0 tua).
    fn color(kind: ParticleKind, age: f32) -> Color {
        let (y, o) = match kind {
            ParticleKind::Exhaust => ((255, 255, 200), (160, 40, 0)),
            ParticleKind::WarpTrail => ((0, 255, 255), (200, 0, 255)),
            ParticleKind::Nebula => ((150, 70, 190), (90, 40, 120)),
            ParticleKind::Sparkle => ((120, 255, 150), (40, 120, 60)),
        };
        let l = |a: u8, b: u8| (lerp(b as f32, a as f32, age)) as u8;
        Color::Rgb(l(y.0, o.0), l(y.1, o.1), l(y.2, o.2))
    }

    /// Render ke `area`: akumulasi densitas per sel → glyph RAMP_DENSITY + warna kind termuda.
    pub fn render(&self, f: &mut Frame, area: Rect, _th: &Theme) {
        if self.particles.is_empty() || area.width < 1 || area.height < 1 {
            return;
        }
        let cells = area.width as usize * area.height as usize;
        let mut energy = vec![0.0f32; cells];
        let mut top = vec![None::<(ParticleKind, f32)>; cells]; // partikel termuda per sel
        let (w, h) = ((area.width - 1) as f32, (area.height - 1) as f32);
        for p in &self.particles {
            let col = (p.pos.0 * w).round() as i32;
            let row = (p.pos.1 * h).round() as i32;
            if col < 0 || row < 0 || col as u16 >= area.width || row as u16 >= area.height {
                continue;
            }
            let ix = row as usize * area.width as usize + col as usize;
            let age = (p.life / p.max_life).clamp(0.0, 1.0);
            energy[ix] += age;
            if top[ix].is_none_or(|(_, a)| age > a) {
                top[ix] = Some((p.kind, age));
            }
        }
        let buf = f.buffer_mut();
        for j in 0..area.height {
            for i in 0..area.width {
                let ix = j as usize * area.width as usize + i as usize;
                let e = energy[ix];
                if e <= 0.0 {
                    continue;
                }
                let gi = (e.floor() as usize).min(RAMP_DENSITY.len() - 1);
                let g = RAMP_DENSITY[gi];
                let (kind, age) = top[ix].unwrap();
                let cell = &mut buf[(area.x + i, area.y + j)];
                cell.set_char(g);
                cell.set_style(Style::default().fg(Self::color(kind, age)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exhaust_emitter() -> Emitter {
        Emitter::exhaust((0.5, 0.5))
    }

    #[test]
    fn emitter_spawns_proportional_to_rate() {
        let mut ps = ParticleSystem::new(1);
        ps.add_emitter(exhaust_emitter());
        ps.update(0.1); // 40/s * 0.1 = 4 partikel
        assert_eq!(ps.len(), 4);
    }

    #[test]
    fn budget_is_capped() {
        let mut ps = ParticleSystem::new(2);
        let mut e = exhaust_emitter();
        e.rate = 100_000.0;
        e.life = (100.0, 100.0); // tak mati selama test
        ps.add_emitter(e);
        ps.update(1.0);
        assert_eq!(ps.len(), PARTICLE_BUDGET as usize);
    }

    #[test]
    fn particles_decay_and_cull() {
        let mut ps = ParticleSystem::new(3);
        ps.burst(ParticleKind::Sparkle, (0.5, 0.5), 10, (0.0, 0.0));
        assert_eq!(ps.len(), 10);
        ps.update(1.0); // life maks 0.6 < 1.0 → semua mati
        assert_eq!(ps.len(), 0);
    }

    #[test]
    fn disabled_emitter_idle() {
        let mut ps = ParticleSystem::new(4);
        let idx = ps.add_emitter(exhaust_emitter());
        ps.set_emitter(idx, false, (0.5, 0.5));
        ps.update(0.5);
        assert!(ps.is_empty());
    }

    #[test]
    fn deterministic_same_seed() {
        let run = || {
            let mut ps = ParticleSystem::new(99);
            ps.add_emitter(exhaust_emitter());
            for _ in 0..5 {
                ps.update(0.05);
            }
            ps.len()
        };
        assert_eq!(run(), run());
    }
}
