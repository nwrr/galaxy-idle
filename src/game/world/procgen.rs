//! ProcGen galaksi luar (level ≥1): generate galaxy/planet deterministik dari seed.
//! Spec: `16-procgen-logic.md` (kontrak determinisme, pipeline, name gen).
//!
//! Semua output **hanya** fungsi dari `(global_seed, level, idx)` → save cukup menyimpan
//! `GalaxyKind::Procedural { seed, visited }`; planet belum dikunjungi dihitung ulang on-the-fly.
//! Urutan konsumsi RNG di `generate_planet` TETAP (dist, angle, biome, nodes, anomaly, name) —
//! field baru ditambah di AKHIR agar seed lama tidak bergeser.
//!
//! Asumsi (spec ambigu → default aman, lihat ITERATION_LOG iter 29):
//! - `BIOME_TABLE` & `NODES_FOR_BIOME` dari `08-balancing.md`; `LavaWorld` tidak masuk tabel.
//! - `level_bias`: biome rare (CrystalWorld/GasGiant) bobotnya naik 0.15×(level−1).
//! - resource dalam satu biome: bobot menurun per posisi (yang pertama paling umum, 4..1).
//! - slot factory planet ProcGen = `2 + tier`; `unlock_req = WarpTier(tier)` (sinkron tier↔warp).
#![allow(dead_code)]

use crate::balance::{
    ANOMALY_CHANCE, DIST_D1, DIST_D2, DIST_MAX, DIST_MIN, NODE_MAX, NODE_MIN, PLANET_MAX,
    PLANET_MIN, RICH_MAX, RICH_MIN,
};
use crate::game::defs::{Content, ResourceId};
use crate::game::state::{
    Biome, Galaxy, GalaxyKind, NodeId, Planet, PlanetId, ResourceMap, ResourceNode, UnlockReq,
};
use crate::rng::{SplitMix64, derive, planet_rng};
use std::f64::consts::TAU;

const GOLDEN: u64 = 0x9E3779B97F4A7C15;

/// Metadata galaksi ProcGen (ringkas; planet dihitung on-the-fly dari `seed`).
#[derive(Clone, Debug, PartialEq)]
pub struct GalaxyMeta {
    pub seed: u64,
    pub level: u8,
    pub tier: u8,
    pub planet_count: u32,
    pub name: String,
}

/// `BIOME_TABLE` + `NODES_FOR_BIOME` (08-balancing §ProcGen). Bobot & resource per biome.
/// `LavaWorld` sengaja tidak digenerate (tidak ada di tabel spec).
const BIOME_TABLE: &[(Biome, f64, &[&str])] = &[
    (
        Biome::IronWorld,
        22.0,
        &["iron", "copper", "nickel", "carbon"],
    ),
    (
        Biome::OceanPlanet,
        16.0,
        &["water", "brine", "silicon_ore", "deuterium"],
    ),
    (
        Biome::GasGiant,
        14.0,
        &["hydrogen", "helium", "helium3", "exotic_gas"],
    ),
    (
        Biome::DeadWorld,
        18.0,
        &["iron", "sulfur", "uranium", "lithium"],
    ),
    (
        Biome::CrystalWorld,
        8.0,
        &["quartz", "rare_earth", "rare_crystals", "dilithium"],
    ),
    (Biome::Terran, 8.0, &["iron", "carbon", "water", "copper"]),
    (
        Biome::AsteroidBelt,
        8.0,
        &["nickel", "platinum", "iridium", "gold"],
    ),
    (
        Biome::IceWorld,
        6.0,
        &["water", "liquid_methane", "ammonia", "deuterium"],
    ),
];

