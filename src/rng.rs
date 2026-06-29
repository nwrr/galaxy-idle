//! RNG deterministik ProcGen: `SplitMix64`, `planet_rng`, `derive`.
//!
//! Reproducible hanya dari seed → save tetap kecil (`07-architecture.md` §RNG,
//! `16-procgen-logic.md`). RNG event saat travel pakai sumber non-deterministik
//! terpisah (tidak di sini).
//!
//! NOTE(scaffold): `allow(dead_code)` sementara — API ini dikonsumsi ProcGen di M9
//! (`game/procgen.rs`). Hapus allow saat sudah dipakai dari jalur `main`.
#![allow(dead_code)]

const GOLDEN: u64 = 0x9E3779B97F4A7C15;

/// Generator splitmix64: cepat, zero-dep, deterministik dari state awal.
pub struct SplitMix64(pub u64);

impl SplitMix64 {
    /// RNG baru dari seed.
    pub fn new(seed: u64) -> Self {
        SplitMix64(seed)
    }

    /// `u64` acak berikutnya (advance state).
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(GOLDEN);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    /// `f64` di `[0, 1)`.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// `u32` di `[0, n)` (n > 0). Modulo bias diabaikan (cukup utk ProcGen kosmetik).
    pub fn below(&mut self, n: u32) -> u32 {
        (self.next_u64() % n as u64) as u32
    }
}

/// RNG planet: turunkan dari `(galaxy_seed, planet_index)` — selalu sama.
pub fn planet_rng(galaxy_seed: u64, planet_index: u32) -> SplitMix64 {
    SplitMix64(galaxy_seed ^ (planet_index as u64).wrapping_mul(GOLDEN))
}

/// Turunkan sub-seed dari seed induk + label arbitrer (mis. "biome", "name").
/// Deterministik: seed sama + label sama → hasil sama.
pub fn derive(seed: u64, label: &str) -> u64 {
    let mut h = seed;
    for b in label.bytes() {
        h = (h ^ b as u64).wrapping_mul(GOLDEN);
        h ^= h >> 29;
    }
    let mut s = SplitMix64(h);
    s.next_u64()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_sequence() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = SplitMix64::new(1);
        let mut b = SplitMix64::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn next_f64_in_range() {
        let mut r = SplitMix64::new(7);
        for _ in 0..1000 {
            let x = r.next_f64();
            assert!((0.0..1.0).contains(&x));
        }
    }

    #[test]
    fn below_bounded() {
        let mut r = SplitMix64::new(99);
        for _ in 0..1000 {
            assert!(r.below(6) < 6);
        }
    }

    #[test]
    fn planet_rng_reproducible() {
        let mut a = planet_rng(0xDEAD_BEEF, 3);
        let mut b = planet_rng(0xDEAD_BEEF, 3);
        assert_eq!(a.next_u64(), b.next_u64());
        let mut c = planet_rng(0xDEAD_BEEF, 4);
        assert_ne!(planet_rng(0xDEAD_BEEF, 3).next_u64(), c.next_u64());
    }

    #[test]
    fn derive_stable_and_label_sensitive() {
        assert_eq!(derive(123, "biome"), derive(123, "biome"));
        assert_ne!(derive(123, "biome"), derive(123, "name"));
        assert_ne!(derive(123, "biome"), derive(124, "biome"));
    }
}
