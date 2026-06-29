//! Mekanik ekonomi: extractor (node→stockpile), refinery (recipe), market, auto-sell, scaling.
//!
//! Rumus dari `02-economy.md`; konstanta `balance.rs`. Fungsi di sini **murni per-planet**;
//! orkestrasi urutan fase tick (upkeep, galaksi aktif, tech_mult) ada di `sim::tick`.
//!
//! NOTE(scaffold): `allow(dead_code)` — dipanggil dari `sim::tick` (M2 item 3) & UI/aksi (M4).
#![allow(dead_code)]

use crate::balance::{BASE_EXTRACTOR_RATE, GROWTH, TICK_DURATION_SECS};
use crate::game::defs::{Content, RecipeId, ResourceId};
use crate::game::state::{FactoryKind, Planet, ResourceMap};
use std::collections::HashSet;

/// Output extractor /detik (sebelum dikalikan durasi tick). `02` §2.
pub fn extractor_output(level: u32, richness: f64, tech_mult: f64) -> f64 {
    BASE_EXTRACTOR_RATE * level as f64 * richness * tech_mult
}

/// Biaya upgrade geometrik: `base * GROWTH^level_sekarang`. `02` §5.
pub fn upgrade_cost(base_cost: f64, current_level: u32) -> f64 {
    base_cost * GROWTH.powi(current_level as i32)
}

/// Harga jual /unit = `base_price` resource di katalog. `02` §4 / `10`.
pub fn price(content: &Content, res: ResourceId) -> f64 {
    content.resources.get(res.0).base_price
}

/// Tambah `amount` ke stockpile, dibatasi `cap` per-resource. Kembalikan kelebihan yang terbuang.
pub fn add_capped(map: &mut ResourceMap, res: ResourceId, amount: f64, cap: f64) -> f64 {
    let cur = map.entry(res).or_insert(0.0);
    let room = (cap - *cur).max(0.0);
    let added = amount.min(room);
    *cur += added;
    amount - added
}

fn stock(map: &ResourceMap, res: ResourceId) -> f64 {
    map.get(&res).copied().unwrap_or(0.0)
}

/// Fase extractor: tiap Extractor enabled menambang node-nya → stockpile (per-resource cap).
/// `tech_mult` memberi multiplier per resource (default 1.0; dinaikkan research M5).
pub fn run_extractors(planet: &mut Planet, tech_mult: impl Fn(ResourceId) -> f64) {
    // Kumpulkan dulu (resource, amount) agar tidak meminjam slots & stockpile sekaligus.
    let mut gains: Vec<(ResourceId, f64)> = Vec::new();
    for slot in planet.factory_slots.iter().flatten() {
        if !slot.enabled || slot.level == 0 {
            continue;
        }
        if let FactoryKind::Extractor { node } = slot.kind
            && let Some(n) = planet.nodes.iter().find(|n| n.id == node)
        {
            let out = extractor_output(slot.level, n.richness, tech_mult(n.resource))
                * TICK_DURATION_SECS;
            gains.push((n.resource, out));
        }
    }
    let cap = planet.stockpile_cap;
    for (res, amt) in gains {
        add_capped(&mut planet.stockpile, res, amt, cap);
    }
}

/// Satu resource yang kurang (untuk feedback "Deficit <resource>"). `02` §6.
pub type Deficit = ResourceId;

/// Resolusi id string recipe → (ResourceId, jumlah). `None` bila resource tak dikenal (tak terjadi
/// pasca-`content.validate()`).
fn resolve(content: &Content, pairs: &[(String, f64)]) -> Option<Vec<(ResourceId, f64)>> {
    pairs
        .iter()
        .map(|(name, q)| content.resources.id(name).map(|h| (ResourceId(h), *q)))
        .collect()
}

/// Fase refinery: tiap Refinery enabled menjalankan recipe-nya `level` kali (recipe instan,
/// `craft_time_secs == 0`). Recipe blueprint hanya jalan bila dimiliki. Kembalikan deficit.
/// Recipe ber-waktu (>0) ditunda (butuh state progress per-factory; M5/UI craft).
pub fn run_refineries(
    planet: &mut Planet,
    content: &Content,
    blueprints: &HashSet<RecipeId>,
) -> Vec<Deficit> {
    let mut jobs: Vec<(u32, RecipeId)> = Vec::new();
    for slot in planet.factory_slots.iter().flatten() {
        if !slot.enabled || slot.level == 0 {
            continue;
        }
        if let FactoryKind::Refinery { recipe } = slot.kind {
            jobs.push((slot.level, recipe));
        }
    }
    let cap = planet.stockpile_cap;
    let mut deficits = Vec::new();
    for (level, recipe_id) in jobs {
        let rec = content.recipes.get(recipe_id.0);
        if rec.craft_time_secs != 0.0 {
            continue; // recipe ber-waktu: ditunda (lihat doc).
        }
        if rec.requires_blueprint && !blueprints.contains(&recipe_id) {
            continue; // terkunci sampai blueprint dimiliki.
        }
        let (Some(inputs), Some(outputs)) = (
            resolve(content, &rec.inputs),
            resolve(content, &rec.outputs),
        ) else {
            continue;
        };
        for _ in 0..level {
            let enough = inputs
                .iter()
                .all(|(r, q)| stock(&planet.stockpile, *r) >= *q);
            if !enough {
                if let Some((missing, _)) = inputs
                    .iter()
                    .find(|(r, q)| stock(&planet.stockpile, *r) < *q)
                {
                    deficits.push(*missing);
                }
                break;
            }
            for (r, q) in &inputs {
                *planet.stockpile.entry(*r).or_insert(0.0) -= q;
            }
            for (r, q) in &outputs {
                add_capped(&mut planet.stockpile, *r, *q, cap);
            }
        }
    }
    deficits
}

