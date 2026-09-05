//! Katalog POI (Point of Interest) dekoratif per galaksi -- MURNI visual/flavor (nama bintang/
//! gugus notable utk fokus kamera, Phase 7), TAK terikat mekanik/ekonomi game sama sekali
//! (planet/starmap asli tetap `GalaxyMapMode::Starmap`, tak disentuh sedikitpun). Anchor
//! (Milky Way) pakai nama familiar; galaksi frontier pakai `gen_name` yg SAMA dipakai
//! planet/galaksi (`game::world::procgen`) -- konsisten gaya penamaan lintas game, bukan
//! sistem penamaan baru terpisah.

use super::model::PoiOrbitKey;
use crate::game::world::procgen::gen_name;
use crate::rng::derive;

/// Satu entri katalog: nama tampilan + kunci orbit (utk lookup posisi via `GalaxyModel::
/// poi_orbit`).
#[derive(Clone, Debug)]
pub struct Poi {
    pub name: String,
    pub orbit_key: PoiOrbitKey,
}

/// Katalog POI galaksi `seed` (3 entri tetap: 2 bintang notable + 1 gugus, sesuai
/// `PoiOrbitKey`+`generate.rs`'s populasi "notable"). `is_anchor` (Milky Way) pakai nama
/// literal familiar; frontier deterministik dr seed (seed sama -> nama sama, seed beda ->
/// nama beda, pola SAMA planet/galaxy naming).
pub fn catalog(seed: u64, is_anchor: bool) -> Vec<Poi> {
    if is_anchor {
        return vec![
            Poi {
                name: "Sol".into(),
                orbit_key: PoiOrbitKey::NotableStar1,
            },
            Poi {
                name: "Rigel".into(),
                orbit_key: PoiOrbitKey::NotableStar2,
            },
            Poi {
                name: "Orion Nebula".into(),
                orbit_key: PoiOrbitKey::NotableCluster,
            },
        ];
    }
    vec![
        Poi {
            name: gen_name(derive(seed, "poi_star1")),
            orbit_key: PoiOrbitKey::NotableStar1,
        },
        Poi {
            name: gen_name(derive(seed, "poi_star2")),
            orbit_key: PoiOrbitKey::NotableStar2,
        },
        Poi {
            name: format!("{} Nebula", gen_name(derive(seed, "poi_cluster"))),
            orbit_key: PoiOrbitKey::NotableCluster,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Anchor (Milky Way) HARUS genuinely pakai nama literal familiar (BUKAN procedural) --
    /// dites LANGSUNG isi string, bukan cuma panjang/non-kosong.
    #[test]
    fn anchor_uses_literal_familiar_names() {
        let c = catalog(0, true);
        assert_eq!(c.len(), 3);
        assert_eq!(c[0].name, "Sol");
        assert_eq!(c[1].name, "Rigel");
        assert_eq!(c[2].name, "Orion Nebula");
    }

    /// Katalog frontier HARUS genuinely deterministik (seed sama -> SEMUA nama sama persis).
    #[test]
    fn frontier_catalog_is_deterministic() {
        let a = catalog(777, false);
        let b = catalog(777, false);
        for (x, y) in a.iter().zip(&b) {
            assert_eq!(x.name, y.name);
            assert_eq!(x.orbit_key, y.orbit_key);
        }
    }

    /// Seed BEDA HARUS genuinely hasilkan nama BEDA (bukan katalog statis dgn seed diabaikan)
    /// -- dites lintas beberapa seed, bukan satu pasang saja.
    #[test]
    fn different_seeds_produce_different_names() {
        let names_for = |seed: u64| {
            catalog(seed, false)
                .into_iter()
                .map(|p| p.name)
                .collect::<Vec<_>>()
        };
        let n1 = names_for(1);
        let n2 = names_for(2);
        let n3 = names_for(3);
        assert!(
            n1 != n2 || n2 != n3,
            "minimal 1 pasang seed HARUS genuinely beda nama"
        );
    }

    /// Dalam SATU katalog, 3 entri HARUS genuinely py nama BEDA satu sama lain (bukan
    /// kebetulan sama krn `derive` label collision) -- dites label derive genuinely berbeda.
    #[test]
    fn entries_within_one_catalog_have_distinct_names() {
        for seed in [1u64, 42, 999, 123_456] {
            let c = catalog(seed, false);
            let names: Vec<_> = c.iter().map(|p| p.name.as_str()).collect();
            assert_ne!(names[0], names[1], "seed={seed}");
            assert_ne!(names[1], names[2], "seed={seed}");
        }
    }
}
