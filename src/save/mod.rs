//! Save system: path XDG, atomic JSON write, read + migrasi versi, konversi id↔string.
//!
//! `07-architecture.md` §Save System. Format JSON (`serde_json`), id konten disimpan sebagai
//! string (lihat [`dto`]). Atomic: tulis `*.tmp` lalu `rename`. Lokasi:
//! `${XDG_DATA_HOME:-~/.local/share}/galaxy-idle/save.json`.
//!
//! Dipanggil `App::manual_save` (`app.rs`'s `event_loop`: autosave berkala, quit, `Ctrl+S`).

pub mod dto;

use crate::game::defs::Content;
use crate::game::state::{GameState, SAVE_VERSION};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum SaveError {
    Io(String),
    Parse(String),
    /// Versi save lebih baru dari yang didukung biner ini.
    VersionTooNew(u32),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Io(e) => write!(f, "save io: {e}"),
            SaveError::Parse(e) => write!(f, "save parse: {e}"),
            SaveError::VersionTooNew(v) => {
                write!(f, "save versi {v} > didukung {SAVE_VERSION}")
            }
        }
    }
}
impl std::error::Error for SaveError {}

/// `${XDG_DATA_HOME:-~/.local/share}/galaxy-idle/save.json`.
pub fn save_path() -> PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_default();
            home.join(".local").join("share")
        });
    base.join("galaxy-idle").join("save.json")
}

/// Tulis save ke lokasi XDG default (atomic).
pub fn write(state: &GameState, content: &Content) -> Result<(), SaveError> {
    write_to(&save_path(), state, content)
}

/// Tulis save ke `path` tertentu (atomic via `*.tmp` + rename). Buat dir induk bila perlu.
pub fn write_to(path: &Path, state: &GameState, content: &Content) -> Result<(), SaveError> {
    let data = dto::to_save(state, content);
    let json = serde_json::to_vec_pretty(&data).map_err(|e| SaveError::Parse(e.to_string()))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| SaveError::Io(e.to_string()))?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &json).map_err(|e| SaveError::Io(e.to_string()))?;
    std::fs::rename(&tmp, path).map_err(|e| SaveError::Io(e.to_string()))?;
    Ok(())
}

/// Baca save dari lokasi XDG default.
pub fn read(content: &Content) -> Result<GameState, SaveError> {
    read_from(&save_path(), content)
}

/// Baca + migrasi save dari `path`, konversi string→id via `content`.
pub fn read_from(path: &Path, content: &Content) -> Result<GameState, SaveError> {
    let text = std::fs::read_to_string(path).map_err(|e| SaveError::Io(e.to_string()))?;
    let mut value: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| SaveError::Parse(e.to_string()))?;
    let version = value.get("version").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    if version > SAVE_VERSION {
        return Err(SaveError::VersionTooNew(version));
    }
    migrate(&mut value, version);
    let data: dto::SaveData =
        serde_json::from_value(value).map_err(|e| SaveError::Parse(e.to_string()))?;
    Ok(dto::from_save(&data, content))
}