/// Auto-sell satu planet: jual surplus di atas `keep_above` untuk tiap rule enabled.
/// Kembalikan total Credits diperoleh (caller menambah ke `GameState.credits`). `02` §4.
pub fn auto_sell_planet(
    planet: &mut Planet,
    content: &Content,
    rules: &[crate::game::state::AutoSellRule],
) -> f64 {
    let mut credits = 0.0;
    for rule in rules.iter().filter(|r| r.enabled) {
        let have = stock(&planet.stockpile, rule.resource);
        let surplus = have - rule.keep_above;
        if surplus > 0.0 {
            credits += surplus * price(content, rule.resource);
            planet.stockpile.insert(rule.resource, rule.keep_above);
        }
    }
    credits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_content;
    use crate::game::state::{
        AutoSellRule, Biome, Factory, FactoryId, NodeId, Planet, PlanetId, ResourceNode, UnlockReq,
    };
    use std::path::Path;

    fn content() -> Content {
        load_content(&Path::new(env!("CARGO_MANIFEST_DIR")).join("data")).unwrap()
    }

    fn rid(c: &Content, name: &str) -> ResourceId {
        ResourceId(c.resources.id(name).unwrap())
    }

    fn empty_planet() -> Planet {
        Planet {
            id: PlanetId(0),
            name: "Test".into(),
            tier: 1,
            biome: Biome::Terran,
            distance: 1.0,
            unlocked: true,
            unlock_req: UnlockReq::None,
            nodes: vec![],
            factory_slots: vec![],
            stockpile: ResourceMap::new(),
            stockpile_cap: 1000.0,
        }
    }

    #[test]
    fn cost_grows_geometric() {
        assert!((upgrade_cost(50.0, 0) - 50.0).abs() < 1e-9);
        assert!((upgrade_cost(50.0, 1) - 57.5).abs() < 1e-9);
    }

    #[test]
    fn extractor_fills_stockpile() {
        let c = content();
        let iron = rid(&c, "iron");
        let mut p = empty_planet();
        p.nodes.push(ResourceNode {
            id: NodeId(0),
            resource: iron,
            richness: 2.0,
            level: 1,
        });
        p.factory_slots.push(Some(Factory {
            id: FactoryId(0),
            building: crate::game::defs::BuildingId(c.buildings.id("mining_drill").unwrap()),
            kind: FactoryKind::Extractor { node: NodeId(0) },
            level: 3,
            enabled: true,
        }));
        run_extractors(&mut p, |_| 1.0);
        // BASE_EXTRACTOR_RATE(1) * level(3) * richness(2) = 6 /tick.
        assert!((stock(&p.stockpile, iron) - 6.0).abs() < 1e-9);
    }

    #[test]
    fn refinery_crafts_and_reports_deficit() {
        let c = content();
        let iron = rid(&c, "iron");
        let carbon = rid(&c, "carbon");
        let steel = rid(&c, "steel");
        let steel_recipe = RecipeId(c.recipes.id("steel_mill").unwrap());
        let mut p = empty_planet();
        p.factory_slots.push(Some(Factory {
            id: FactoryId(0),
            building: crate::game::defs::BuildingId(c.buildings.id("steel_mill_bld").unwrap()),
            kind: FactoryKind::Refinery {
                recipe: steel_recipe,
            },
            level: 1,
            enabled: true,
        }));
        // Cukup utk 1 craft: 2 iron + 1 carbon → 1 steel.
        p.stockpile.insert(iron, 2.0);
        p.stockpile.insert(carbon, 1.0);
        let d = run_refineries(&mut p, &c, &HashSet::new());
        assert!(d.is_empty());
        assert!((stock(&p.stockpile, steel) - 1.0).abs() < 1e-9);
        assert!(stock(&p.stockpile, iron) < 1e-9);
        // Tanpa input → deficit.
        let d2 = run_refineries(&mut p, &c, &HashSet::new());
        assert!(!d2.is_empty());
    }

    #[test]
    fn auto_sell_keeps_threshold() {
        let c = content();
        let iron = rid(&c, "iron");
        let mut p = empty_planet();
        p.stockpile.insert(iron, 100.0);
        let rules = vec![AutoSellRule {
            resource: iron,
            keep_above: 30.0,
            enabled: true,
        }];
        let gained = auto_sell_planet(&mut p, &c, &rules);
        // 70 surplus × base_price iron (1.0) = 70 credits; sisa = 30.
        assert!((gained - 70.0).abs() < 1e-9);
        assert!((stock(&p.stockpile, iron) - 30.0).abs() < 1e-9);
    }
}
