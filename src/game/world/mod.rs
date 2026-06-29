//! Loader Milky Way (Sol System) dari `data/milky_way.ron` → `Galaxy` runtime. `09-milky-way.md`.
//!
//! DTO `Deserialize` (id resource string) → konversi ke `Galaxy`/`Planet` runtime (ResourceId
//! interned via `Content`). Pola sama dengan `content.rs` (defs = load, state = runtime).
//! Body dengan `unlock_req: None` → langsung `unlocked` (Earth start); sisanya terkunci sampai
//! travel tiba (`sim::tick::advance_travel`).
#![allow(dead_code)]

pub mod procgen;

use crate::balance::BASE_CAP;
use crate::game::defs::{Content, ResourceId};
use crate::game::state::{
    Biome, Factory, Galaxy, GalaxyId, GalaxyKind, NodeId, Planet, PlanetId, ResourceMap,
    ResourceNode, UnlockReq,
};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug)]
pub enum WorldError {
    Io(String),
    Parse(String),
    UnknownResource(String),
}

impl std::fmt::Display for WorldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorldError::Io(e) => write!(f, "baca milky_way.ron: {e}"),
            WorldError::Parse(e) => write!(f, "parse milky_way.ron: {e}"),
            WorldError::UnknownResource(r) => {
                write!(f, "resource '{r}' tak dikenal di milky_way.ron")
            }
        }
    }
}
impl std::error::Error for WorldError {}

#[derive(Deserialize)]
struct NodeDef {
    resource: String,
    richness: f64,
}

/// DTO unlock_req (resource sebagai string) → dikonversi ke `state::UnlockReq`.
#[derive(Deserialize)]
enum UnlockReqDef {
    None,
    WarpTier(u8),
    Resource(String, f64),
    All(Vec<UnlockReqDef>),
}

#[derive(Deserialize)]
struct BodyDef {
    id: String,
    name: String,
    tier: u8,
    biome: Biome,
    distance: f64,
    slots: u8,
    unlock_req: UnlockReqDef,
    nodes: Vec<NodeDef>,
    #[serde(default)]
    moons: Vec<String>,
}

#[derive(Deserialize)]
struct MilkyWayDef {
    id: String,
    name: String,
    bodies: Vec<BodyDef>,
}

fn rid(content: &Content, name: &str) -> Result<ResourceId, WorldError> {
    content
        .resources
        .id(name)
        .map(ResourceId)
        .ok_or_else(|| WorldError::UnknownResource(name.to_string()))
}

fn convert_req(content: &Content, def: &UnlockReqDef) -> Result<UnlockReq, WorldError> {
    Ok(match def {
        UnlockReqDef::None => UnlockReq::None,
        UnlockReqDef::WarpTier(t) => UnlockReq::WarpTier(*t),
        UnlockReqDef::Resource(r, amt) => UnlockReq::Resource(rid(content, r)?, *amt),
        UnlockReqDef::All(list) => UnlockReq::All(
            list.iter()
                .map(|r| convert_req(content, r))
                .collect::<Result<_, _>>()?,
        ),
    })
}

fn convert_body(content: &Content, idx: u32, b: &BodyDef) -> Result<Planet, WorldError> {
    let nodes = b
        .nodes
        .iter()
        .enumerate()
        .map(|(ni, n)| {
            Ok(ResourceNode {
                id: NodeId(ni as u32),
                resource: rid(content, &n.resource)?,
                richness: n.richness,
                level: 0,
            })
        })
        .collect::<Result<Vec<_>, WorldError>>()?;
    let req = convert_req(content, &b.unlock_req)?;
    Ok(Planet {
        id: PlanetId(idx),
        name: b.name.clone(),
        tier: b.tier,
        biome: b.biome,
        distance: b.distance,
        unlocked: matches!(b.unlock_req, UnlockReqDef::None),
        unlock_req: req,
        nodes,
        factory_slots: vec![None::<Factory>; b.slots as usize],
        stockpile: ResourceMap::new(),
        stockpile_cap: BASE_CAP,
    })
}

/// Load + materialisasi Milky Way (Sol System) sebagai `Galaxy` level 0 (Fixed, Anchor).
pub fn load_milky_way(dir: &Path, content: &Content) -> Result<Galaxy, WorldError> {
    let text = std::fs::read_to_string(dir.join("milky_way.ron"))
        .map_err(|e| WorldError::Io(e.to_string()))?;
    let def: MilkyWayDef = ron::from_str(&text).map_err(|e| WorldError::Parse(e.to_string()))?;
    let planets = def
        .bodies
        .iter()
        .enumerate()
        .map(|(i, b)| convert_body(content, i as u32, b))
        .collect::<Result<Vec<_>, WorldError>>()?;
    let _ = &def.id;
    Ok(Galaxy {
        id: GalaxyId(0),
        name: def.name,
        level: 0,
        kind: GalaxyKind::Fixed,
        planets,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_content;

    fn data_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
    }

    fn content() -> Content {
        load_content(&data_dir()).unwrap()
    }

    #[test]
    fn loads_sol_system() {
        let c = content();
        let g = load_milky_way(&data_dir(), &c).expect("milky_way load");
        // 24 body Sol handcrafted (inner+belt+outer+kuiper) di data/milky_way.ron.
        assert_eq!(g.planets.len(), 24);
        assert_eq!(g.level, 0);
        // Earth = unlock None → unlocked sejak start; punya 6 node + 6 slot.
        let earth = g.planets.iter().find(|p| p.name == "Earth").unwrap();
        assert!(earth.unlocked);
        assert_eq!(earth.nodes.len(), 6);
        assert_eq!(earth.factory_slots.len(), 6);
        assert!((earth.distance - 1.0).abs() < 1e-9);
        // Luna = WarpTier(1) → terkunci sampai travel.
        let luna = g
            .planets
            .iter()
            .find(|p| p.name.starts_with("Luna"))
            .unwrap();
        assert!(!luna.unlocked);
        assert!(matches!(luna.unlock_req, UnlockReq::WarpTier(1)));
    }

    #[test]
    fn unknown_resource_rejected() {
        // Loader harus error untuk resource menggantung — diuji via konversi langsung.
        let c = content();
        let bad = NodeDef {
            resource: "unobtanium".into(),
            richness: 1.0,
        };
        let body = BodyDef {
            id: "x".into(),
            name: "X".into(),
            tier: 1,
            biome: Biome::Terran,
            distance: 1.0,
            slots: 1,
            unlock_req: UnlockReqDef::None,
            nodes: vec![bad],
            moons: vec![],
        };
        assert!(matches!(
            convert_body(&c, 0, &body),
            Err(WorldError::UnknownResource(_))
        ));
    }
}