/// Migrasi save lama → skema saat ini, berurutan per versi.
/// Tambah cabang `if version < N { ... ubah value ke skema N ... }` saat `SAVE_VERSION` naik.
fn migrate(value: &mut serde_json::Value, version: u32) {
    // M3: v1→v2, tambah `tutorial_step`. Save v1 TAK py field ini di JSON sama sekali —
    // pemain existing dianggap sudah lewat onboarding (`null`/`None`), bukan dipaksa ulang
    // dr step 0. Real migrasi PERTAMA lewat fungsi ini (dulu genuinely no-op).
    if version < 2
        && let Some(obj) = value.as_object_mut()
    {
        obj.entry("tutorial_step")
            .or_insert(serde_json::Value::Null);
    }
    // M6: v2→v3, tambah `quests` (`QuestState` kosong — save lama blm py quest apa pun aktif,
    // bukan diam2 ditandai selesai/gagal).
    if version < 3
        && let Some(obj) = value.as_object_mut()
    {
        obj.entry("quests")
            .or_insert_with(|| serde_json::json!({ "active": {}, "completed": [], "choices": {} }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_content;

    fn content() -> Content {
        load_content(&Path::new(env!("CARGO_MANIFEST_DIR")).join("data")).unwrap()
    }

    fn tmp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "galaxy_idle_test_{name}_{}.json",
            std::process::id()
        ))
    }

    #[test]
    fn save_path_uses_xdg() {
        // `XDG_DATA_HOME` proses-global. Test ini dulu set/remove-nya BEBAS LOCK, jadi balapan
        // dgn test title di `app.rs` yg juga pakai env var itu -- `save_path()` mereka sesekali
        // resolve ke `/tmp/xdgtest` (yg bisa berisi save sisa run lain) → flake. Pinjam lock yg
        // sama (`with_temp_save_dir`), set nilai spesifik DI DALAM closure-nya.
        crate::app::tests::with_temp_save_dir(|| {
            // SAFETY: di dalam ENV_LOCK — tak ada test lain menyentuh env var ini bersamaan.
            unsafe { std::env::set_var("XDG_DATA_HOME", "/tmp/xdgtest") };
            assert_eq!(
                save_path(),
                PathBuf::from("/tmp/xdgtest/galaxy-idle/save.json")
            );
        });
    }

    #[test]
    fn write_then_read_roundtrips_demo() {
        let c = content();
        let app = crate::app::App::demo();
        let path = tmp_path("roundtrip");
        write_to(&path, &app.state, &c).unwrap();
        let loaded = read_from(&path, &c).unwrap();
        // Field skalar identik.
        assert_eq!(loaded.tick, app.state.tick);
        assert_eq!(loaded.version, app.state.version);
        assert!((loaded.credits - app.state.credits).abs() < 1e-9);
        assert_eq!(loaded.ship.warp_tier, app.state.ship.warp_tier);
        assert_eq!(loaded.ship.engine, app.state.ship.engine);
        // Galaksi + planet + factory utuh.
        assert_eq!(loaded.galaxies.len(), app.state.galaxies.len());
        let pe = &loaded.galaxies[0].planets[0];
        let oe = &app.state.galaxies[0].planets[0];
        assert_eq!(pe.name, oe.name);
        assert_eq!(pe.nodes.len(), oe.nodes.len());
        assert_eq!(pe.factory_slots.len(), oe.factory_slots.len());
        // Stockpile (id→string→id) identik per resource.
        for (r, v) in &oe.stockpile {
            assert!((pe.stockpile.get(r).copied().unwrap_or(0.0) - v).abs() < 1e-9);
        }
        // Research aktif (string id) terbawa.
        assert_eq!(
            loaded.research.active.map(|a| a.tech_id),
            app.state.research.active.clone().map(|a| a.tech_id)
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn json_stores_string_ids() {
        let c = content();
        let app = crate::app::App::demo();
        let data = dto::to_save(&app.state, &c);
        let json = serde_json::to_string(&data).unwrap();
        // Bukti id konten disimpan sebagai string (portabilitas), mis. resource "iron".
        assert!(json.contains("\"iron\""));
        assert!(json.contains("research_lab") || json.contains("mining_drill"));
    }

    #[test]
    fn save_load_then_offline_progresses() {
        // M7 round-trip penuh: save → load → offline batch lanjut dari state termuat.
        let c = content();
        let app = crate::app::App::demo();
        let path = tmp_path("save_offline");
        write_to(&path, &app.state, &c).unwrap();
        let mut loaded = read_from(&path, &c).unwrap();
        let tick_before = loaded.tick;
        let report = crate::sim::offline::apply_offline(&mut loaded, &c, 100);
        // 100s offline × 0.75 = 75 tick disimulasikan dari state termuat.
        assert_eq!(report.sim_secs, 75);
        assert_eq!(loaded.tick, tick_before + 75);
        assert_eq!(loaded.last_saved_unix, 100);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn version_too_new_rejected() {
        let c = content();
        let app = crate::app::App::demo();
        let path = tmp_path("toonew");
        write_to(&path, &app.state, &c).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let bumped = text.replacen(
            &format!("\"version\": {SAVE_VERSION}"),
            &format!("\"version\": {}", SAVE_VERSION + 1),
            1,
        );
        std::fs::write(&path, bumped).unwrap();
        assert!(matches!(
            read_from(&path, &c),
            Err(SaveError::VersionTooNew(_))
        ));
        let _ = std::fs::remove_file(&path);
    }

    /// M3: `migrate`'s first REAL branch (v1→v2, `tutorial_step`). Simulates an actual v1 save
    /// on disk (no `tutorial_step` key at all, `"version": 1`) — confirms it loads cleanly
    /// (no panic/error) and lands on `None` (existing players not forced back into onboarding).
    #[test]
    fn v1_save_without_tutorial_step_migrates_to_none() {
        let c = content();
        let app = crate::app::App::demo();
        let path = tmp_path("migrate_v1");
        let mut data = dto::to_save(&app.state, &c);
        data.version = 1;
        let mut json: serde_json::Value = serde_json::to_value(&data).expect("SaveData -> Value");
        json.as_object_mut().unwrap().remove("tutorial_step");
        json["version"] = serde_json::Value::from(1);
        std::fs::write(&path, serde_json::to_vec_pretty(&json).unwrap()).unwrap();
        let loaded = read_from(&path, &c).expect("v1 save (no tutorial_step) harus load bersih");
        assert_eq!(loaded.tutorial_step, None);
        let _ = std::fs::remove_file(&path);
    }
}