fn mix(x: u64) -> u64 {
    x.wrapping_mul(GOLDEN)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Multiplier richness & jarak per level galaksi: `1.0 + 0.5·(level−1)` (08-balancing).
pub fn galaxy_tier_mult(level: u8) -> f64 {
    1.0 + 0.5 * (level.saturating_sub(1) as f64)
}

/// Seed galaksi: `splitmix(GLOBAL_SEED ^ mix(level) ^ mix(idx))` (16 §Hirarki Seed).
pub fn galaxy_seed(global_seed: u64, level: u8, idx: u32) -> u64 {
    let s = global_seed ^ mix(level as u64) ^ mix(idx as u64);
    SplitMix64::new(s).next_u64()
}

/// Geser bobot ke biome rare saat level naik (16 §Biome).
fn level_bias(biome: Biome, level: u8) -> f64 {
    if matches!(biome, Biome::CrystalWorld | Biome::GasGiant) {
        1.0 + 0.15 * (level.saturating_sub(1) as f64)
    } else {
        1.0
    }
}

/// Weighted pick biome dari `BIOME_TABLE` (cumulative ≥ roll).
fn weighted_biome(r: &mut SplitMix64, level: u8) -> (Biome, &'static [&'static str]) {
    let total: f64 = BIOME_TABLE
        .iter()
        .map(|(b, w, _)| w * level_bias(*b, level))
        .sum();
    let roll = r.next_f64() * total;
    let mut acc = 0.0;
    for (b, w, nodes) in BIOME_TABLE {
        acc += w * level_bias(*b, level);
        if acc >= roll {
            return (*b, nodes);
        }
    }
    let (b, _, nodes) = BIOME_TABLE[BIOME_TABLE.len() - 1];
    (b, nodes)
}

/// Tier planet dari jarak; galaksi level ≥3 menaikkan tier dasar (16 §Tier dari Jarak).
fn tier_from_distance(dist: f64, level: u8) -> u8 {
    let base = if dist < DIST_D1 {
        1
    } else if dist < DIST_D2 {
        2
    } else {
        3
    };
    (base + if level >= 3 { 1 } else { 0 }).clamp(1, 3)
}

/// Pilih resource dalam biome: bobot menurun per posisi (pertama = paling umum).
fn pick_resource(r: &mut SplitMix64, templates: &[&'static str]) -> &'static str {
    let n = templates.len();
    let total: f64 = (1..=n).map(|w| w as f64).sum();
    let roll = r.next_f64() * total;
    let mut acc = 0.0;
    for (i, name) in templates.iter().enumerate() {
        acc += (n - i) as f64; // posisi 0 → bobot n, posisi terakhir → 1
        if acc >= roll {
            return name;
        }
    }
    templates[n - 1]
}

fn rid(content: &Content, name: &str) -> Option<ResourceId> {
    content.resources.id(name).map(ResourceId)
}

/// Generate node planet dari biome (16 §Generate Nodes). Resource tak dikenal dilewati.
fn gen_nodes(
    r: &mut SplitMix64,
    templates: &[&'static str],
    level: u8,
    content: &Content,
) -> Vec<ResourceNode> {
    let count = NODE_MIN + r.below(NODE_MAX - NODE_MIN + 1); // inklusif [NODE_MIN, NODE_MAX]
    let mult = galaxy_tier_mult(level);
    (0..count)
        .filter_map(|i| {
            let name = pick_resource(r, templates);
            let richness = lerp(RICH_MIN, RICH_MAX, r.next_f64()) * mult;
            rid(content, name).map(|resource| ResourceNode {
                id: NodeId(i),
                resource,
                richness,
                level: 0,
            })
        })
        .collect()
}

// ── Name generator (16 §Name Generator) ─────────────────────────────────────
const PREFIX: &[&str] = &[
    "And", "Ori", "Vega", "Cyg", "Lyr", "Cas", "Dra", "Hel", "Xan", "Zor", "Kep", "Tau",
];
const MID: &[&str] = &[
    "o", "a", "e", "ome", "ade", "ira", "une", "ax", "yx", "or", "el", "is",
];
const SUFFIX: &[&str] = &[
    "da", "on", "us", "ar", "ix", "eus", "ara", "ion", "is", "ae", "or",
];
const GREEK: &[&str] = &["Alpha", "Beta", "Gamma", "Delta", "Prime", "Minor", "Major"];

fn pick<'a>(r: &mut SplitMix64, table: &[&'a str]) -> &'a str {
    table[r.below(table.len() as u32) as usize]
}

