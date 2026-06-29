//! Loader konten data-driven: `data/*.ron` → `Content` (intern id + validasi referensi).
//!
//! Gagal-cepat saat startup bila id duplikat atau referensi silang menggantung
//! (`07-architecture.md` §Content Loading). Struct di `game::defs`.
//!
//! NOTE(scaffold): `allow(dead_code)` — `load_content` dipanggil dari `app::run` mulai M3.
#![allow(dead_code)]

use crate::game::defs::{
    BuildingDef, Content, ItemDef, ItemEffect, RecipeDef, Registry, ResourceDef, TechDef,
    TechUnlock,
};
use serde::de::DeserializeOwned;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum ContentError {
    Io {
        file: String,
        err: String,
    },
    Parse {
        file: String,
        err: String,
    },
    DuplicateId {
        file: String,
        id: String,
    },
    MissingRef {
        kind: &'static str,
        id: String,
        referenced_by: String,
    },
}

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentError::Io { file, err } => write!(f, "baca {file}: {err}"),
            ContentError::Parse { file, err } => write!(f, "parse {file}: {err}"),
            ContentError::DuplicateId { file, id } => write!(f, "id duplikat '{id}' di {file}"),
            ContentError::MissingRef {
                kind,
                id,
                referenced_by,
            } => {
                write!(f, "{kind} '{id}' (dirujuk {referenced_by}) tidak ada")
            }
        }
    }
}
impl std::error::Error for ContentError {}

fn read_vec<D: DeserializeOwned>(dir: &Path, file: &str) -> Result<Vec<D>, ContentError> {
    let path = dir.join(file);
    let text = fs::read_to_string(&path).map_err(|e| ContentError::Io {
        file: file.to_string(),
        err: e.to_string(),
    })?;
    ron::from_str(&text).map_err(|e| ContentError::Parse {
        file: file.to_string(),
        err: e.to_string(),
    })
}

fn build_reg<D: crate::game::defs::HasId>(
    defs: Vec<D>,
    file: &str,
) -> Result<Registry<D>, ContentError> {
    Registry::build(defs).map_err(|id| ContentError::DuplicateId {
        file: file.to_string(),
        id,
    })
}

/// Load semua katalog → `Content`, intern id, validasi referensi silang.
pub fn load_content(dir: &Path) -> Result<Content, ContentError> {
    let resources = build_reg(
        read_vec::<ResourceDef>(dir, "resources.ron")?,
        "resources.ron",
    )?;
    let items = build_reg(read_vec::<ItemDef>(dir, "items.ron")?, "items.ron")?;
    let recipes = build_reg(read_vec::<RecipeDef>(dir, "recipes.ron")?, "recipes.ron")?;
    let buildings = build_reg(
        read_vec::<BuildingDef>(dir, "buildings.ron")?,
        "buildings.ron",
    )?;
    let techs = build_reg(read_vec::<TechDef>(dir, "tech_tree.ron")?, "tech_tree.ron")?;
    let content = Content {
        resources,
        items,
        recipes,
        buildings,
        techs,
    };
    validate(&content)?;
    Ok(content)
}

fn need_resource(c: &Content, id: &str, by: &str) -> Result<(), ContentError> {
    if c.resources.contains(id) {
        Ok(())
    } else {
        Err(ContentError::MissingRef {
            kind: "resource",
            id: id.to_string(),
            referenced_by: by.to_string(),
        })
    }
}

/// Validasi: setiap id resource/item/recipe/building/tech yang dirujuk antar-katalog ada.
pub fn validate(c: &Content) -> Result<(), ContentError> {
    // Recipe → resource (inputs/outputs), item (item_outputs), building (building_id).
    for r in c.recipes.iter() {
        let by = format!("recipe {}", r.id);
        for (res, _) in r.inputs.iter().chain(r.outputs.iter()) {
            need_resource(c, res, &by)?;
        }
        for (it, _) in &r.item_outputs {
            if !c.items.contains(it) {
                return Err(ContentError::MissingRef {
                    kind: "item",
                    id: it.clone(),
                    referenced_by: by,
                });
            }
        }
        if !c.buildings.contains(&r.building_id) {
            return Err(ContentError::MissingRef {
                kind: "building",
                id: r.building_id.clone(),
                referenced_by: by,
            });
        }
    }
    // Building → resource (cost/upkeep), recipe (recipes).
    for b in c.buildings.iter() {
        let by = format!("building {}", b.id);
        for (res, _) in b.base_cost.iter().chain(b.upkeep.iter()) {
            need_resource(c, res, &by)?;
        }
        for rc in &b.recipes {
            if !c.recipes.contains(rc) {
                return Err(ContentError::MissingRef {
                    kind: "recipe",
                    id: rc.clone(),
                    referenced_by: by,
                });
            }
        }
    }
    // Item → recipe (UnlockRecipe), resource (InstantResources).
    for it in c.items.iter() {
        let by = format!("item {}", it.id);
        match &it.effect {
            ItemEffect::UnlockRecipe { recipe } if !c.recipes.contains(recipe) => {
                return Err(ContentError::MissingRef {
                    kind: "recipe",
                    id: recipe.clone(),
                    referenced_by: by,
                });
            }
            ItemEffect::InstantResources(list) => {
                for (res, _) in list {
                    need_resource(c, res, &by)?;
                }
            }
            _ => {}
        }
    }
    // Tech → tech (depends_on), recipe/building/resource (unlock).
    for t in c.techs.iter() {
        let by = format!("tech {}", t.id);
        for dep in &t.depends_on {
            if !c.techs.contains(dep) {
                return Err(ContentError::MissingRef {
                    kind: "tech",
                    id: dep.clone(),
                    referenced_by: by,
                });
            }
        }
        match &t.unlock {
            TechUnlock::Recipe(r) if !c.recipes.contains(r) => {
                return Err(ContentError::MissingRef {
                    kind: "recipe",
                    id: r.clone(),
                    referenced_by: by,
                });
            }
            TechUnlock::UnlockBuilding(b) if !c.buildings.contains(b) => {
                return Err(ContentError::MissingRef {
                    kind: "building",
                    id: b.clone(),
                    referenced_by: by,
                });
            }
            TechUnlock::TechMult { resources, .. } => {
                for res in resources {
                    need_resource(c, res, &by)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
    }

    #[test]
    fn loads_and_validates() {
        let c = load_content(&data_dir()).expect("content harus load + valid");
        assert!(c.resources.len() >= 80);
        assert!(c.items.len() >= 50);
        assert!(c.recipes.len() >= 40);
        assert!(c.buildings.len() >= 25);
        assert!(c.techs.len() >= 7);
        // id dikenal ter-intern.
        assert!(c.resources.contains("iron"));
        assert!(c.recipes.contains("steel_mill"));
        assert!(c.buildings.contains("smelter"));
    }

    #[test]
    fn dangling_ref_rejected() {
        let mut c = load_content(&data_dir()).unwrap();
        // Rusak satu registry building → recipe steel_mill jadi menggantung.
        c.buildings = Registry::build(vec![]).unwrap();
        assert!(validate(&c).is_err());
    }
}