/// Nama deterministik dari sub-seed (penggabung suku-kata + sufiks Greek opsional).
pub fn gen_name(seed: u64) -> String {
    let mut r = SplitMix64::new(seed);
    let mut s = format!(
        "{}{}{}",
        pick(&mut r, PREFIX),
        pick(&mut r, MID),
        pick(&mut r, SUFFIX)
    );
    if r.next_f64() < 0.3 {
        s.push(' ');
        s.push_str(pick(&mut r, GREEK));
    }
    s
}

/// Jumlah planet galaksi dari `galaxy_seed` (rekonstruksi dari save `Procedural { seed }`).
pub fn planet_count(galaxy_seed: u64) -> u32 {
    let mut layout = SplitMix64::new(derive(galaxy_seed, "layout"));
    let span = (PLANET_MAX - PLANET_MIN + 1) as u64;
    PLANET_MIN + (layout.next_u64() % span) as u32
}

/// Nama galaksi dari `galaxy_seed` (deterministik, sama dgn `generate_galaxy`).
pub fn galaxy_name(galaxy_seed: u64) -> String {
    gen_name(derive(galaxy_seed, "name"))
}

/// Generate metadata galaksi ProcGen pada `(global_seed, level, idx)`.
pub fn generate_galaxy(global_seed: u64, level: u8, idx: u32) -> GalaxyMeta {
    let seed = galaxy_seed(global_seed, level, idx);
    GalaxyMeta {
        seed,
        level,
        tier: level,
        planet_count: planet_count(seed),
        name: galaxy_name(seed),
    }
}

/// Materialisasi planet index `idx` ke galaksi ProcGen (16 §Materialisasi & Save):
/// hitung on-the-fly, push ke `planets`, tandai `visited`. No-op bila bukan ProcGen,
/// `idx` di luar rentang, atau sudah dimaterialisasi.
pub fn materialize_planet(galaxy: &mut Galaxy, idx: u32, content: &Content) -> bool {
    let GalaxyKind::Procedural { seed, visited } = &mut galaxy.kind else {
        return false;
    };
    if idx >= planet_count(*seed) || visited.contains(&idx) {
        return false;
    }
    let planet = generate_planet(*seed, idx, galaxy.level, content);
    visited.insert(idx);
    galaxy.planets.push(planet);
    true
}

/// Generate satu planet murni dari seed (preview/materialisasi). Urutan RNG TETAP.
pub fn generate_planet(galaxy_seed: u64, i: u32, level: u8, content: &Content) -> Planet {
    let mut r = planet_rng(galaxy_seed, i);
    // 1. Posisi polar (dist untuk travel; angle dicadangkan utk Galaxy Map).
    let distance = lerp(DIST_MIN, DIST_MAX, r.next_f64()) * galaxy_tier_mult(level);
    let _angle = r.next_f64() * TAU;
    // 2. Biome (weighted) → 3. tier dari jarak → 4. nodes.
    let (biome, templates) = weighted_biome(&mut r, level);
    let tier = tier_from_distance(distance, level);
    let nodes = gen_nodes(&mut r, templates, level, content);
    // 5. Anomaly seeding (dicadangkan utk event; field belum di Planet).
    let _anomaly = r.next_f64() < ANOMALY_CHANCE * (1.0 + 0.1 * level as f64);
    // 6. Nama.
    let name = gen_name(derive(galaxy_seed, &format!("planet{i}")));
    let slots = (2 + tier as usize).min(8);
    Planet {
        id: PlanetId(i),
        name,
        tier,
        biome,
        distance,
        unlocked: false,
        unlock_req: UnlockReq::WarpTier(tier),
        nodes,
        factory_slots: vec![None; slots],
        stockpile: ResourceMap::new(),
        stockpile_cap: crate::balance::BASE_CAP,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_content;
    use std::path::Path;

    fn content() -> Content {
        load_content(&Path::new(env!("CARGO_MANIFEST_DIR")).join("data")).unwrap()
    }

    fn in_table(b: Biome) -> bool {
        BIOME_TABLE.iter().any(|(tb, _, _)| *tb == b)
    }

    #[test]
    fn galaxy_deterministic_and_bounded() {
        let a = generate_galaxy(0xCAFE, 2, 7);
        let b = generate_galaxy(0xCAFE, 2, 7);
        assert_eq!(a, b);
        for idx in 0..50 {
            let g = generate_galaxy(0xCAFE, 1, idx);
            assert!(g.planet_count >= PLANET_MIN && g.planet_count <= PLANET_MAX);
            assert!(!g.name.is_empty());
        }
        // Seed berbeda → galaksi (kemungkinan besar) berbeda.
        assert_ne!(generate_galaxy(1, 1, 0).seed, generate_galaxy(2, 1, 0).seed);
    }

    #[test]
    fn planet_deterministic_field_by_field() {
        let c = content();
        let seed = galaxy_seed(0xBEEF, 3, 4);
        let p1 = generate_planet(seed, 5, 3, &c);
        let p2 = generate_planet(seed, 5, 3, &c);
        assert_eq!(p1.name, p2.name);
        assert_eq!(p1.biome, p2.biome);
        assert_eq!(p1.tier, p2.tier);
        assert!((p1.distance - p2.distance).abs() < 1e-12);
        assert_eq!(p1.nodes.len(), p2.nodes.len());
        for (n1, n2) in p1.nodes.iter().zip(&p2.nodes) {
            assert_eq!(n1.resource, n2.resource);
            assert!((n1.richness - n2.richness).abs() < 1e-12);
        }
    }

    #[test]
    fn planet_invariants_hold() {
        let c = content();
        let seed = galaxy_seed(0x1234, 2, 1);
        for i in 0..40 {
            let p = generate_planet(seed, i, 2, &c);
            assert!(in_table(p.biome), "biome harus dari BIOME_TABLE");
            assert!((1..=3).contains(&p.tier));
            assert!(!p.unlocked);
            assert!(matches!(p.unlock_req, UnlockReq::WarpTier(t) if t == p.tier));
            assert!(p.nodes.len() >= NODE_MIN as usize && p.nodes.len() <= NODE_MAX as usize);
            assert!(!p.factory_slots.is_empty());
            for n in &p.nodes {
                assert!(n.richness > 0.0);
            }
        }
    }

    #[test]
    fn count_matches_meta_and_materialize_is_idempotent() {
        use crate::game::state::{Galaxy, GalaxyId, GalaxyKind};
        let c = content();
        let meta = generate_galaxy(0x5EED, 1, 2);
        assert_eq!(planet_count(meta.seed), meta.planet_count);
        assert_eq!(galaxy_name(meta.seed), meta.name);

        let mut g = Galaxy {
            id: GalaxyId(1),
            name: meta.name.clone(),
            level: 1,
            kind: GalaxyKind::Procedural {
                seed: meta.seed,
                visited: Default::default(),
            },
            planets: vec![],
        };
        assert!(materialize_planet(&mut g, 0, &c));
        assert!(!materialize_planet(&mut g, 0, &c)); // sudah visited → no-op
        assert!(!materialize_planet(&mut g, meta.planet_count, &c)); // out of range
        assert_eq!(g.planets.len(), 1);
        // Planet termaterialisasi identik dengan preview murni (save kecil round-trip).
        let preview = generate_planet(meta.seed, 0, 1, &c);
        assert_eq!(g.planets[0].name, preview.name);
        assert_eq!(g.planets[0].biome, preview.biome);
    }

    #[test]
    fn different_global_seed_diverges_planets() {
        let c = content();
        let pa = generate_planet(galaxy_seed(1, 1, 0), 0, 1, &c);
        let pb = generate_planet(galaxy_seed(2, 1, 0), 0, 1, &c);
        // Sangat mungkin berbeda di salah satu field utama.
        assert!(pa.name != pb.name || (pa.distance - pb.distance).abs() > 1e-9);
    }
}
